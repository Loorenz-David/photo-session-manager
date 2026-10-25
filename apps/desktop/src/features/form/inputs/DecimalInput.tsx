import { Input } from '@/components/ui/input';
import { validationsOf, type FieldInputProps } from '../MetafieldInput';

/** Currency of a money field: the stored one, else the definition's, else the shop's kronor (delegated). */
function currencyOf(props: FieldInputProps): string {
  const stored = (props.value as { currency?: string } | undefined)?.currency;
  return stored ?? (validationsOf(props.definition).currency as string | undefined) ?? 'SEK';
}

/** `decimal` (a decimal string, never a float) and `money` (`{amount, currency}` with a currency prefix). */
export function DecimalInput(props: FieldInputProps) {
  const { id, kind, value, onChange, onBlur } = props;
  if (kind === 'money') {
    const currency = currencyOf(props);
    const amount = (value as { amount?: string } | undefined)?.amount ?? '';
    return (
      <div className="relative w-48">
        <span className="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-xs text-muted-foreground">{currency}</span>
        <Input id={id} inputMode="decimal" className="pl-11" value={amount} onChange={event => onChange(event.target.value === '' ? '' : { amount: event.target.value, currency })} onBlur={onBlur} />
      </div>
    );
  }
  return <Input id={id} inputMode="decimal" className="w-48" value={typeof value === 'string' ? value : ''} onChange={event => onChange(event.target.value)} onBlur={onBlur} />;
}
