/** Small formatting helpers shared by several features. */

export function formatWordCount(count: number): string {
  return count === 1 ? '1 word' : `${count} words`;
}

export function formatRelativeDate(isoDate: string): string {
  const target = new Date(isoDate).getTime();
  if (Number.isNaN(target)) {
    return '';
  }

  const diffMinutes = Math.round((Date.now() - target) / 60_000);
  if (diffMinutes < 1) {
    return 'just now';
  }
  if (diffMinutes < 60) {
    return `${diffMinutes} min ago`;
  }

  const diffHours = Math.round(diffMinutes / 60);
  if (diffHours < 24) {
    return `${diffHours} h ago`;
  }

  const diffDays = Math.round(diffHours / 24);
  if (diffDays < 7) {
    return `${diffDays} d ago`;
  }

  return new Date(isoDate).toLocaleDateString();
}

export function percent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

export function pluralize(count: number, singular: string, plural = `${singular}s`): string {
  return `${count} ${count === 1 ? singular : plural}`;
}