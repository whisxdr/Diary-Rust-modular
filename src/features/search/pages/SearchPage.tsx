import { useState } from 'react';
import { Link } from 'react-router-dom';

import { EmptyState } from '@/components/feedback/EmptyState';
import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Spinner } from '@/components/feedback/Spinner';
import { Button } from '@/components/ui/Button';
import { useAsyncData } from '@/hooks/useAsyncData';
import { formatShortDate } from '@/lib/date';

import { searchApi } from '../api';

export function SearchPage() {
  const [term, setTerm] = useState('');
  const [submitted, setSubmitted] = useState('');

  const results = useAsyncData((signal) => searchApi.query(submitted, signal), [submitted]);

  return (
    <div className="page">
      <header className="page__header">
        <h1>Search</h1>
      </header>

      <form
        className="filters"
        onSubmit={(event) => {
          event.preventDefault();
          setSubmitted(term.trim());
        }}
      >
        <input
          value={term}
          placeholder="Search titles and text"
          onChange={(event) => setTerm(event.target.value)}
        />
        <Button type="submit" variant="primary">
          Search
        </Button>
      </form>

      {results.isLoading && submitted ? <Spinner /> : null}
      {results.error ? <ErrorMessage message={results.error} /> : null}

      {submitted && !results.isLoading && (results.data?.items.length ?? 0) === 0 ? (
        <EmptyState title="No matches" description={`Nothing matches “${submitted}”.`} />
      ) : null}

      <ul className="results">
        {results.data?.items.map((hit) => (
          <li key={hit.entry_id}>
            <Link to={`/entries/${hit.entry_id}`}>{hit.title || 'Untitled entry'}</Link>
            <time className="hint" dateTime={hit.date}>
              {formatShortDate(hit.date)}
            </time>
            <p dangerouslySetInnerHTML={{ __html: hit.snippet }} />
          </li>
        ))}
      </ul>
    </div>
  );
}