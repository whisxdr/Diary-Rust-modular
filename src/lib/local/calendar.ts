import { requireUser } from './accounts';
import * as db from './db';
import { moodById } from './moods';
import type { EntryRow } from './types';

export interface CalendarDay {
  date: string;
  has_entry: boolean;
  word_count: number;
  mood_slug: string | null;
  is_today: boolean;
}

function toIso(date: Date): string {
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${date.getFullYear()}-${month}-${day}`;
}

/** Reads the year and month the API accepts as query parameters. */
export function monthFromParams(params: URLSearchParams) {
  return month(Number(params.get('year')), Number(params.get('month')));
}

/** Day cells for one month, including days without an entry. */
export async function month(year: number, monthNumber: number) {
  const user = await requireUser();
  const entries = await db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', user.id);

  const first = new Date(year, monthNumber - 1, 1);
  const last = new Date(year, monthNumber, 0);
  const today = toIso(new Date());

  const days: CalendarDay[] = [];

  for (let day = 1; day <= last.getDate(); day += 1) {
    const date = toIso(new Date(year, monthNumber - 1, day));
    const entry = entries.find((candidate) => candidate.date === date);

    days.push({
      date,
      has_entry: Boolean(entry),
      word_count: entry?.word_count ?? 0,
      mood_slug: moodById(entry?.mood_id ?? null)?.slug ?? null,
      is_today: date === today,
    });
  }

  return {
    first_date: toIso(first),
    last_date: toIso(last),
    days,
    total_entries: days.filter((day) => day.has_entry).length,
    total_words: entries
      .filter((entry) => entry.date >= toIso(first) && entry.date <= toIso(last))
      .reduce((total, entry) => total + entry.word_count, 0),
  };
}

/** Years that contain at least one entry. */
export async function years(): Promise<number[]> {
  const user = await requireUser();
  const entries = await db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', user.id);

  const found = new Set(entries.map((entry) => Number(entry.date.slice(0, 4))));
  return [...found].filter((year) => Number.isFinite(year)).sort((a, b) => a - b);
}