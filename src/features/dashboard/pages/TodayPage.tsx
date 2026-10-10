import { useState } from 'react';
import { Link } from 'react-router-dom';

import { EmptyState } from '@/components/feedback/EmptyState';
import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Spinner } from '@/components/feedback/Spinner';
import { Button } from '@/components/ui/Button';
import { useAsyncData } from '@/hooks/useAsyncData';
import { formatLongDate, todayIso } from '@/lib/date';
import { pluralize } from '@/lib/formatting';
import { entriesApi } from '@/features/entries/api';
import { EntryEditor } from '@/features/entries/components/EntryEditor';
import { insightsApi } from '@/features/insights/api';

/** The landing page: write today's entry, see how the streak is going. */
export function TodayPage() {
  const [date, setDate] = useState(todayIso());

  const entry = useAsyncData((signal) => entriesApi.byDate(date, signal), [date]);
  const insights = useAsyncData((signal) => insightsApi.overview(signal), []);

  async function startEntry() {
    await entriesApi.create({ date, title: '', content: '', mood_id: null, tags: [] });
    entry.reload();
  }

  return (
    <div className="page">
      <header className="page__header">
        <h1>{formatLongDate(date)}</h1>
        <input type="date" value={date} onChange={(event) => setDate(event.target.value)} />
      </header>

      {entry.isLoading ? <Spinner /> : null}
      {entry.error ? <ErrorMessage message={entry.error} /> : null}

      {entry.data ? (
        <EntryEditor entry={entry.data} onSaved={() => entry.reload()} />
      ) : null}

      {!entry.isLoading && entry.data === null ? (
        <EmptyState
          title="Nothing written for this day"
          description="A few lines are enough."
          action={
            <Button variant="primary" onClick={() => void startEntry()}>
              Start writing
            </Button>
          }
        />
      ) : null}

      {insights.data ? (
        <section className="today__stats">
          <p>
            {pluralize(insights.data.total_entries, 'entry', 'entries')} ·{' '}
            {pluralize(insights.data.current_streak, 'day')} in a row
          </p>
          <Link to="/insights">See your reflection</Link>
        </section>
      ) : null}
    </div>
  );
}