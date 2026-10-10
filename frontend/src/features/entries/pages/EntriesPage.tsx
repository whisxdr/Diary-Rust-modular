import { useMemo, useState } from 'react';
import { Link } from 'react-router-dom';

import { EmptyState } from '@/components/feedback/EmptyState';
import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Spinner } from '@/components/feedback/Spinner';
import { Button } from '@/components/ui/Button';
import { useAsyncData } from '@/hooks/useAsyncData';
import { todayIso } from '@/lib/date';

import { entriesApi } from '../api';
import type { EntryFilters, EntryStatus } from '../types';
import { EntryCard } from '../components/EntryCard';
import { EntryEditor } from '../components/EntryEditor';

export function EntriesPage() {
  const [filters, setFilters] = useState<EntryFilters>({ limit: 20 });
  const [draftDate, setDraftDate] = useState(todayIso());

  const list = useAsyncData((signal) => entriesApi.list(filters, signal), [
    filters.limit,
    filters.offset,
    filters.tag,
    filters.status,
    filters.from,
    filters.to,
  ]);

  const offset = filters.offset ?? 0;
  const total = list.data?.total ?? 0;
  const visible = list.data?.items ?? [];

  const day = useAsyncData(
    (signal) => entriesApi.byDate(draftDate, signal),
    [draftDate],
  );

  function shiftFilter(patch: Partial<EntryFilters>) {
    setFilters((current) => ({ ...current, offset: 0, ...patch }));
  }

  const summary = useMemo(
    () => (total === 0 ? 'No entries yet' : `${visible.length} of ${total} entries`),
    [visible.length, total],
  );

  return (
    <div className="page">
      <header className="page__header">
        <h1>Diary</h1>
        <p className="hint">{summary}</p>
      </header>

      <section className="filters">
        <input
          type="date"
          value={draftDate}
          onChange={(event) => setDraftDate(event.target.value)}
        />
        <input
          placeholder="Filter by tag"
          value={filters.tag ?? ''}
          onChange={(event) => shiftFilter({ tag: event.target.value })}
        />
        <select
          value={filters.status ?? ''}
          onChange={(event) =>
            shiftFilter({ status: (event.target.value || undefined) as EntryStatus | undefined })
          }
        >
          <option value="">All entries</option>
          <option value="published">Published</option>
          <option value="draft">Drafts</option>
        </select>
      </section>

      {list.isLoading ? <Spinner /> : null}
      {list.error ? <ErrorMessage message={list.error} /> : null}

      {!list.isLoading && visible.length === 0 ? (
        <EmptyState
          title="Nothing written yet"
          description="Pick a day and write the first entry."
        />
      ) : null}

      <div className="entry-list">
        {visible.map((entry) => (
          <EntryCard key={entry.id} entry={entry} />
        ))}
      </div>

      <nav className="pager">
        <Button disabled={offset === 0} onClick={() => shiftFilter({ offset: Math.max(0, offset - 20) })}>
          Newer
        </Button>
        <Button
          disabled={offset + 20 >= total}
          onClick={() => shiftFilter({ offset: offset + 20 })}
        >
          Older
        </Button>
      </nav>

      <section className="day">
        <h2>{draftDate}</h2>
        {day.isLoading ? <Spinner label="Loading day" /> : null}
        {day.data ? (
          <EntryEditor
            entry={day.data}
            onSaved={() => {
              day.reload();
              list.reload();
            }}
            onDeleted={() => {
              day.reload();
              list.reload();
            }}
          />
        ) : null}
        {!day.isLoading && day.data === null ? (
          <EmptyState
            title="This day is empty"
            description="Create an entry to start writing."
            action={
              <Button
                variant="primary"
                onClick={async () => {
                  const created = await entriesApi.create({
                    date: draftDate,
                    title: '',
                    content: '',
                    mood_id: null,
                    tags: [],
                  });
                  day.reload();
                  list.reload();
                  void created;
                }}
              >
                Start writing
              </Button>
            }
          />
        ) : null}
      </section>

      <p className="hint">
        Looking for one entry? <Link to="/search">Search the whole diary</Link>.
      </p>
    </div>
  );
}