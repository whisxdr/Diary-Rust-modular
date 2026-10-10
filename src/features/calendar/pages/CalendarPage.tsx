import { useState } from 'react';
import { Link } from 'react-router-dom';

import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Spinner } from '@/components/feedback/Spinner';
import { useAsyncData } from '@/hooks/useAsyncData';
import { currentMonth, monthLabel, toIsoDate } from '@/lib/date';
import { pluralize } from '@/lib/formatting';

import { calendarApi } from '../api';

const WEEKDAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

export function CalendarPage() {
  const [cursor, setCursor] = useState(currentMonth);

  const month = useAsyncData((signal) => calendarApi.month(cursor.year, cursor.month, signal), [
    cursor.year,
    cursor.month,
  ]);

  function shift(delta: number) {
    setCursor((current) => {
      const next = current.month + delta;
      if (next < 1) {
        return { year: current.year - 1, month: 12 };
      }
      if (next > 12) {
        return { year: current.year + 1, month: 1 };
      }
      return { year: current.year, month: next };
    });
  }

  if (month.isLoading) {
    return <Spinner />;
  }
  if (month.error || !month.data) {
    return <ErrorMessage message={month.error ?? 'The calendar is unavailable.'} />;
  }

  const data = month.data;
  const leadingBlanks = weekdayIndexOf(data.first_date);

  return (
    <div className="page">
      <header className="page__header">
        <h1>{monthLabel(cursor.year, cursor.month)}</h1>
        <div className="calendar__nav">
          <button type="button" onClick={() => shift(-1)} aria-label="Previous month">
            ‹
          </button>
          <button type="button" onClick={() => setCursor(currentMonth())} aria-label="Today">
            Today
          </button>
          <button type="button" onClick={() => shift(1)} aria-label="Next month">
            ›
          </button>
        </div>
      </header>

      <div className="calendar">
        {WEEKDAY_LABELS.map((label) => (
          <span key={label} className="calendar__weekday">
            {label}
          </span>
        ))}

        {Array.from({ length: leadingBlanks }, (_, index) => (
          <span key={`blank-${index}`} className="calendar__blank" />
        ))}

        {data.days.map((day) => (
          <Link
            key={day.date}
            to={`/?date=${day.date}`}
            className={`calendar__day ${day.has_entry ? 'calendar__day--filled' : ''} ${day.is_today ? 'calendar__day--today' : ''}`}
            title={`${day.date} · ${pluralize(day.word_count, 'word')}`}
          >
            <span>{Number(day.date.slice(-2))}</span>
            {day.mood_slug ? <span className="calendar__mood" aria-hidden="true" /> : null}
          </Link>
        ))}
      </div>

      <p className="hint">
        {pluralize(data.total_entries, 'entry', 'entries')} · {pluralize(data.total_words, 'word')} this month
        {data.first_date ? ` · starting ${toIsoDate(new Date(data.first_date))}` : ''}
      </p>
    </div>
  );
}

/** Monday-first column index for the first day of the month. */
function weekdayIndexOf(isoDate: string): number {
  const [year, month, day] = isoDate.split('-').map(Number);
  const date = new Date(year ?? 1970, (month ?? 1) - 1, day ?? 1);
  return (date.getDay() + 6) % 7;
}