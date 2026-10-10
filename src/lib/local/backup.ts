/**
 * Portable backup format.
 *
 * The shape matches `shared/portable/schema.rs` in the backend, so a file
 * exported here can be imported by a self-hosted install and the other way
 * round. Credentials are never part of it.
 */
import { LocalError, requireUser } from './accounts';
import * as db from './db';
import { moodById, moodIdBySlug } from './moods';
import * as tags from './tags';
import { isoNow, wordCount } from './text';
import type { EntryRow, StagedImport } from './types';

export const PORTABLE_FORMAT = 'dear-my-diary/backup';
export const PORTABLE_FORMAT_VERSION = 1;

interface PortableTag {
  key: string;
  name: string;
  color?: string | null;
}

interface PortableEntry {
  key: string;
  date: string;
  title: string;
  content: string;
  status: string;
  mood?: string | null;
  tags: string[];
  created_at: string;
  updated_at: string;
}

interface PortableArchive {
  manifest: {
    format: string;
    format_version: number;
    exported_at: string;
    source: string;
    counts: { entries: number; tags: number; attachments: number };
  };
  tags: PortableTag[];
  entries: PortableEntry[];
  attachments: unknown[];
}

const IMPORTS_KEY = 'imports';

export async function buildArchive(): Promise<PortableArchive> {
  const user = await requireUser();
  const [entries, tagRows] = await Promise.all([
    db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', user.id),
    tags.list(user.id),
  ]);

  const tagKeys = new Map(tagRows.map((tag) => [tag.id, `tag-${tag.id}`]));
  const keyFor = (tagId: string) => tagKeys.get(tagId) ?? '';

  return {
    manifest: {
      format: PORTABLE_FORMAT,
      format_version: PORTABLE_FORMAT_VERSION,
      exported_at: isoNow(),
      source: user.email,
      counts: { entries: entries.length, tags: tagRows.length, attachments: 0 },
    },
    tags: tagRows.map((tag) => ({ key: tagKeys.get(tag.id) ?? '', name: tag.name, color: tag.color })),
    entries: entries.map((entry) => ({
      key: `entry-${entry.id}`,
      date: entry.date,
      title: entry.title,
      content: entry.content,
      status: entry.status,
      mood: moodById(entry.mood_id)?.slug ?? null,
      tags: entry.tag_ids.map(keyFor).filter(Boolean),
      created_at: entry.created_at,
      updated_at: entry.updated_at,
    })),
    attachments: [],
  };
}

export function fileName(archive: PortableArchive): string {
  const stamp = archive.manifest.exported_at.replace(/[-:T]/g, '').slice(0, 15);
  return `dear-my-diary-${stamp}.json`;
}

function parse(text: string): PortableArchive {
  let archive: PortableArchive;
  try {
    archive = JSON.parse(text) as PortableArchive;
  } catch {
    throw new LocalError(422, 'validation_error', 'File is not valid JSON.');
  }

  if (archive?.manifest?.format !== PORTABLE_FORMAT) {
    throw new LocalError(422, 'validation_error', 'File is not a Dear My Diary backup.');
  }
  if (archive.manifest.format_version > PORTABLE_FORMAT_VERSION) {
    throw new LocalError(
      422,
      'validation_error',
      `Archive format version ${archive.manifest.format_version} is newer than the supported version ${PORTABLE_FORMAT_VERSION}.`,
    );
  }
  if (!Array.isArray(archive.entries)) {
    throw new LocalError(422, 'validation_error', 'File is not a Dear My Diary backup.');
  }

  return archive;
}

async function stage(filename: string, archive: PortableArchive): Promise<StagedImport> {
  const job: StagedImport = {
    job_id: crypto.randomUUID(),
    filename,
    status: 'pending',
    created_at: isoNow(),
    completed_at: null,
    archive,
    summary: null,
  };

  const history = await allImports();
  await db.setMeta(IMPORTS_KEY, [job, ...history].slice(0, 20));

  return job;
}

export async function stageUpload(filename: string, text: string) {
  const archive = parse(text);
  const job = await stage(filename, archive);
  return previewFor(job);
}

async function previewFor(job: StagedImport) {
  const user = await requireUser();
  const archive = job.archive as PortableArchive;
  const existing = await db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', user.id);

  const dates = new Set(existing.map((entry) => entry.date));
  const entriesNew = archive.entries.filter((entry) => !dates.has(entry.date)).length;

  return {
    job_id: job.job_id,
    format_version: archive.manifest.format_version,
    exported_at: archive.manifest.exported_at,
    source: archive.manifest.source,
    entries_total: archive.entries.length,
    entries_new: entriesNew,
    entries_updated: archive.entries.length - entriesNew,
    tags_new: archive.tags.length,
    date_range: dateRange(archive),
    problems: [] as { entry_key: string; message: string }[],
  };
}

function dateRange(archive: PortableArchive) {
  if (archive.entries.length === 0) {
    return null;
  }
  const dates = archive.entries.map((entry) => entry.date).sort();
  return { first: dates[0] ?? '', last: dates[dates.length - 1] ?? '' };
}

export async function preview(jobId: string) {
  const job = await allImports().then((jobs) => jobs.find((candidate) => candidate.job_id === jobId));
  if (!job) {
    throw new LocalError(404, 'not_found', 'Import not found.');
  }
  return previewFor(job);
}

export async function commit(body: { job_id: string; conflict_strategy: 'merge' | 'skip' }) {
  const user = await requireUser();
  const jobs = await allImports();
  const job = jobs.find((candidate) => candidate.job_id === body.job_id);

  if (!job) {
    throw new LocalError(404, 'not_found', 'Import not found.');
  }
  if (job.status === 'completed') {
    throw new LocalError(409, 'conflict', 'This import has already been applied.');
  }

  const archive = job.archive as PortableArchive;
  const existing = await db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', user.id);
  const byDate = new Map(existing.map((entry) => [entry.date, entry]));

  const tagIdsByKey = new Map<string, string>();
  for (const tag of archive.tags) {
    const resolved = await tags.resolve(user.id, tag.name);
    tagIdsByKey.set(tag.key, resolved.id);
  }

  let created = 0;
  let updated = 0;
  let skipped = 0;

  for (const entry of archive.entries) {
    const current = byDate.get(entry.date);

    if (current && body.conflict_strategy === 'skip') {
      skipped += 1;
      continue;
    }

    const row: EntryRow = {
      id: current?.id ?? crypto.randomUUID(),
      user_id: user.id,
      date: entry.date,
      title: entry.title ?? '',
      content: entry.content ?? '',
      word_count: wordCount(entry.content ?? ''),
      mood_id: moodIdBySlug(entry.mood),
      status: entry.status === 'draft' ? 'draft' : 'published',
      tag_ids: entry.tags
        .map((key) => tagIdsByKey.get(key))
        .filter((id): id is string => Boolean(id)),
      created_at: current?.created_at ?? entry.created_at ?? isoNow(),
      updated_at: entry.updated_at ?? isoNow(),
    };

    await db.put(db.STORES.entries, row);
    if (current) {
      updated += 1;
    } else {
      created += 1;
    }
  }

  const summary = {
    job_id: job.job_id,
    entries_created: created,
    entries_updated: updated,
    entries_skipped: skipped,
    tags_created: archive.tags.length,
    status: 'completed',
  };

  await db.setMeta(
    IMPORTS_KEY,
    jobs.map((candidate) =>
      candidate.job_id === job.job_id
        ? { ...candidate, status: 'completed', completed_at: isoNow(), summary }
        : candidate,
    ),
  );

  return summary;
}

export async function history() {
  const jobs = await allImports();
  return jobs.map(({ archive: _archive, ...rest }) => rest);
}

async function allImports(): Promise<StagedImport[]> {
  return (await db.getMeta<StagedImport[]>(IMPORTS_KEY)) ?? [];
}