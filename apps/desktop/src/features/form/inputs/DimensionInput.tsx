import { Input } from '@/components/ui/input';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import { validationsOf, type FieldInputProps } from '../MetafieldInput';

const DEFAULT_UNITS = { dimension: ['cm', 'mm', 'in'], weight: ['kg', 'lb'] };

/** `dimension` / `weight`: number plus unit; units are the definition's (`validations.units`), else the design's. */
export function DimensionInput({ id, label, kind, definition, value, onChange, onBlur }: FieldInputProps) {
  const units = (validationsOf(definition).units as string[] | undefined) ?? (kind === 'weight' ? DEFAULT_UNITS.weight : DEFAULT_UNITS.dimension);
  const current = (value ?? {}) as { value?: string; unit?: string };
  const unit = current.unit ?? units[0];
  return (
    <div className="flex items-center gap-2">
      <Input id={id} inputMode="decimal" className="w-32" value={current.value ?? ''} onChange={event => onChange(event.target.value === '' ? '' : { value: event.target.value, unit })} onBlur={onBlur} />
      <ToggleGroup type="single" variant="outline" size="sm" aria-label={`${label} unit`} value={unit} onValueChange={next => next && current.value && onChange({ value: current.value, unit: next })}>
        {units.map(u => (
          <ToggleGroupItem key={u} value={u}>
            {u}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}
