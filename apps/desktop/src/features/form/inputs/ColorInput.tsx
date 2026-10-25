import { Input } from '@/components/ui/input';
import type { FieldInputProps } from '../MetafieldInput';

const SWATCHES = ['#000000', '#ffffff', '#8b5a2b', '#c19a6b', '#7a7a7a', '#2f4f4f', '#800000', '#1e3a5f'];

/** `color`: swatches plus a hex entry; stored `#rrggbb` lowercase by the backend (MC-9). */
export function ColorInput({ id, label, value, onChange, onBlur }: FieldInputProps) {
  const current = typeof value === 'string' ? value : '';
  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {SWATCHES.map(swatch => (
        <button
          key={swatch}
          type="button"
          aria-label={`${label} ${swatch}`}
          aria-pressed={current.toLowerCase() === swatch}
          className="size-6 rounded-full border border-border aria-pressed:ring-2 aria-pressed:ring-foreground"
          style={{ background: swatch }}
          onClick={() => onChange(swatch)}
        />
      ))}
      <Input id={id} className="ml-2 w-28" placeholder="#rrggbb" value={current} onChange={event => onChange(event.target.value)} onBlur={onBlur} />
    </div>
  );
}
