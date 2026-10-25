import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import type { FieldInputProps } from '../MetafieldInput';

/** `multi`: ≤ 4 short list options as pills; the value keeps the order in which they were picked. */
export function MultiPills({ id, label, definition, value, onChange }: FieldInputProps) {
  const current = Array.isArray(value) ? (value as string[]) : [];
  return (
    <ToggleGroup
      id={id}
      type="multiple"
      variant="outline"
      aria-label={label}
      value={current}
      onValueChange={next => onChange([...current.filter(v => next.includes(v)), ...next.filter(v => !current.includes(v))])}
      className="flex-wrap"
    >
      {definition.choices.map(choice => (
        <ToggleGroupItem key={choice} value={choice} className="rounded-full px-3.5">
          {choice}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}
