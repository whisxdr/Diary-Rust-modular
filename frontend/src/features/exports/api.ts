import { download } from '@/lib/api-client';

export const exportsApi = {
  /** Saves the whole diary as a portable JSON backup. */
  archive: (fileName = 'dear-my-diary-backup.json') => download('/exports/archive', fileName),
};