import { cn } from '@/lib/utils';
import type { FieldInputProps } from '../MetafieldInput';

/** "Very Good - This vintage item has no defects…" → title and description. */
function split(choice: string): { title: string; description: string | null } {
  const at = choice.indexOf(' - ');
  return at === -1 ? { title: choice, description: null } : { title: choice.slice(0, at), description: choice.slice(at + 3) };
}

/** `radio`: options long enough to carry a description render as cards; the stored value is the whole option. */
export function RadioCards({ id, label, definition, value, onChange }: FieldInputProps) {
  return (
    <div id={id} role="radiogroup" aria-label={label} className="grid grid-cols-2 gap-2">
      {definition.choices.map(choice => {
        const on = value === choice;
        const { title, description } = split(choice);
        return (
          <button
            key={choice}
            type="button"
            role="radio"
            aria-checked={on}
            className={cn('flex gap-2.5 rounded-lg border p-3 text-left', on ? 'border-foreground bg-muted/50' : 'border-border')}
            onClick={() => onChange(on ? '' : choice)}
          >
            <span className={cn('mt-0.5 size-3.5 shrink-0 rounded-full border', on && 'border-4 border-foreground')} aria-hidden />
            <span>
              <span className="block text-sm font-semibold">{title}</span>
              {description && <span className="block text-xs text-muted-foreground">{description}</span>}
            </span>
          </button>
        );
      })}
    </div>
  );
}
