/**
 * Text helpers that must match the server so the two produce identical results.
 * Mirrors `entries::models` in the backend.
 */

export function wordCount(content: string): number {
  return content.split(/\s+/).filter(Boolean).length;
}

export function excerpt(content: string): string {
  const words = content.split(/\s+/).filter(Boolean);
  const joined = words.join(' ');
  return words.length > 28 ? `${joined.slice(0, joined.length - 1)}…` : joined;
}

export function isoNow(): string {
  return new Date().toISOString();
}

export function isoIn(days: number): string {
  return new Date(Date.now() + days * 86_400_000).toISOString();
}