import { useState, type KeyboardEvent } from 'react';
import { X } from 'lucide-react';
import { validationsOf, type FieldInputProps } from '../MetafieldInput';

/**
 * `tags`: free entry (Enter or comma adds, Backspace on an empty entry removes the last tag) plus suggestions
 * from `validations.suggestions` when the integration provides them (none without one).
 */
export function TagsInput({ id, label, definition, value, onChange, onBlur }: FieldInputProps) {
  const [entry, setEntry] = useState('');
  const current = Array.isArray(value) ? (value as string[]) : [];
  const suggestions = (validationsOf(definition).suggestions as string[] | undefined) ?? [];
  const matches = entry.trim() ? suggestions.filter(s => s.toLowerCase().includes(entry.trim().toLowerCase()) && !current.includes(s)).slice(0, 6) : [];

  function add(tag: string) {
    const clean = tag.trim();
    setEntry('');
    if (clean && !current.includes(clean)) onChange([...current, clean]);
  }

  function onKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === 'Enter' || event.key === ',') {
      event.preventDefault();
      add(entry);
    } else if (event.key === 'Backspace' && entry === '' && current.length > 0) {
      onChange(current.slice(0, -1));
    }
  }

  return (
    <div>
      <div className="flex min-h-9 flex-wrap items-center gap-1.5 rounded-lg border border-input px-2 py-1">
        {current.map(tag => (
          <span key={tag} className="inline-flex items-center gap-1 rounded-full bg-muted px-2.5 py-0.5 text-xs">
            {tag}
            <button type="button" aria-label={`Remove ${tag}`} onClick={() => onChange(current.filter(t => t !== tag))}>
              <X className="size-3" />
            </button>
          </span>
        ))}
        <input
          id={id}
          aria-label={label}
          className="min-w-24 flex-1 bg-transparent py-1 text-sm outline-none"
          placeholder="Add a tag"
          value={entry}
          onChange={event => setEntry(event.target.value)}
          onKeyDown={onKeyDown}
          onBlur={() => {
            if (entry.trim()) add(entry);
            onBlur();
          }}
        />
      </div>
      {matches.length > 0 && (
        <div className="mt-1.5 flex flex-wrap gap-1.5">
          {matches.map(s => (
            <button key={s} type="button" className="rounded-full border px-2.5 py-0.5 text-xs text-muted-foreground" onMouseDown={event => event.preventDefault()} onClick={() => add(s)}>
              + {s}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
