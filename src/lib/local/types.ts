/** Row shapes stored in the browser. They mirror the API responses. */

export interface UserRow {
  id: string;
  email: string;
  display_name: string;
  /** PBKDF2 digest, never a plain password. */
  password_hash: string;
  created_at: string;
  updated_at: string;
}

export interface EntryRow {
  id: string;
  user_id: string;
  date: string;
  title: string;
  content: string;
  word_count: number;
  mood_id: string | null;
  status: string;
  /** Tag ids attached to this entry. */
  tag_ids: string[];
  created_at: string;
  updated_at: string;
}

export interface TagRow {
  id: string;
  user_id: string;
  name: string;
  normalized: string;
  color: string | null;
  created_at: string;
}

export interface StagedImport {
  job_id: string;
  filename: string;
  status: string;
  created_at: string;
  completed_at: string | null;
  archive: unknown;
  summary: unknown;
}

export interface StagedSession {
  token: string;
  user_id: string;
  expires_at: string;
}