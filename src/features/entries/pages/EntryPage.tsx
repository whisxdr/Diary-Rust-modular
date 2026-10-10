import { useParams } from 'react-router-dom';

import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Spinner } from '@/components/feedback/Spinner';
import { useAsyncData } from '@/hooks/useAsyncData';
import { formatLongDate } from '@/lib/date';

import { entriesApi } from '../api';
import { EntryEditor } from '../components/EntryEditor';

export function EntryPage() {
  const { id = '' } = useParams();
  const entry = useAsyncData((signal) => entriesApi.byId(id, signal), [id]);

  if (entry.isLoading) {
    return <Spinner />;
  }
  if (entry.error || !entry.data) {
    return <ErrorMessage message={entry.error ?? 'Entry not found.'} />;
  }

  return (
    <div className="page">
      <header className="page__header">
        <h1>{entry.data.title || 'Untitled entry'}</h1>
        <p className="hint">{formatLongDate(entry.data.date)}</p>
      </header>

      <EntryEditor entry={entry.data} onSaved={() => entry.reload()} />
    </div>
  );
}