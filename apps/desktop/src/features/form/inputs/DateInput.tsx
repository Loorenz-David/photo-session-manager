import { Input } from '@/components/ui/input';
import type { FieldInputProps } from '../MetafieldInput';

/** `date`: the webview's native date picker (delegated choice); the value is `YYYY-MM-DD` (MC-9). */
export function DateInput({ id, value, onChange, onBlur }: FieldInputProps) {
  return <Input id={id} type="date" className="w-48" value={typeof value === 'string' ? value : ''} onChange={event => onChange(event.target.value)} onBlur={onBlur} />;
}
