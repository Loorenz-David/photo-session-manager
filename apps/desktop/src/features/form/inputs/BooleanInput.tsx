import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import type { FieldInputProps } from '../MetafieldInput';

/** `boolean`: Yes / No; pressing the chosen one again empties the field. */
export function BooleanInput({ id, label, value, onChange }: FieldInputProps) {
  const current = value === true ? 'yes' : value === false ? 'no' : '';
  return (
    <ToggleGroup id={id} type="single" variant="outline" aria-label={label} value={current} onValueChange={next => onChange(next === 'yes' ? true : next === 'no' ? false : null)}>
      <ToggleGroupItem value="yes" className="px-4">
        Yes
      </ToggleGroupItem>
      <ToggleGroupItem value="no" className="px-4">
        No
      </ToggleGroupItem>
    </ToggleGroup>
  );
}
