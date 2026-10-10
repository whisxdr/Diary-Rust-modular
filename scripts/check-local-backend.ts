/**
 * Runnable check for the browser-local backend.
 *
 * It runs the same modules the app calls, against an in-memory IndexedDB, so a
 * broken local path fails here instead of in the browser.
 *
 *   npm run check:local
 */
import assert from 'node:assert/strict';

import 'fake-indexeddb/auto';

import { handle } from '../src/lib/local';

interface Entry {
  id: string;
  date: string;
  title: string;
  content: string;
  word_count: number;
  tags: { name: string }[];
}

async function call<T>(path: string, method: 'GET' | 'POST' | 'PATCH' | 'DELETE', body?: unknown) {
  return handle<T>({ path, method, body });
}

async function main() {
  const credentials = { email: 'rani@example.com', display_name: 'Rani', password: 'a long enough password' };

  const session = await call<{ user: { email: string } }>('/auth/register', 'POST', credentials);
  assert.equal(session.user.email, 'rani@example.com', 'registers an account');

  const duplicate = await call('/auth/register', 'POST', credentials).then(
    () => null,
    (error: Error) => error,
  );
  assert.ok(duplicate, 'refuses a duplicate email');

  const me = await call<{ user: { display_name: string } }>('/auth/session', 'GET');
  assert.equal(me.user.display_name, 'Rani', 'keeps the session');

  await call('/auth/logout', 'POST');
  const signedOut = await call('/auth/session', 'GET').then(
    () => null,
    (error: Error) => error,
  );
  assert.ok(signedOut, 'signs out');

  await call('/auth/login', 'POST', { email: credentials.email, password: 'wrong password' }).then(
    () => {
      throw new Error('a wrong password must be rejected');
    },
    () => undefined,
  );
  await call('/auth/login', 'POST', { email: credentials.email, password: credentials.password });

  const created = await call<Entry>('/entries', 'POST', {
    date: '2026-10-10',
    title: 'A quiet day',
    content: 'one two three four five',
    mood_id: null,
    tags: ['work'],
  });
  assert.equal(created.word_count, 5, 'counts words on write');
  assert.equal(created.tags[0]?.name, 'work', 'creates the tag');

  const byDate = await call<Entry | null>('/entries/by-date/2026-10-10', 'GET');
  assert.equal(byDate?.id, created.id, 'reads an entry by date');

  const updated = await call<Entry>(`/entries/${created.id}`, 'PATCH', { title: 'Edited' });
  assert.equal(updated.title, 'Edited', 'updates an entry');
  assert.equal(updated.word_count, 5, 'keeps the word count when the body is untouched');

  const page = await call<{ items: Entry[]; total: number }>('/entries?limit=10&offset=0', 'GET');
  assert.equal(page.total, 1, 'lists entries');
  assert.equal(page.items[0]?.excerpt, 'one two three four five', 'returns an excerpt');

  const tagged = await call<{ items: Entry[]; total: number }>('/entries?tag=work', 'GET');
  assert.equal(tagged.total, 1, 'filters by tag');
  const otherTag = await call<{ total: number }>('/entries?tag=unknown', 'GET');
  assert.equal(otherTag.total, 0, 'an unknown tag matches nothing');

  const calendar = await call<{ days: unknown[]; total_entries: number }>(
    '/calendar/month?year=2026&month=10',
    'GET',
  );
  assert.equal(calendar.days.length, 31, 'covers every day of the month');
  assert.equal(calendar.total_entries, 1, 'counts entries in the month');

  const insights = await call<{ total_entries: number; total_words: number; current_streak: number }>(
    '/insights',
    'GET',
  );
  assert.equal(insights.total_entries, 1, 'insights see the entry');
  assert.equal(insights.total_words, 5, 'insights sum the words');

  const found = await call<{ total: number; items: { snippet: string }[] }>('/search?q=quiet', 'GET');
  assert.equal(found.total, 0, 'search covers the body, not the title only');
  const byBody = await call<{ total: number; items: { snippet: string }[] }>('/search?q=three', 'GET');
  assert.equal(byBody.total, 1, 'finds text in the body');
  assert.ok(byBody.items[0]?.snippet.includes('<mark>'), 'marks the match');

  const archive = await call<{ manifest: { counts: { entries: number } }; entries: unknown[] }>(
    '/exports/archive',
    'GET',
  );
  assert.equal(archive.manifest.counts.entries, 1, 'exports the entry');

  const preferences = await call<{ theme: string }>('/settings/preferences', 'PATCH', {
    theme: 'dark',
  });
  assert.equal(preferences.theme, 'dark', 'stores preferences');

  await call(`/entries/${created.id}`, 'DELETE');
  const emptied = await call<{ total: number }>('/entries', 'GET');
  assert.equal(emptied.total, 0, 'deletes the entry');

  console.log('local backend check passed');
}

main().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});