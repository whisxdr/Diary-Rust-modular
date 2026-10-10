import { request } from '@/lib/api-client';

export interface SearchHit {
  entry_id: string;
  date: string;
  title: string;
  snippet: string;
  rank: number;
}

export interface SearchPage {
  items: SearchHit[];
  total: number;
  limit: number;
  offset: number;
}

export const searchApi = {
  query: (term: string, signal?: AbortSignal) =>
    request<SearchPage>({ path: `/search?q=${encodeURIComponent(term)}`, signal }),
};