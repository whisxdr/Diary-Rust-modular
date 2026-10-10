import { requireUser } from './accounts';
import * as db from './db';
import { moodById, MOODS } from './moods';
import * as tags from './tags';
import type { EntryRow } from './types';

export interface Insights {
  total_entries: number;
  total_words: number;
  current_streak: number;
  longest_streak: number;
  first_entry_date: string | null;
  mood_distribution: { slug: string; label: string; entries: number; share: number }[];
  top_tags: { name: string; color: string | null; entries: number }[];
  writing_by_weekday: { weekday: number; entries: number }[];
}

const WEEKDAY_COUNT = 7;

export async function overview(): Promise<Insights> {
  const user = await requireUser();
  const [entries, tagRows] = await Promise.all([
    db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', user.id),
    tags.list(user.id),
  ]);

  const dates = entries.map((entry) => entry.date).sort();
  const { current, longest } = streaks(dates);

  return {
    total_entries: entries.length,
    total_words: entries.reduce((total, entry) => total + entry.word_count, 0),
    current_streak: current,
    longest_streak: longest,
    first_entry_date: dates[0] ?? null,
    mood_distribution: moodDistribution(entries),
    top_tags: tagRows
      .filter((tag) => tag.entry_count > 0)
      .slice(0, 8)
      .map((tag) => ({ name: tag.name, color: tag.color, entries: tag.entry_count })),
    writing_by_weekday: Array.from({ length: WEEKDAY_COUNT }, (_, weekday) => ({
      weekday,
      entries: entries.filter((entry) => new Date(`${entry.date}T00:00:00`).getDay() === weekday)
        .length,
    })),
  };
}

/** Mirrors the server's streak rules, including the grace day. */
export function streaks(dates: string[]): { current: number; longest: number } {
  if (dates.length === 0) {
    return { current: 0, longest: 0 };
  }

  let longest = 1;
  let run = 1;

  for (let index = 1; index < dates.length; index += 1) {
    const previous = Date.parse(`${dates[index - 1]}T00:00:00`);
    const next = Date.parse(`${dates[index]}T00:00:00`);
    if (next - previous === 86_400_000) {
      run += 1;
    } else {
      longest = Math.max(longest, run);
      run = 1;
    }
  }
  longest = Math.max(longest, run);

  const last = dates[dates.length - 1] ?? '';
  const today = new Date().toISOString().slice(0, 10);
  const yesterday = new Date(Date.now() - 86_400_000).toISOString().slice(0, 10);

  return { current: last === today || last === yesterday ? run : 0, longest };
}

function moodDistribution(entries: EntryRow[]) {
  const counted = MOODS.map((mood) => ({
    slug: mood.slug,
    label: mood.label,
    entries: entries.filter((entry) => moodById(entry.mood_id)?.slug === mood.slug).length,
  })).filter((row) => row.entries > 0);

  const total = counted.reduce((sum, row) => sum + row.entries, 0);
  return counted.map((row) => ({ ...row, share: total === 0 ? 0 : row.entries / total }));
}