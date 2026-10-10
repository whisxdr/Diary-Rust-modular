import { LocalError } from './accounts';
import * as db from './db';
import { isoNow } from './text';
import type { EntryRow, TagRow } from './types';

/** Every tag of the account with how many entries use it. */
export async function list(userId: string): Promise<(TagRow & { entry_count: number })[]> {
  const [tags, entries] = await Promise.all([
    db.getByIndex<TagRow>(db.STORES.tags, 'byUser', userId),
    db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', userId),
  ]);

  const usage = new Map<string, number>();
  for (const entry of entries) {
    for (const tagId of entry.tag_ids) {
      usage.set(tagId, (usage.get(tagId) ?? 0) + 1);
    }
  }

  return tags
    .map((tag) => ({ ...tag, entry_count: usage.get(tag.id) ?? 0 }))
    .sort((left, right) => left.normalized.localeCompare(right.normalized));
}

export async function findByName(userId: string, name: string): Promise<TagRow | undefined> {
  return db.findByIndex<TagRow>(db.STORES.tags, 'byNormalized', [
    userId,
    name.trim().toLowerCase(),
  ]);
}

/** Returns the existing tag or creates it, like the server's find-or-create. */
export async function resolve(userId: string, name: string): Promise<TagRow> {
  const normalized = name.trim().toLowerCase();
  const existing = await findByName(userId, normalized);
  if (existing) {
    return existing;
  }

  const tag: TagRow = {
    id: crypto.randomUUID(),
    user_id: userId,
    name: name.trim(),
    normalized,
    color: null,
    created_at: isoNow(),
  };

  await db.put(db.STORES.tags, tag);
  return tag;
}

export async function resolveAll(userId: string, names: string[]): Promise<TagRow[]> {
  const resolved: TagRow[] = [];
  for (const name of names) {
    if (name.trim()) {
      resolved.push(await resolve(userId, name));
    }
  }
  return resolved;
}

export async function rename(
  userId: string,
  id: string,
  name: string,
): Promise<TagRow | undefined> {
  const tag = await db.get<TagRow>(db.STORES.tags, id);
  if (!tag || tag.user_id !== userId) {
    return undefined;
  }

  const updated: TagRow = { ...tag, name: name.trim(), normalized: name.trim().toLowerCase() };
  await db.put(db.STORES.tags, updated);
  return updated;
}

/** Deletes the tag and removes it from every entry that used it. */
export async function remove(userId: string, id: string): Promise<boolean> {
  const tag = await db.get<TagRow>(db.STORES.tags, id);
  if (!tag || tag.user_id !== userId) {
    return false;
  }

  const entries = await db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', userId);

  await Promise.all([
    db.remove(db.STORES.tags, id),
    ...entries
      .filter((entry) => entry.tag_ids.includes(id))
      .map((entry) =>
        db.put(db.STORES.entries, {
          ...entry,
          tag_ids: entry.tag_ids.filter((value) => value !== id),
        }),
      ),
  ]);

  return true;
}

/** Tag summaries for an entry, in the shape the API returns. */
export function tagsOf(entry: EntryRow, all: TagRow[]) {
  return entry.tag_ids
    .map((id) => all.find((tag) => tag.id === id))
    .filter((tag): tag is TagRow => tag !== undefined)
    .map((tag) => ({ id: tag.id, name: tag.name, color: tag.color }));
}

// Thin wrappers that shape rows exactly like the API, for the router.

export async function listForUser(user: { id: string }) {
  const rows = await list(user.id);
  return rows.map((tag) => ({ id: tag.id, name: tag.name, color: tag.color, entry_count: tag.entry_count }));
}

export async function createForUser(userId: string, name: string) {
  const tag = await resolve(userId, name);
  return { id: tag.id, name: tag.name, color: tag.color, entry_count: 0 };
}

export async function renameForUser(userId: string, id: string, name: string) {
  const tag = await rename(userId, id, name);
  if (!tag) {
    throw new LocalError(404, 'not_found', 'Tag not found.');
  }
  return { id: tag.id, name: tag.name, color: tag.color, entry_count: 0 };
}

export async function removeForUser(userId: string, id: string) {
  await remove(userId, id);
}