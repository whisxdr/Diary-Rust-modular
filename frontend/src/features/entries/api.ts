import { request } from '@/lib/api-client';

import type { CreateEntryInput, Entry, EntryFilters, EntryStatus, EntrySummary, Page } from './types';

export const entriesApi = {
  list: (filters: EntryFilters, signal?: AbortSignal) =>
    request<Page<EntrySummary>>({ path: `/entries${toQuery(filters)}`, signal }),

  byDate: (date: string, signal?: AbortSignal) =>
    request<Entry | null>({ path: `/entries/by-date/${date}`, signal }),

  byId: (id: string, signal?: AbortSignal) => request<Entry>({ path: `/entries/${id}`, signal }),

  create: (body: CreateEntryInput) => request<Entry>({ path: '/entries', method: 'POST', body }),

  update: (
    id: string,
    body: { title?: string; content?: string; mood_id?: string | null; tags?: string[] },
  ) => request<Entry>({ path: `/entries/${id}`, method: 'PATCH', body }),

  autosave: (id: string, body: { title?: string; content?: string }) =>
    request<Entry>({ path: `/entries/${id}/autosave`, method: 'PATCH', body }),

  setStatus: (id: string, status: EntryStatus) =>
    request<Entry>({ path: `/entries/${id}/status`, method: 'PATCH', body: { status } }),

  remove: (id: string) => request<void>({ path: `/entries/${id}`, method: 'DELETE' }),
};

function toQuery(filters: EntryFilters): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(filters)) {
    if (value !== undefined && value !== '') {
      params.set(key, String(value));
    }
  }
  const query = params.toString();
  return query ? `?${query}` : '';
}