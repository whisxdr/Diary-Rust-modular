import { useState } from 'react';

import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { messageOf } from '@/hooks/useAsyncData';
import { pluralize } from '@/lib/formatting';

import { importsApi } from '../api';
import type { ConflictStrategy, ImportPreview, ImportSummary } from '../api';

/** Upload, review, then apply. Nothing is written before the review step. */
export function ImportPage() {
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [summary, setSummary] = useState<ImportSummary | null>(null);
  const [strategy, setStrategy] = useState<ConflictStrategy>('merge');
  const [error, setError] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  async function stage(file: File) {
    setIsBusy(true);
    setError(null);
    setSummary(null);

    try {
      setPreview(await importsApi.upload(file));
    } catch (cause) {
      setPreview(null);
      setError(messageOf(cause));
    } finally {
      setIsBusy(false);
    }
  }

  async function commit() {
    if (!preview?.job_id) {
      return;
    }

    setIsBusy(true);
    setError(null);
    try {
      setSummary(await importsApi.commit({ job_id: preview.job_id, conflict_strategy: strategy }));
    } catch (cause) {
      setError(messageOf(cause));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <div className="page">
      <header className="page__header">
        <h1>Import a backup</h1>
        <p className="hint">Your credentials are never part of a backup file.</p>
      </header>

      <Card title="Choose a file">
        <input
          type="file"
          accept="application/json,.json"
          onChange={(event) => {
            const file = event.target.files?.[0];
            if (file) {
              void stage(file);
            }
          }}
        />
        {isBusy ? <p className="hint">Working…</p> : null}
        {error ? <ErrorMessage message={error} /> : null}
      </Card>

      {preview ? (
        <Card title="What will happen">
          <ul>
            <li>{pluralize(preview.entries_new, 'new entry', 'new entries')}</li>
            <li>{pluralize(preview.entries_updated, 'entry on a day you already wrote')}</li>
            <li>{pluralize(preview.tags_new, 'tag')}</li>
            {preview.date_range ? (
              <li>
                From {preview.date_range.first} to {preview.date_range.last}
              </li>
            ) : null}
          </ul>

          {preview.problems.length > 0 ? (
            <details>
              <summary>{pluralize(preview.problems.length, 'problem')}</summary>
              <ul>
                {preview.problems.map((problem) => (
                  <li key={problem.entry_key}>{problem.message}</li>
                ))}
              </ul>
            </details>
          ) : null}

          <label className="field">
            <span className="field__label">When a day already exists</span>
            <select
              value={strategy}
              onChange={(event) => setStrategy(event.target.value as ConflictStrategy)}
            >
              <option value="merge">Replace with the backup</option>
              <option value="skip">Keep what is here</option>
            </select>
          </label>

          <Button variant="primary" disabled={isBusy} onClick={() => void commit()}>
            Apply import
          </Button>
        </Card>
      ) : null}

      {summary ? (
        <Card title="Import finished">
          <p>
            {summary.entries_created} added, {summary.entries_updated} updated, {summary.entries_skipped}{' '}
            skipped.
          </p>
        </Card>
      ) : null}
    </div>
  );
}