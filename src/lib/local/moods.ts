/** The mood catalog. Ids match the ones the server migration seeds. */
export interface Mood {
  id: string;
  slug: string;
  label: string;
  icon: string;
  score: number;
  position: number;
}

export const MOODS: Mood[] = [
  { id: '11111111-1111-4111-8111-111111111111', slug: 'radiant', label: 'Radiant', icon: 'sun', score: 5, position: 1 },
  { id: '22222222-2222-4222-8222-222222222222', slug: 'happy', label: 'Happy', icon: 'smile', score: 4, position: 2 },
  { id: '33333333-3333-4333-8333-333333333333', slug: 'calm', label: 'Calm', icon: 'leaf', score: 3, position: 3 },
  { id: '44444444-4444-4444-8444-444444444444', slug: 'tired', label: 'Tired', icon: 'moon', score: 2, position: 4 },
  { id: '55555555-5555-4555-8555-555555555555', slug: 'anxious', label: 'Anxious', icon: 'wave', score: 2, position: 5 },
  { id: '66666666-6666-4666-8666-666666666666', slug: 'sad', label: 'Sad', icon: 'cloud', score: 1, position: 6 },
  { id: '77777777-7777-4777-8777-777777777777', slug: 'angry', label: 'Angry', icon: 'flame', score: 1, position: 7 },
];

/** Catalog in the shape the API returns. */
export function list() {
  return MOODS.map(({ id, slug, label, icon }) => ({ id, slug, label, icon }));
}

export interface MoodSummary {
  id: string;
  slug: string;
  label: string;
  icon: string;
}

export function moodById(id: string | null): MoodSummary | null {
  if (!id) {
    return null;
  }
  const mood = MOODS.find((candidate) => candidate.id === id);
  return mood
    ? { id: mood.id, slug: mood.slug, label: mood.label, icon: mood.icon }
    : null;
}

export function moodIdBySlug(slug: string | null | undefined): string | null {
  if (!slug) {
    return null;
  }
  return MOODS.find((mood) => mood.slug === slug)?.id ?? null;
}