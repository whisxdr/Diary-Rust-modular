import { request } from '@/lib/api-client';

export interface Mood {
  id: string;
  slug: string;
  label: string;
  icon: string;
}

/** The mood catalog is seeded by the backend and shared by everyone. */
export const moodsApi = {
  list: (signal?: AbortSignal) => request<Mood[]>({ path: '/moods', signal }),
};