/**
 * Minimal IndexedDB access layer.
 *
 * The local backend keeps every row in the browser; this file is the only
 * place that talks to the database API.
 */

const DB_NAME = 'dear-my-diary';
const DB_VERSION = 1;

export const STORES = {
  users: 'users',
  entries: 'entries',
  tags: 'tags',
  meta: 'meta',
} as const;

export type StoreName = (typeof STORES)[keyof typeof STORES];

let connection: Promise<IDBDatabase> | null = null;

function open(): Promise<IDBDatabase> {
  connection ??= new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onupgradeneeded = () => {
      const db = request.result;

      if (!db.objectStoreNames.contains(STORES.users)) {
        const users = db.createObjectStore(STORES.users, { keyPath: 'id' });
        users.createIndex('byEmail', 'email', { unique: true });
      }

      if (!db.objectStoreNames.contains(STORES.entries)) {
        const entries = db.createObjectStore(STORES.entries, { keyPath: 'id' });
        entries.createIndex('byUser', 'user_id');
        entries.createIndex('byUserDate', ['user_id', 'date'], { unique: true });
      }

      if (!db.objectStoreNames.contains(STORES.tags)) {
        const tags = db.createObjectStore(STORES.tags, { keyPath: 'id' });
        tags.createIndex('byUser', 'user_id');
        tags.createIndex('byNormalized', ['user_id', 'normalized'], { unique: true });
      }

      if (!db.objectStoreNames.contains(STORES.meta)) {
        db.createObjectStore(STORES.meta, { keyPath: 'key' });
      }
    };

    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error('Cannot open the local database.'));
  });

  return connection;
}

function run<T>(
  store: StoreName,
  mode: IDBTransactionMode,
  work: (objectStore: IDBObjectStore) => IDBRequest<T>,
): Promise<T> {
  return open().then(
    (db) =>
      new Promise<T>((resolve, reject) => {
        const request = work(db.transaction(store, mode).objectStore(store));
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error ?? new Error('Local database request failed.'));
      }),
  );
}

export function get<T>(store: StoreName, key: IDBValidKey): Promise<T | undefined> {
  return run<T | undefined>(store, 'readonly', (objectStore) => objectStore.get(key));
}

export function getAll<T>(store: StoreName): Promise<T[]> {
  return run<T[]>(store, 'readonly', (objectStore) => objectStore.getAll());
}

/** First row matching an index value, or undefined. */
export async function findByIndex<T>(
  store: StoreName,
  index: string,
  query: IDBValidKey | IDBKeyRange,
): Promise<T | undefined> {
  const [row] = await getByIndex<T>(store, index, query);
  return row;
}

export function getByIndex<T>(
  store: StoreName,
  index: string,
  query: IDBValidKey | IDBKeyRange,
): Promise<T[]> {
  return run<T[]>(store, 'readonly', (objectStore) => objectStore.index(index).getAll(query));
}

export function put<T>(store: StoreName, value: T): Promise<IDBValidKey> {
  return run<IDBValidKey>(store, 'readwrite', (objectStore) => objectStore.put(value));
}

export function remove(store: StoreName, key: IDBValidKey): Promise<undefined> {
  return run<undefined>(store, 'readwrite', (objectStore) => objectStore.delete(key));
}

export function clearStore(store: StoreName): Promise<undefined> {
  return run<undefined>(store, 'readwrite', (objectStore) => objectStore.clear());
}

/** Single-value settings: session, current user id, preferences, staged imports. */
export async function getMeta<T>(key: string): Promise<T | undefined> {
  const row = await get<{ key: string; value: T }>(STORES.meta, key);
  return row?.value;
}

export function setMeta<T>(key: string, value: T): Promise<IDBValidKey> {
  return put(STORES.meta, { key, value });
}