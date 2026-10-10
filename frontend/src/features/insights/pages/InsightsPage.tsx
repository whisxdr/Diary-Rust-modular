import { Card } from '@/components/ui/Card';
import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Spinner } from '@/components/feedback/Spinner';
import { useAsyncData } from '@/hooks/useAsyncData';
import { formatWordCount, percent, pluralize } from '@/lib/formatting';

import { insightsApi } from '../api';

const WEEKDAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

export function InsightsPage() {
  const insights = useAsyncData((signal) => insightsApi.overview(signal), []);

  if (insights.isLoading) {
    return <Spinner />;
  }
  if (insights.error || !insights.data) {
    return <ErrorMessage message={insights.error ?? 'Insights are unavailable.'} />;
  }

  const data = insights.data;

  return (
    <div className="page">
      <header className="page__header">
        <h1>Reflection</h1>
        <p className="hint">Derived from everything you have written.</p>
      </header>

      <div className="grid">
        <Card title="Entries">
          <p className="stat">{pluralize(data.total_entries, 'entry', 'entries')}</p>
          <p className="hint">{formatWordCount(data.total_words)}</p>
        </Card>

        <Card title="Streaks">
          <p className="stat">{data.current_streak} days</p>
          <p className="hint">Longest: {data.longest_streak} days</p>
        </Card>

        <Card title="First entry">
          <p className="stat">{data.first_entry_date ?? '—'}</p>
        </Card>
      </div>

      <Card title="Moods">
        {data.mood_distribution.length === 0 ? (
          <p className="hint">No moods recorded yet.</p>
        ) : (
          <ul className="bars">
            {data.mood_distribution.map((mood) => (
              <li key={mood.slug}>
                <span>{mood.label}</span>
                <span className="bar" style={{ inlineSize: percent(mood.share) }} />
                <span className="hint">
                  {mood.entries} · {percent(mood.share)}
                </span>
              </li>
            ))}
          </ul>
        )}
      </Card>

      <Card title="Tags you use most">
        {data.top_tags.length === 0 ? (
          <p className="hint">No tags yet.</p>
        ) : (
          <ul className="chips">
            {data.top_tags.map((tag) => (
              <li key={tag.name} className="chip">
                {tag.name} <span className="hint">{tag.entries}</span>
              </li>
            ))}
          </ul>
        )}
      </Card>

      <Card title="When you write">
        <ul className="bars">
          {WEEKDAYS.map((label, index) => {
            const count = data.writing_by_weekday.find((row) => row.weekday === index)?.entries ?? 0;
            return (
              <li key={label}>
                <span>{label}</span>
                <span className="hint">{count}</span>
              </li>
            );
          })}
        </ul>
      </Card>
    </div>
  );
}