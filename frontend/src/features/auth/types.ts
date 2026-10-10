export interface SessionUser {
  id: string;
  email: string;
  display_name: string;
  created_at: string;
}

export interface Session {
  user: SessionUser;
  expires_at: string;
}