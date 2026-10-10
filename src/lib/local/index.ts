/**
 * Path router for the browser-local backend.
 *
 * It answers the same URLs the API answers, so no feature module changes: the
 * app works against IndexedDB when no API URL is configured and against the
 * server otherwise.
 *
 * Each module owns the query string it needs, so this file only matches the
 * path segments.
 */
import * as accounts from './accounts';
export { buildArchive, fileName } from './backup';
import * as backup from './backup';
import * as calendar from './calendar';
import * as entries from './entries';
import * as insights from './insights';
import * as moods from './moods';
import * as search from './search';
import * as settings from './settings';
import * as tags from './tags';

type Method = 'GET' | 'POST' | 'PATCH' | 'DELETE';

export interface LocalRequest {
  path: string;
  method: Method;
  body?: unknown;
  file?: File | undefined;
}

export interface Route {
  segments: string[];
  params: URLSearchParams;
}

export function route(path: string): Route {
  const parts = path.split('?');
  const pathname = parts[0] ?? '';

  return {
    segments: pathname.split('/').filter(Boolean).map(decodeURIComponent),
    params: new URLSearchParams(parts[1] ?? ''),
  };
}

function matches(routed: Route, ...expected: string[]): boolean {
  return (
    routed.segments.length === expected.length &&
    expected.every((segment, index) => segment === '*' || segment === routed.segments[index])
  );
}

function body<T>(request: LocalRequest): T {
  return request.body as T;
}

export async function handle<T>(request: LocalRequest): Promise<T> {
  const routed = route(request.path);
  const { method } = request;
  const segments = routed.segments;

  if (matches(routed, 'auth', 'register') && method === 'POST') {
    const created = await accounts.register(body<Parameters<typeof accounts.register>[0]>(request));
    return { user: accounts.toProfile(created.user), expires_at: created.expires_at } as T;
  }
  if (matches(routed, 'auth', 'login') && method === 'POST') {
    const signedIn = await accounts.login(body<Parameters<typeof accounts.login>[0]>(request));
    return { user: accounts.toProfile(signedIn.user), expires_at: signedIn.expires_at } as T;
  }
  if (matches(routed, 'auth', 'logout') && method === 'POST') {
    await accounts.logout();
    return { message: 'Signed out.' } as T;
  }
  if (matches(routed, 'auth', 'session') && method === 'GET') {
    return { user: accounts.toProfile(await accounts.requireUser()), expires_at: '' } as T;
  }
  if (matches(routed, 'auth', 'password') && method === 'POST') {
    return accounts.changePassword(body<Parameters<typeof accounts.changePassword>[0]>(request)) as T;
  }

  if (matches(routed, 'account') && method === 'GET') {
    return accounts.toProfile(await accounts.requireUser()) as T;
  }
  if (matches(routed, 'account') && method === 'PATCH') {
    const updated = await accounts.updateProfile(body<{ display_name: string }>(request));
    return accounts.toProfile(updated) as T;
  }
  if (matches(routed, 'account') && method === 'DELETE') {
    return accounts.deleteAccount() as T;
  }

  if (matches(routed, 'settings', 'preferences') && method === 'GET') {
    return settings.get() as T;
  }
  if (matches(routed, 'settings', 'preferences') && method === 'PATCH') {
    return settings.update(body<Parameters<typeof settings.update>[0]>(request)) as T;
  }

  if (matches(routed, 'moods') && method === 'GET') {
    return moods.list() as T;
  }

  if (matches(routed, 'tags') && method === 'GET') {
    return tags.listForUser(await accounts.requireUser()) as T;
  }
  if (matches(routed, 'tags') && method === 'POST') {
    const user = await accounts.requireUser();
    return tags.createForUser(user.id, body<{ name: string }>(request).name) as T;
  }
  if (matches(routed, 'tags', '*') && method === 'PATCH') {
    const user = await accounts.requireUser();
    return tags.renameForUser(user.id, segments[1] ?? '', body<{ name: string }>(request).name) as T;
  }
  if (matches(routed, 'tags', '*') && method === 'DELETE') {
    const user = await accounts.requireUser();
    await tags.removeForUser(user.id, segments[1] ?? '');
    return undefined as T;
  }

  if (matches(routed, 'entries') && method === 'GET') {
    return entries.listFromParams((await accounts.requireUser()).id, routed.params) as T;
  }
  if (matches(routed, 'entries') && method === 'POST') {
    return entries.create(body<Parameters<typeof entries.create>[0]>(request)) as T;
  }
  if (matches(routed, 'entries', 'by-date', '*') && method === 'GET') {
    return entries.byDate(segments[2] ?? '') as T;
  }
  if (matches(routed, 'entries', '*', 'autosave') && method === 'PATCH') {
    return entries.update(segments[1] ?? '', body<{ title?: string; content?: string }>(request)) as T;
  }
  if (matches(routed, 'entries', '*', 'status') && method === 'PATCH') {
    return entries.setStatus(segments[1] ?? '', body<{ status: string }>(request).status) as T;
  }
  if (matches(routed, 'entries', '*') && method === 'GET') {
    return entries.byId(segments[1] ?? '') as T;
  }
  if (matches(routed, 'entries', '*') && method === 'PATCH') {
    return entries.update(segments[1] ?? '', body<never>(request)) as T;
  }
  if (matches(routed, 'entries', '*') && method === 'DELETE') {
    await entries.remove(segments[1] ?? '');
    return undefined as T;
  }

  if (matches(routed, 'calendar', 'month') && method === 'GET') {
    return calendar.monthFromParams(routed.params) as T;
  }
  if (matches(routed, 'calendar', 'years') && method === 'GET') {
    return calendar.years() as T;
  }

  if (matches(routed, 'search') && method === 'GET') {
    return search.queryFromParams(routed.params) as T;
  }
  if (matches(routed, 'insights') && method === 'GET') {
    return insights.overview() as T;
  }

  if (matches(routed, 'exports', 'archive') && method === 'GET') {
    return backup.buildArchive() as T;
  }
  if (matches(routed, 'imports') && method === 'POST') {
    if (!request.file) {
      throw new accounts.LocalError(422, 'validation_error', 'file: is required');
    }
    return backup.stageUpload(request.file.name, await request.file.text()) as T;
  }
  if (matches(routed, 'imports', 'history') && method === 'GET') {
    return backup.history() as T;
  }
  if (matches(routed, 'imports', '*', 'preview') && method === 'GET') {
    return backup.preview(segments[1] ?? '') as T;
  }
  if (matches(routed, 'imports', 'commit') && method === 'POST') {
    return backup.commit(body<Parameters<typeof backup.commit>[0]>(request)) as T;
  }

  throw new accounts.LocalError(404, 'not_found', 'Unknown endpoint.');
}