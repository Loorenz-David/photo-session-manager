import { Minus, Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import type { FieldInputProps } from '../MetafieldInput';

/** `int`: stepper. A whole number is saved as a JSON integer; anything else typed is saved as typed (MC-9). */
export function IntInput({ id, label, value, onChange, onBlur }: FieldInputProps) {
  const shown = typeof value === 'number' || typeof value === 'string' ? String(value) : '';
  const number = typeof value === 'number' ? value : 0;
  const fromText = (text: string) => (/^-?\d+$/.test(text.trim()) ? Number(text.trim()) : text);
  return (
    <div className="flex w-40 items-center gap-1">
      <Button type="button" variant="outline" size="icon-sm" aria-label={`Decrease ${label}`} onClick={() => onChange(number - 1)}>
        <Minus />
      </Button>
      <Input id={id} inputMode="numeric" className="text-center" value={shown} onChange={event => onChange(fromText(event.target.value))} onBlur={onBlur} />
      <Button type="button" variant="outline" size="icon-sm" aria-label={`Increase ${label}`} onClick={() => onChange(number + 1)}>
        <Plus />
      </Button>
    </div>
  );
}
