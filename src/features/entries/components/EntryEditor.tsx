import { useEffect, useState } from 'react';

import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Button } from '@/components/ui/Button';
import { MoodPicker } from '@/features/moods/components/MoodPicker';
import { TagPicker } from '@/features/tags/components/TagPicker';
import { formatWordCount } from '@/lib/formatting';
import { messageOf } from '@/hooks/useAsyncData';

import { entriesApi } from '../api';
import { useAutosave } from '../hooks/useAutosave';
import type { Entry } from '../types';
import { AutosaveStatus } from './AutosaveStatus';

interface EntryEditorProps {
  entry: Entry | null;
  onSaved: (entry: Entry) => void;
  onDeleted?: () => void;
}

/**
 * Title, body, mood and tags for one entry.
 *
 * The component owns the draft text; saving and autosave are delegated.
 */
export function EntryEditor({ entry, onSaved, onDeleted }: EntryEditorProps) {
  const [title, setTitle] = useState(entry?.title ?? '');
  const [content, setContent] = useState(entry?.content ?? '');
  const [moodId, setMoodId] = useState<string | null>(entry?.mood?.id ?? null);
  const [tags, setTags] = useState<string[]>(entry?.tags.map((tag) => tag.name) ?? []);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  const autosave = useAutosave(entry);

  // A different entry is loaded into the same component, so reset the draft.
  useEffect(() => {
    setTitle(entry?.title ?? '');
    setContent(entry?.content ?? '');
    setMoodId(entry?.mood?.id ?? null);
    setTags(entry?.tags.map((tag) => tag.name) ?? []);
  }, [entry?.id, entry?.title, entry?.content, entry?.mood?.id, entry?.tags]);

  async function save() {
    if (!entry) {
      return;
    }

    autosave.cancel();
    setIsSaving(true);
    setError(null);

    try {
      const saved = await entriesApi.update(entry.id, { title, content, mood_id: moodId, tags });
      onSaved(saved);
    } catch (cause) {
      setError(messageOf(cause));
    } finally {
      setIsSaving(false);
    }
  }

  async function remove() {
    if (!entry || !window.confirm('Delete this entry? This cannot be undone.')) {
      return;
    }

    try {
      await entriesApi.remove(entry.id);
      onDeleted?.();
    } catch (cause) {
      setError(messageOf(cause));
    }
  }

  const wordCount = content.split(/\s+/).filter(Boolean).length;

  return (
    <div className="editor">
      <input
        className="editor__title"
        placeholder="Title"
        value={title}
        onChange={(event) => {
          setTitle(event.target.value);
          autosave.schedule();
        }}
      />

      <textarea
        className="editor__body"
        placeholder="What happened today?"
        value={content}
        onChange={(event) => {
          setContent(event.target.value);
          autosave.schedule();
        }}
      />

      <div className="editor__meta">
        <span className="hint">{formatWordCount(wordCount)}</span>
        <AutosaveStatus state={autosave.state} error={autosave.error} />
      </div>

      <MoodPicker selected={moodId} onChange={setMoodId} />
      <TagPicker selected={tags} onChange={setTags} />

      {error ? <ErrorMessage message={error} /> : null}

      <div className="editor__actions">
        <Button variant="primary" onClick={save} disabled={isSaving || !entry}>
          {isSaving ? 'Saving…' : 'Save'}
        </Button>
        {onDeleted ? (
          <Button variant="danger" onClick={remove}>
            Delete
          </Button>
        ) : null}
      </div>
    </div>
  );
}