import { hashPassword, randomToken, verifyPassword } from './crypto';
import * as db from './db';
import { isoIn, isoNow } from './text';
import type { EntryRow, StagedSession, TagRow, UserRow } from './types';

const SESSION_KEY = 'session';
const SESSION_DAYS = 30;

/** Mirrors the server's error shape so the UI needs no special case. */
export class LocalError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = 'LocalError';
  }
}

async function findByEmail(email: string): Promise<UserRow | undefined> {
  return db.findByIndex<UserRow>(db.STORES.users, 'byEmail', email.trim().toLowerCase());
}

export async function register(body: {
  email: string;
  display_name: string;
  password: string;
}): Promise<{ user: UserRow; expires_at: string }> {
  if (await findByEmail(body.email)) {
    throw new LocalError(409, 'conflict', 'email: is already registered');
  }

  const user: UserRow = {
    id: crypto.randomUUID(),
    email: body.email.trim().toLowerCase(),
    display_name: body.display_name.trim(),
    password_hash: await hashPassword(body.password),
    created_at: isoNow(),
    updated_at: isoNow(),
  };

  await db.put(db.STORES.users, user);
  const expires_at = await openSession(user.id);

  return { user, expires_at };
}

export async function login(body: {
  email: string;
  password: string;
}): Promise<{ user: UserRow; expires_at: string }> {
  const user = await findByEmail(body.email);
  if (!user || !(await verifyPassword(body.password, user.password_hash))) {
    throw new LocalError(401, 'unauthorized', 'Email or password is incorrect.');
  }

  const expires_at = await openSession(user.id);
  return { user, expires_at };
}

async function openSession(userId: string): Promise<string> {
  const expires_at = isoIn(SESSION_DAYS);
  await db.setMeta<StagedSession>(SESSION_KEY, {
    token: randomToken(),
    user_id: userId,
    expires_at,
  });
  return expires_at;
}

export async function logout(): Promise<void> {
  await db.remove(db.STORES.meta, SESSION_KEY);
}

/** Current account, or null when nobody is signed in. */
export async function currentUser(): Promise<UserRow | null> {
  const session = await db.getMeta<StagedSession>(SESSION_KEY);
  if (!session || session.expires_at <= isoNow()) {
    return null;
  }

  const user = await db.get<UserRow>(db.STORES.users, session.user_id);
  if (!user) {
    await logout();
    return null;
  }
  return user;
}

export async function requireUser(): Promise<UserRow> {
  const user = await currentUser();
  if (!user) {
    throw new LocalError(401, 'unauthorized', 'Sign in to continue.');
  }
  return user;
}

export async function changePassword(body: {
  current_password: string;
  new_password: string;
}): Promise<{ message: string }> {
  const user = await requireUser();

  if (!(await verifyPassword(body.current_password, user.password_hash))) {
    throw new LocalError(401, 'unauthorized', 'Email or password is incorrect.');
  }

  await db.put<UserRow>(db.STORES.users, {
    ...user,
    password_hash: await hashPassword(body.new_password),
    updated_at: isoNow(),
  });

  return { message: 'Password updated.' };
}

export async function updateProfile(body: { display_name: string }): Promise<UserRow> {
  const user = await requireUser();
  const updated: UserRow = { ...user, display_name: body.display_name.trim(), updated_at: isoNow() };

  await db.put(db.STORES.users, updated);
  return updated;
}

/** Removes every row belonging to the account, including the session. */
export async function deleteAccount(): Promise<{
  message: string;
  summary: { entries_deleted: number; attachments_deleted: number };
}> {
  const user = await requireUser();

  const [entries, tags] = await Promise.all([
    db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', user.id),
    db.getByIndex<TagRow>(db.STORES.tags, 'byUser', user.id),
  ]);

  await Promise.all([
    db.remove(db.STORES.users, user.id),
    db.remove(db.STORES.meta, SESSION_KEY),
    ...entries.map((entry) => db.remove(db.STORES.entries, entry.id)),
    ...tags.map((tag) => db.remove(db.STORES.tags, tag.id)),
  ]);

  return {
    message: 'Account deleted.',
    summary: { entries_deleted: entries.length, attachments_deleted: 0 },
  };
}

/** Public shape of an account. The password hash never leaves this module. */
export function toProfile(user: UserRow) {
  return {
    id: user.id,
    email: user.email,
    display_name: user.display_name,
    created_at: user.created_at,
  };
}