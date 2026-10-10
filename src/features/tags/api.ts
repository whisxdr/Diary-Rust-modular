import { request } from '@/lib/api-client';

export interface Tag {
  id: string;
  name: string;
  color: string | null;
  entry_count?: number;
}

export const tagsApi = {
  list: (signal?: AbortSignal) => request<Tag[]>({ path: '/tags', signal }),

  create: (body: { name: string; color?: string | null }) =>
    request<Tag>({ path: '/tags', method: 'POST', body }),

  rename: (id: string, name: string) =>
    request<Tag>({ path: `/tags/${id}`, method: 'PATCH', body: { name } }),

  remove: (id: string) => request<void>({ path: `/tags/${id}`, method: 'DELETE' }),
};