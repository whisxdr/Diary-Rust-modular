import { moodsApi } from '@/features/moods/api';
import { useAsyncData } from '@/hooks/useAsyncData';

export function MoodPicker({
  selected,
  onChange,
}: {
  selected: string | null;
  onChange: (moodId: string | null) => void;
}) {
  const { data: moods, isLoading } = useAsyncData((signal) => moodsApi.list(signal), []);

  if (isLoading) {
    return <p className="hint">Loading moods…</p>;
  }

  return (
    <div className="moods" role="group" aria-label="Mood">
      {moods?.map((mood) => (
        <button
          key={mood.id}
          type="button"
          className={`mood ${selected === mood.id ? 'mood--selected' : ''}`}
          aria-pressed={selected === mood.id}
          onClick={() => onChange(selected === mood.id ? null : mood.id)}
        >
          <span aria-hidden="true">{mood.icon}</span>
          {mood.label}
        </button>
      ))}
      {moods?.length === 0 ? <p className="hint">No moods available.</p> : null}
    </div>
  );
}