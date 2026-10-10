import { request } from '@/lib/api-client';

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

export const insightsApi = {
  overview: (signal?: AbortSignal) => request<Insights>({ path: '/insights', signal }),
};