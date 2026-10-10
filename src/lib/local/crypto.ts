/**
 * Password hashing and tokens, using Web Crypto.
 *
 * The server hashes with Argon2id; a browser has no Argon2, so local accounts
 * use PBKDF2-SHA256 with a per-account salt.
 */

const ITERATIONS = 210_000;
const KEY_BITS = 256;

const encoder = new TextEncoder();

function toHex(bytes: ArrayBuffer | Uint8Array): string {
  const view = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  return Array.from(view, (byte) => byte.toString(16).padStart(2, '0')).join('');
}

export function randomToken(): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return toHex(bytes);
}

function toBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

async function derive(password: string, saltHex: string): Promise<string> {
  const salt = new Uint8Array(
    (saltHex.match(/.{2}/g) ?? []).map((byte) => Number.parseInt(byte, 16)),
  );

  const key = await crypto.subtle.importKey('raw', encoder.encode(password), 'PBKDF2', false, [
    'deriveBits',
  ]);

  const bits = await crypto.subtle.deriveBits(
    { name: 'PBKDF2', salt, iterations: ITERATIONS, hash: 'SHA-256' },
    key,
    KEY_BITS,
  );

  return toBase64(bits);
}

export async function hashPassword(password: string): Promise<string> {
  const saltHex = toHex(crypto.getRandomValues(new Uint8Array(16)));
  return `pbkdf2$${saltHex}$${await derive(password, saltHex)}`;
}

export async function verifyPassword(password: string, stored: string): Promise<boolean> {
  const [scheme, saltHex, expected] = stored.split('$');
  if (scheme !== 'pbkdf2' || !saltHex || !expected) {
    return false;
  }

  const actual = await derive(password, saltHex);

  // Compare every character so a mismatch costs the same everywhere.
  if (actual.length !== expected.length) {
    return false;
  }
  let difference = 0;
  for (let index = 0; index < actual.length; index += 1) {
    difference |= actual.charCodeAt(index) ^ expected.charCodeAt(index);
  }
  return difference === 0;
}