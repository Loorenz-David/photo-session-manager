import { Link } from 'lucide-react';
import { Input } from '@/components/ui/input';
import type { FieldInputProps } from '../MetafieldInput';

/** `url`: input with a link icon. */
export function UrlInput({ id, value, onChange, onBlur }: FieldInputProps) {
  return (
    <div className="relative">
      <Link className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden />
      <Input id={id} type="url" className="pl-8" placeholder="https://" value={typeof value === 'string' ? value : ''} onChange={event => onChange(event.target.value)} onBlur={onBlur} />
    </div>
  );
}
