import { useCallback, useEffect, useRef, useState } from 'react';

import { entriesApi } from '../api';
import type { Entry } from '../types';

export type AutosaveState = 'idle' | 'saving' | 'saved' | 'failed';

/**
 * Debounced autosave for the editor.
 *
 * ponytail: one timer per editor. A shared autosave service would only pay off
 * with several editors on screen at once.
 */
export function useAutosave(entry: Entry | null, delayMs = 1200) {
  const [state, setState] = useState<AutosaveState>('idle');
  const [error, setError] = useState<string | null>(null);

  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const latest = useRef({ id: entry?.id ?? null, title: '', content: '' });

  useEffect(() => {
    latest.current = {
      id: entry?.id ?? null,
      title: entry?.title ?? '',
      content: entry?.content ?? '',
    };
  }, [entry?.id, entry?.title, entry?.content]);

  const cancel = useCallback(() => {
    if (timer.current) {
      clearTimeout(timer.current);
      timer.current = null;
    }
  }, []);

  const flush = useCallback(() => {
    const { id, title, content } = latest.current;
    if (!id) {
      return;
    }

    setState('saving');
    entriesApi
      .autosave(id, { title, content })
      .then(() => setState('saved'))
      .catch(() => {
        setState('failed');
        setError('Autosave failed. Your text is still in this tab.');
      });
  }, []);

  const schedule = useCallback(() => {
    if (!latest.current.id) {
      return;
    }

    cancel();
    setState('idle');
    timer.current = setTimeout(flush, delayMs);
  }, [cancel, delayMs, flush]);

  useEffect(() => cancel, [cancel]);

  return { state, error, schedule, flush, cancel };
}