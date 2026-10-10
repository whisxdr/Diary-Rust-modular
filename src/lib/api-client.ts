/**
 * Single place that knows where the data lives.
 *
 * Feature modules call `request`; none of them touch fetch, the database, or
 * cookies.
 *
 * - `VITE_API_BASE_URL` set: requests go over HTTP to that API.
 * - `VITE_API_BASE_URL` empty: the app runs entirely on this device and the
 *   same URLs are answered from IndexedDB by `lib/local`. That is what makes a
 *   static deployment on a CDN usable with no server at all.
 */

import * as local from './local';
import { LocalError } from './local/accounts';
import { buildArchive as localBuildArchive, fileName as localFileName } from './local/backup';

const CONFIGURED_API = (import.meta.env['VITE_API_BASE_URL'] as string | undefined)?.trim();
const API_BASE = CONFIGURED_API ? CONFIGURED_API.replace(/\/$/, '') : '/api';
const CSRF_COOKIE = 'dmd_csrf';
const CSRF_HEADER = 'X-CSRF-Token';

/** True when no API was configured, so everything stays in the browser. */
export const isLocalOnly = !CONFIGURED_API;

export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = 'ApiError';
  }

  get isUnauthorized(): boolean {
    return this.status === 401;
  }

  get isValidation(): boolean {
    return this.status === 422;
  }
}

type Method = 'GET' | 'POST' | 'PATCH' | 'DELETE';

export interface RequestOptions {
  method?: Method | undefined;
  body?: unknown;
  signal?: AbortSignal | undefined;
}

function readCookie(name: string): string | null {
  const match = document.cookie
    .split(';')
    .map((pair) => pair.trim())
    .find((pair) => pair.startsWith(`${name}=`));

  return match ? decodeURIComponent(match.slice(name.length + 1)) : null;
}

let csrfReady: Promise<void> | null = null;

/** Fetches a CSRF token once; the server also sets it as a cookie. */
export function ensureCsrfToken(): Promise<void> {
  if (isLocalOnly || readCookie(CSRF_COOKIE)) {
    return Promise.resolve();
  }

  csrfReady ??= request<void>({ path: '/auth/csrf' }).then(() => undefined);
  return csrfReady;
}

export async function request<T>({
  path,
  method = 'GET',
  body,
  signal,
}: RequestOptions & { path: string }): Promise<T> {
  if (isLocalOnly) {
    return localCall<T>({ path, method, body });
  }

  if (method !== 'GET') {
    await ensureCsrfToken();
  }

  const headers: Record<string, string> = {};
  if (body !== undefined) {
    headers['Content-Type'] = 'application/json';
  }
  if (method !== 'GET') {
    const token = readCookie(CSRF_COOKIE);
    if (token) {
      headers[CSRF_HEADER] = token;
    }
  }

  const response = await fetch(`${API_BASE}${path}`, {
    method,
    headers,
    credentials: 'include',
    body: body === undefined ? null : JSON.stringify(body),
    signal: signal ?? null,
  });

  if (response.status === 204) {
    return undefined as T;
  }

  const payload = await readPayload(response);

  if (!response.ok) {
    throw toApiError(response.status, payload, 'Request failed.');
  }

  return payload as T;
}

/** Uploads a file as multipart. In local mode the file is read in the browser. */
export async function upload<T>(path: string, form: FormData, signal?: AbortSignal): Promise<T> {
  if (isLocalOnly) {
    const file = form.get('file');
    return localCall<T>({
      path,
      method: 'POST',
      file: file instanceof File ? file : undefined,
    });
  }

  await ensureCsrfToken();

  const headers: Record<string, string> = {};
  const token = readCookie(CSRF_COOKIE);
  if (token) {
    headers[CSRF_HEADER] = token;
  }

  const response = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers,
    credentials: 'include',
    body: form,
    signal: signal ?? null,
  });

  const payload = await readPayload(response);
  if (!response.ok) {
    throw toApiError(response.status, payload, 'Upload failed.');
  }

  return payload as T;
}

/** Saves a file. In local mode the archive is built in the browser. */
export async function download(path: string, fileName: string): Promise<void> {
  if (isLocalOnly) {
    await saveArchive(fileName);
    return;
  }

  const response = await fetch(`${API_BASE}${path}`, { credentials: 'include' });
  if (!response.ok) {
    throw new ApiError(response.status, 'download_failed', 'The file could not be downloaded.');
  }

  saveBlob(await response.blob(), fileName);
}

async function localCall<T>(request: local.LocalRequest): Promise<T> {
  try {
    return await local.handle<T>(request);
  } catch (cause) {
    if (cause instanceof LocalError) {
      throw new ApiError(cause.status, cause.code, cause.message);
    }
    throw cause;
  }
}

async function saveArchive(fileName: string): Promise<void> {
  const archive = await localBuildArchive();
  const json = JSON.stringify(archive, null, 2);

  saveBlob(
    new Blob([json], { type: 'application/json' }),
    fileName === 'dear-my-diary-backup.json' ? localFileName(archive) : fileName,
  );
}

function saveBlob(blob: Blob, fileName: string): void {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  link.click();
  URL.revokeObjectURL(url);
}

function toApiError(status: number, payload: Record<string, unknown>, fallback: string): ApiError {
  return new ApiError(status, stringField(payload, 'code') ?? 'error', stringField(payload, 'message') ?? fallback);
}

async function readPayload(response: Response): Promise<Record<string, unknown>> {
  const text = await response.text();
  if (!text) {
    return {};
  }

  try {
    return JSON.parse(text) as Record<string, unknown>;
  } catch {
    return { message: text };
  }
}

function stringField(payload: Record<string, unknown>, key: string): string | undefined {
  const value = payload[key];
  return typeof value === 'string' ? value : undefined;
}