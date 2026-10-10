import { request, upload } from '@/lib/api-client';

export interface ImportPreview {
  job_id: string | null;
  format_version: number;
  exported_at: string;
  source: string;
  entries_total: number;
  entries_new: number;
  entries_updated: number;
  tags_new: number;
  date_range: { first: string; last: string } | null;
  problems: { entry_key: string; message: string }[];
}

export interface ImportJob {
  id: string;
  filename: string;
  status: string;
  summary: unknown;
  error: string | null;
  created_at: string;
  completed_at: string | null;
}

export interface ImportSummary {
  job_id: string;
  entries_created: number;
  entries_updated: number;
  entries_skipped: number;
  tags_created: number;
  status: string;
}

export type ConflictStrategy = 'merge' | 'skip';

export const importsApi = {
  /** Validates the archive and stages it; nothing is written to the diary yet. */
  upload: (file: File, signal?: AbortSignal) => {
    const form = new FormData();
    form.append('file', file);
    return upload<ImportPreview>('/imports', form, signal);
  },

  preview: (jobId: string, signal?: AbortSignal) =>
    request<ImportPreview>({ path: `/imports/${jobId}/preview`, signal }),

  commit: (body: { job_id: string; conflict_strategy: ConflictStrategy }) =>
    request<ImportSummary>({ path: '/imports/commit', method: 'POST', body }),

  history: (signal?: AbortSignal) => request<ImportJob[]>({ path: '/imports/history', signal }),
};