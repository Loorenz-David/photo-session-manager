import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import type { AttributeValue } from '@/services/attributes';
import { AiPill } from './AiPill';

/** Shown value of a stored row: a JSON string without quotes, a list joined, anything else as JSON. */
export function displayValue(value: AttributeValue): string {
  try {
    const parsed = JSON.parse(value.value);
    if (typeof parsed === 'string') return parsed;
    if (Array.isArray(parsed)) return parsed.join(', ');
    if (parsed && typeof parsed === 'object' && 'value' in parsed) return `${parsed.value}${parsed.unit ? ` ${parsed.unit}` : ''}`;
    return value.value;
  } catch {
    return value.value;
  }
}

const CONFIDENCE_LABEL: Record<string, string> = { high: 'High confidence', medium: 'Medium confidence', low: 'Low confidence · check' };

type AiPopoverProps = {
  value: AttributeValue;
  label: string;
  onLooksRight: () => Promise<unknown>;
  onClear: () => Promise<unknown>;
  onSeeResearch: () => void;
};

/** The AI pill's popover (ch08, NOTES §AI agent rules): confidence, value, reason, sources, Looks right / Clear / See research. */
export function AiPopover({ value, label, onLooksRight, onClear, onSeeResearch }: AiPopoverProps) {
  const [open, setOpen] = useState(false);
  const act = (action: () => void) => () => {
    setOpen(false);
    action();
  };
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <AiPill confidence={value.ai_confidence} />
      </PopoverTrigger>
      <PopoverContent align="start" className="w-80" aria-label={`AI suggestion for ${label}`}>
        <p className="text-xs font-semibold text-(--ai-foreground)">{CONFIDENCE_LABEL[value.ai_confidence ?? ''] ?? 'AI suggestion'}</p>
        <p className="text-sm font-semibold">{displayValue(value)}</p>
        {value.ai_reason && <p className="text-xs text-muted-foreground">{value.ai_reason}</p>}
        {(value.ai_sources ?? []).length > 0 && (
          <ul className="flex flex-col gap-0.5 text-xs">
            {(value.ai_sources ?? []).map(source => (
              <li key={source} className="truncate">
                <a href={source} target="_blank" rel="noreferrer" className="underline underline-offset-2">
                  {source}
                </a>
              </li>
            ))}
          </ul>
        )}
        <div className="flex gap-1.5">
          <ActionButton size="sm" action={async () => { setOpen(false); await onLooksRight(); }}>
            Looks right
          </ActionButton>
          <ActionButton size="sm" variant="outline" action={async () => { setOpen(false); await onClear(); }}>
            Clear
          </ActionButton>
          <Button size="sm" variant="ghost" onClick={act(onSeeResearch)}>
            See research
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  );
}
