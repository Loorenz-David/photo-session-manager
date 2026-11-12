import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import type { AttributeValue } from '@/services/attributes';
import { AiPill } from './AiPill';

/** Shown text of a value held as JSON text: a string without quotes, a list joined, a measurement with its unit. */
export function jsonText(text: string): string {
  try {
    const parsed = JSON.parse(text);
    if (typeof parsed === 'string') return parsed;
    if (Array.isArray(parsed)) return parsed.join(', ');
    if (parsed && typeof parsed === 'object' && 'value' in parsed) return `${parsed.value}${parsed.unit ? ` ${parsed.unit}` : ''}`;
    return text;
  } catch {
    return text;
  }
}

/** Shown value of a stored row: a JSON string without quotes, a list joined, anything else as JSON. */
export function displayValue(value: AttributeValue): string {
  return jsonText(value.value);
}

/** Where a conflict's previous value came from (08B MC-A10). */
export function priorOrigin(provenance: string): string {
  return provenance === 'integration' ? 'from Shopify' : 'preset default';
}

const CONFIDENCE_LABEL: Record<string, string> = { high: 'High confidence', medium: 'Medium confidence', low: 'Low confidence · check' };

/** The three actions of a conflict (08B MC-A12): offered in place of Looks right / Clear. */
export type ConflictActions = {
  onKeepPrevious: () => Promise<unknown>;
  onUseSuggestion: () => Promise<unknown>;
  /** Puts the field in edit mode; the typed value is saved once, on Enter, leaving the field, or Save & close. */
  onEdit: () => void;
};

type AiPopoverProps = {
  value: AttributeValue;
  label: string;
  onLooksRight: () => Promise<unknown>;
  onClear: () => Promise<unknown>;
  onSeeResearch: () => void;
  /** Set when the field is in conflict: the popover shows the previous value and the conflict's actions. */
  conflict?: ConflictActions;
};

/**
 * The AI pill's popover (ch08, NOTES §AI agent rules): confidence, value, reason, sources, Looks right / Clear /
 * See research. Looks right accepts a suggestion, so it is offered only on a suggested row (08B MC-A12). A conflict
 * shows the previous value and where it came from, and Keep previous / Use suggestion / Edit instead (MC-A10).
 */
export function AiPopover({ value, label, onLooksRight, onClear, onSeeResearch, conflict }: AiPopoverProps) {
  const [open, setOpen] = useState(false);
  const act = (action: () => void) => () => {
    setOpen(false);
    action();
  };
  const prior = conflict ? value.ai_prior : null;
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <AiPill confidence={value.ai_confidence} />
      </PopoverTrigger>
      <PopoverContent align="start" className="w-80" aria-label={`AI suggestion for ${label}`}>
        <p className="text-xs font-semibold text-(--ai-foreground)">{CONFIDENCE_LABEL[value.ai_confidence ?? ''] ?? 'AI suggestion'}</p>
        <p className="text-sm font-semibold">{displayValue(value)}</p>
        {prior && (
          <p className="text-xs" data-conflict-prior>
            Replaces <span className="font-semibold">{jsonText(prior.value)}</span> · {priorOrigin(prior.provenance)}
          </p>
        )}
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
        {conflict ? (
          <>
            <div className="flex flex-wrap gap-1.5">
              <ActionButton size="sm" variant="outline" action={async () => { setOpen(false); await conflict.onKeepPrevious(); }}>
                Keep previous
              </ActionButton>
              <ActionButton size="sm" action={async () => { setOpen(false); await conflict.onUseSuggestion(); }}>
                Use suggestion
              </ActionButton>
              <Button size="sm" variant="outline" onClick={act(conflict.onEdit)}>
                Edit
              </Button>
              <Button size="sm" variant="ghost" onClick={act(onSeeResearch)}>
                See research
              </Button>
            </div>
            <p className="text-[11px] text-muted-foreground">
              A publish that already went ahead keeps the value you chose for it; the next publish waits for this choice again.
            </p>
          </>
        ) : (
          <div className="flex gap-1.5">
            {value.ai_status === 'suggested' && (
              <ActionButton size="sm" action={async () => { setOpen(false); await onLooksRight(); }}>
                Looks right
              </ActionButton>
            )}
            <ActionButton size="sm" variant="outline" action={async () => { setOpen(false); await onClear(); }}>
              Clear
            </ActionButton>
            <Button size="sm" variant="ghost" onClick={act(onSeeResearch)}>
              See research
            </Button>
          </div>
        )}
      </PopoverContent>
    </Popover>
  );
}
