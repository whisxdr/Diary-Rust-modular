import { request } from '@/lib/api-client';

export type Theme = 'light' | 'dark' | 'system';
export type Weekday = 'monday' | 'sunday';

export interface Preferences {
  theme: Theme;
  week_starts_on: Weekday;
  entries_per_page: number;
  show_word_count: boolean;
}

export interface UpdatePreferences {
  theme?: Theme;
  week_starts_on?: Weekday;
  entries_per_page?: number;
  show_word_count?: boolean;
}

export interface Profile {
  id: string;
  email: string;
  display_name: string;
  created_at: string;
}

export const settingsApi = {
  preferences: (signal?: AbortSignal) => request<Preferences>({ path: '/settings/preferences', signal }),

  updatePreferences: (body: UpdatePreferences) =>
    request<Preferences>({ path: '/settings/preferences', method: 'PATCH', body }),

  profile: (signal?: AbortSignal) => request<Profile>({ path: '/account', signal }),

  updateProfile: (body: { display_name: string }) =>
    request<Profile>({ path: '/account', method: 'PATCH', body }),

  deleteAccount: (body: { confirm_display_name: string }) =>
    request<{ message: string; summary: { entries_deleted: number; attachments_deleted: number } }>({
      path: '/account',
      method: 'DELETE',
      body,
    }),
};