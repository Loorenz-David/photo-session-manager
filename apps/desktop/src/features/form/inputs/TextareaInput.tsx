import { Textarea } from '@/components/ui/textarea';
import type { FieldInputProps } from '../MetafieldInput';

/** `textarea` (textarea and rich_text): multi-line input; paragraphs separated by a blank line (MC-9). */
export function TextareaInput({ id, value, onChange, onBlur, placeholder }: FieldInputProps) {
  return <Textarea id={id} rows={4} value={typeof value === 'string' ? value : ''} placeholder={placeholder} onChange={event => onChange(event.target.value)} onBlur={onBlur} />;
}
