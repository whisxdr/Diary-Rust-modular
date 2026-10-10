import type { AutosaveState } from '../hooks/useAutosave';

const LABELS: Record<AutosaveState, string> = {
  idle: '',
  saving: 'Saving…',
  saved: 'Saved',
  failed: 'Not saved',
};

/** Tiny indicator shown next to the editor while autosave runs. */
export function AutosaveStatus({ state, error }: { state: AutosaveState; error: string | null }) {
  const label = LABELS[state];
  if (!label) {
    return null;
  }

  return (
    <p className={`autosave autosave--${state}`} role="status">
      {label}
      {error ? ` — ${error}` : ''}
    </p>
  );
}