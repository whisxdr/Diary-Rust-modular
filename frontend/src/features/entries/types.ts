export type EntryStatus = 'draft' | 'published';

export interface Mood {
  id: string;
  slug: string;
  label: string;
  icon: string;
}

export interface Tag {
  id: string;
  name: string;
  color: string | null;
  entry_count?: number;
}

export interface Entry {
  id: string;
  date: string;
  title: string;
  content: string;
  word_count: number;
  mood: Mood | null;
  status: EntryStatus;
  tags: Tag[];
  created_at: string;
  updated_at: string;
}

export interface EntrySummary {
  id: string;
  date: string;
  title: string;
  excerpt: string;
  word_count: number;
  status: EntryStatus;
  updated_at: string;
}

export interface Page<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
}

export interface EntryFilters {
  from?: string | undefined;
  to?: string | undefined;
  mood_id?: string | undefined;
  tag?: string | undefined;
  status?: EntryStatus | undefined;
  limit?: number | undefined;
  offset?: number | undefined;
}

export interface CreateEntryInput {
  date: string;
  title: string;
  content: string;
  mood_id: string | null;
  tags: string[];
}