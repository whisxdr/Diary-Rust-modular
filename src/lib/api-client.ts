/**
 * Single place that knows how to talk to the backend.
 *
 * Feature modules call `request`; none of them touch fetch, headers or cookies.
 *
 * `VITE_API_BASE_URL` points at the API when the frontend is hosted somewhere
 * other than the API itself. Left empty, requests go to `/api` on the same
 * origin, which is what the dev proxy and a reverse-proxied deployment use.
 */

const API_BASE = (import.meta.env['VITE_API_BASE_URL'] as string | undefined)?.replace(/\/$/, '') ?? '/api';
const CSRF_COOKIE = 'dmd_csrf';
const CSRF_HEADER = 'X-CSRF-Token';

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

/** Fetches a CSRF token once; the backend also sets it as a cookie. */
export function ensureCsrfToken(): Promise<void> {
  if (readCookie(CSRF_COOKIE)) {
    return Promise.resolve();
  }

  csrfReady ??= request<void>({ path: '/auth/csrf' }).then(() => undefined);
  return csrfReady;
}

export async function request<T>({ path, method = 'GET', body, signal }: RequestOptions & { path: string }): Promise<T> {
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
    throw new ApiError(
      response.status,
      stringField(payload, 'code') ?? 'error',
      stringField(payload, 'message') ?? 'Request failed.',
    );
  }

  return payload as T;
}

/** Uploads a file as multipart, used by attachments and import. */
export async function upload<T>(path: string, form: FormData, signal?: AbortSignal): Promise<T> {
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
    throw new ApiError(
      response.status,
      stringField(payload, 'code') ?? 'error',
      stringField(payload, 'message') ?? 'Upload failed.',
    );
  }

  return payload as T;
}

/** Downloads a file, keeping the session cookie attached. */
export async function download(path: string, fileName: string): Promise<void> {
  const response = await fetch(`${API_BASE}${path}`, { credentials: 'include' });
  if (!response.ok) {
    throw new ApiError(response.status, 'download_failed', 'The file could not be downloaded.');
  }

  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  link.click();
  URL.revokeObjectURL(url);
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