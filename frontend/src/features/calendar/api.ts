import { request } from '@/lib/api-client';

export interface CalendarDay {
  date: string;
  has_entry: boolean;
  word_count: number;
  mood_slug: string | null;
  is_today: boolean;
}

export interface CalendarMonth {
  first_date: string;
  last_date: string;
  days: CalendarDay[];
  total_entries: number;
  total_words: number;
}

export const calendarApi = {
  month: (year: number, month: number, signal?: AbortSignal) =>
    request<CalendarMonth>({ path: `/calendar/month?year=${year}&month=${month}`, signal }),

  years: (signal?: AbortSignal) => request<number[]>({ path: '/calendar/years', signal }),
};