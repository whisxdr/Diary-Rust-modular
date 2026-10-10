import { LocalError, requireUser } from './accounts';
import * as db from './db';

export type Theme = 'light' | 'dark' | 'system';

export interface Preferences {
  theme: Theme;
  week_starts_on: 'monday' | 'sunday';
  entries_per_page: number;
  show_word_count: boolean;
}

const PREFS_KEY = 'preferences';

const DEFAULTS: Preferences = {
  theme: 'system',
  week_starts_on: 'monday',
  entries_per_page: 20,
  show_word_count: true,
};

export async function get(): Promise<Preferences> {
  const stored = await db.getMeta<Partial<Preferences>>(PREFS_KEY);
  return { ...DEFAULTS, ...stored };
}

export async function update(patch: Partial<Preferences>): Promise<Preferences> {
  await requireUser();

  if (patch.entries_per_page !== undefined && (patch.entries_per_page < 5 || patch.entries_per_page > 100)) {
    throw new LocalError(422, 'validation_error', 'entries_per_page must be between 5 and 100');
  }

  const next = { ...(await get()), ...patch };
  await db.setMeta(PREFS_KEY, next);
  return next;
}