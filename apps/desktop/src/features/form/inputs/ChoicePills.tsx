import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import type { FieldInputProps } from '../MetafieldInput';

/** `choice` with ≤ 4 short options: pills; pressing the chosen pill again empties the field. */
export function ChoicePills({ id, label, definition, value, onChange }: FieldInputProps) {
  return (
    <ToggleGroup id={id} type="single" variant="outline" aria-label={label} value={typeof value === 'string' ? value : ''} onValueChange={next => onChange(next)} className="flex-wrap">
      {definition.choices.map(choice => (
        <ToggleGroupItem key={choice} value={choice} className="rounded-full px-3.5">
          {choice}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}
