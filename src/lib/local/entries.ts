import { LocalError, requireUser } from './accounts';
import * as db from './db';
import { moodById } from './moods';
import * as tags from './tags';
import { excerpt, isoNow, wordCount } from './text';
import type { EntryRow, TagRow } from './types';

export interface EntryFilters {
  from?: string | undefined;
  to?: string | undefined;
  mood_id?: string | undefined;
  tag?: string | undefined;
  status?: string | undefined;
  limit?: number | undefined;
  offset?: number | undefined;
}

const DEFAULT_LIMIT = 20;

async function rowsOf(userId: string): Promise<EntryRow[]> {
  const entries = await db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', userId);
  return entries.sort((left, right) => right.date.localeCompare(left.date));
}

/** Reads the list filters the API accepts as query parameters. */
export function listFromParams(userId: string, params: URLSearchParams): Promise<unknown> {
  return listForUser(userId, {
    from: params.get('from') ?? undefined,
    to: params.get('to') ?? undefined,
    mood_id: params.get('mood_id') ?? undefined,
    tag: params.get('tag') ?? undefined,
    status: params.get('status') ?? undefined,
    limit: params.has('limit') ? Number(params.get('limit')) : undefined,
    offset: params.has('offset') ? Number(params.get('offset')) : undefined,
  });
}

function matchesFilters(entry: EntryRow, filters: EntryFilters, tagIdByName: Map<string, string>): boolean {
  if (filters.from && entry.date < filters.from) {
    return false;
  }
  if (filters.to && entry.date > filters.to) {
    return false;
  }
  if (filters.status && entry.status !== filters.status) {
    return false;
  }
  if (filters.mood_id && entry.mood_id !== filters.mood_id) {
    return false;
  }
  if (filters.tag) {
    const wanted = tagIdByName.get(filters.tag.trim().toLowerCase());
    if (!wanted || !entry.tag_ids.includes(wanted)) {
      return false;
    }
  }
  return true;
}

export async function listForUser(userId: string, filters: EntryFilters) {
  const [entries, tagRows] = await Promise.all([rowsOf(userId), tags.list(userId)]);

  const tagIdByName = new Map(tagRows.map((tag) => [tag.normalized, tag.id]));
  const matching = entries.filter((entry) => matchesFilters(entry, filters, tagIdByName));

  const limit = filters.limit ?? DEFAULT_LIMIT;
  const offset = filters.offset ?? 0;
  const page = matching.slice(offset, offset + limit);

  return {
    items: page.map((entry) => ({
      id: entry.id,
      date: entry.date,
      title: entry.title,
      excerpt: excerpt(entry.content),
      word_count: entry.word_count,
      status: entry.status,
      updated_at: entry.updated_at,
    })),
    total: matching.length,
    limit,
    offset,
  };
}

export async function byDate(date: string) {
  const user = await requireUser();
  const entry = await db.findByIndex<EntryRow>(db.STORES.entries, 'byUserDate', [user.id, date]);
  return entry ? hydrate(entry, user.id) : null;
}

export async function byId(id: string) {
  const user = await requireUser();
  const entry = await db.get<EntryRow>(db.STORES.entries, id);
  if (!entry || entry.user_id !== user.id) {
    throw new LocalError(404, 'not_found', 'Entry not found.');
  }
  return hydrate(entry, user.id);
}

export async function create(body: {
  date: string;
  title?: string;
  content?: string;
  mood_id?: string | null;
  tags?: string[];
}) {
  const user = await requireUser();
  const existing = await db.findByIndex<EntryRow>(db.STORES.entries, 'byUserDate', [user.id, body.date]);

  if (existing) {
    throw new LocalError(409, 'conflict', 'An entry already exists for that day.');
  }

  const entry: EntryRow = {
    id: crypto.randomUUID(),
    user_id: user.id,
    date: body.date,
    title: (body.title ?? '').trim(),
    content: body.content ?? '',
    word_count: wordCount(body.content ?? ''),
    mood_id: body.mood_id ?? null,
    status: 'published',
    tag_ids: (await tags.resolveAll(user.id, body.tags ?? [])).map((tag) => tag.id),
    created_at: isoNow(),
    updated_at: isoNow(),
  };

  await db.put(db.STORES.entries, entry);
  return hydrate(entry, user.id);
}

export async function update(
  id: string,
  body: {
    title?: string;
    content?: string;
    mood_id?: string | null;
    tags?: string[];
  },
) {
  const user = await requireUser();
  const entry = await db.get<EntryRow>(db.STORES.entries, id);
  if (!entry || entry.user_id !== user.id) {
    throw new LocalError(404, 'not_found', 'Entry not found.');
  }

  const updated: EntryRow = {
    ...entry,
    title: body.title !== undefined ? body.title.trim() : entry.title,
    content: body.content !== undefined ? body.content : entry.content,
    word_count: body.content !== undefined ? wordCount(body.content) : entry.word_count,
    mood_id: body.mood_id !== undefined ? body.mood_id : entry.mood_id,
    tag_ids:
      body.tags !== undefined
        ? (await tags.resolveAll(user.id, body.tags)).map((tag) => tag.id)
        : entry.tag_ids,
    updated_at: isoNow(),
  };

  await db.put(db.STORES.entries, updated);
  return hydrate(updated, user.id);
}

export async function setStatus(id: string, status: string) {
  if (status !== 'draft' && status !== 'published') {
    throw new LocalError(422, 'validation_error', 'status: must be draft or published');
  }

  const user = await requireUser();
  const entry = await db.get<EntryRow>(db.STORES.entries, id);
  if (!entry || entry.user_id !== user.id) {
    throw new LocalError(404, 'not_found', 'Entry not found.');
  }

  const updated: EntryRow = { ...entry, status, updated_at: isoNow() };
  await db.put(db.STORES.entries, updated);
  return hydrate(updated, user.id);
}

export async function remove(id: string): Promise<void> {
  const user = await requireUser();
  const entry = await db.get<EntryRow>(db.STORES.entries, id);
  if (!entry || entry.user_id !== user.id) {
    throw new LocalError(404, 'not_found', 'Entry not found.');
  }
  await db.remove(db.STORES.entries, id);
}

/** Adds mood and tag detail to a stored row. */
async function hydrate(entry: EntryRow, userId: string) {
  const tagRows = await db.getByIndex<TagRow>(db.STORES.tags, 'byUser', userId);

  return {
    id: entry.id,
    date: entry.date,
    title: entry.title,
    content: entry.content,
    word_count: entry.word_count,
    mood: moodById(entry.mood_id),
    status: entry.status,
    tags: tags.tagsOf(entry, tagRows),
    created_at: entry.created_at,
    updated_at: entry.updated_at,
  };
}

export type HydratedEntry = Awaited<ReturnType<typeof hydrate>>;