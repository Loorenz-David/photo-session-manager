import { Star } from 'lucide-react';
import { validationsOf, type FieldInputProps } from '../MetafieldInput';

/** `rating`: star buttons over the definition's scale (`validations.min`/`max`, else 0–5); `{value, min, max}` (MC-9). */
export function RatingInput({ id, label, definition, value, onChange }: FieldInputProps) {
  const validations = validationsOf(definition);
  const min = String(validations.min ?? '0');
  const max = String(validations.max ?? '5');
  const stars = Math.max(1, Math.min(10, Math.round(Number(max))));
  const current = Number((value as { value?: string } | undefined)?.value ?? 0);
  return (
    <div id={id} role="radiogroup" aria-label={label} className="flex gap-0.5">
      {Array.from({ length: stars }, (_, index) => index + 1).map(n => (
        <button key={n} type="button" role="radio" aria-checked={current === n} aria-label={`${n} of ${stars}`} onClick={() => onChange(current === n ? null : { value: String(n), min, max })}>
          <Star className={n <= current ? 'size-5 fill-foreground' : 'size-5 text-muted-foreground'} />
        </button>
      ))}
    </div>
  );
}
