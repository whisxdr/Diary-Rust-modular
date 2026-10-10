import { formatShortDate } from '@/lib/date';
import { formatRelativeDate, formatWordCount } from '@/lib/formatting';
import { Link } from 'react-router-dom';

import type { EntrySummary } from '../types';

export function EntryCard({ entry }: { entry: EntrySummary }) {
  return (
    <article className="entry-card">
      <header>
        <Link to={`/entries/${entry.id}`}>{entry.title || 'Untitled entry'}</Link>
        <time dateTime={entry.date}>{formatShortDate(entry.date)}</time>
      </header>

      {entry.excerpt ? <p>{entry.excerpt}</p> : <p className="hint">No text yet.</p>}

      <footer>
        <span className="hint">{formatWordCount(entry.word_count)}</span>
        {entry.status === 'draft' ? <span className="badge">draft</span> : null}
        <span className="hint">{formatRelativeDate(entry.updated_at)}</span>
      </footer>
    </article>
  );
}