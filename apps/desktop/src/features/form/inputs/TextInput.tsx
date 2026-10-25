import { Input } from '@/components/ui/input';
import type { FieldInputProps } from '../MetafieldInput';

/** `text`: single-line input; the value is saved as typed. */
export function TextInput({ id, value, onChange, onBlur, placeholder }: FieldInputProps) {
  return <Input id={id} value={typeof value === 'string' ? value : ''} placeholder={placeholder} onChange={event => onChange(event.target.value)} onBlur={onBlur} />;
}
