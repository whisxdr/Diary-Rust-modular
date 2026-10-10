import { useState } from 'react';

import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { messageOf } from '@/hooks/useAsyncData';

import { exportsApi } from '../api';

export function ExportPage() {
  const [error, setError] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  async function downloadArchive() {
    setIsBusy(true);
    setError(null);
    try {
      await exportsApi.archive();
    } catch (cause) {
      setError(messageOf(cause));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <div className="page">
      <header className="page__header">
        <h1>Export a backup</h1>
        <p className="hint">One JSON file with your entries, tags and moods.</p>
      </header>

      <Card title="Download">
        <p>Attachments stay on this device; the archive records what each entry had.</p>
        <Button variant="primary" disabled={isBusy} onClick={() => void downloadArchive()}>
          {isBusy ? 'Preparing…' : 'Download backup'}
        </Button>
        {error ? <ErrorMessage message={error} /> : null}
      </Card>
    </div>
  );
}