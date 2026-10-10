import { useState } from 'react';

/** Type a tag name, press Enter, and it is attached to the entry. */
export function TagPicker({
  selected,
  onChange,
}: {
  selected: string[];
  onChange: (tags: string[]) => void;
}) {
  const [draft, setDraft] = useState('');

  function add(name: string) {
    const clean = name.trim();
    if (!clean || selected.some((tag) => tag.toLowerCase() === clean.toLowerCase())) {
      return;
    }
    onChange([...selected, clean]);
    setDraft('');
  }

  return (
    <div className="tags">
      <div className="tags__list">
        {selected.map((tag) => (
          <span key={tag} className="tag">
            {tag}
            <button
              type="button"
              aria-label={`Remove ${tag}`}
              onClick={() => onChange(selected.filter((value) => value !== tag))}
            >
              ×
            </button>
          </span>
        ))}
      </div>

      <input
        value={draft}
        placeholder="Add a tag and press Enter"
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter') {
            event.preventDefault();
            add(draft);
          }
        }}
        onBlur={() => add(draft)}
      />
    </div>
  );
}