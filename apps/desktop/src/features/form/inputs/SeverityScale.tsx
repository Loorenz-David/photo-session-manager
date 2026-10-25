import { cn } from '@/lib/utils';
import { SEVERITY_SET } from '../kinds';
import type { FieldInputProps } from '../MetafieldInput';

/** Colour ramp of the severity scale: quiet for N/A and None, warmer as damage grows. */
const RAMP: Record<string, string> = {
  'N/A': 'data-[state=on]:bg-muted',
  None: 'data-[state=on]:bg-[oklch(0.93_0.04_150)]',
  Minor: 'data-[state=on]:bg-[oklch(0.9_0.06_25)]',
  Moderate: 'data-[state=on]:bg-[oklch(0.84_0.1_30)]',
  Major: 'data-[state=on]:bg-[oklch(0.76_0.14_29)]',
};

/** `severity`: the N/A · None · Minor · Moderate · Major set (MC-9), always shown in that order. */
export function SeverityScale({ id, label, value, onChange }: FieldInputProps) {
  const current = typeof value === 'string' ? value : '';
  return (
    <div id={id} role="radiogroup" aria-label={label} className="grid grid-cols-5 gap-1 rounded-lg bg-muted/60 p-1">
      {SEVERITY_SET.map(choice => {
        const on = current === choice;
        return (
          <button
            key={choice}
            type="button"
            role="radio"
            aria-checked={on}
            data-state={on ? 'on' : 'off'}
            className={cn('rounded-md py-1.5 text-xs font-medium text-muted-foreground data-[state=on]:font-semibold data-[state=on]:text-foreground', RAMP[choice])}
            onClick={() => onChange(on ? '' : choice)}
          >
            {choice}
          </button>
        );
      })}
    </div>
  );
}
