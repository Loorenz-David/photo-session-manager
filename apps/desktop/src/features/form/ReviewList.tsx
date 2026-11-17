import { useState } from 'react';
import { ChevronRight, Sparkles } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import type { FormField, Group } from '@/services/attributes';
import { displayValue } from './AiPopover';
import { STEP_LABEL } from './StepTabs';

/** A field waiting for review and the step it is on. */
export type ReviewEntry = { field: FormField; group: Group };

const CONFIDENCE: Record<string, string> = { high: 'High', medium: 'Medium', low: 'Low · check' };

type ReviewListProps = {
  entries: ReviewEntry[];
  /** What Accept all would accept (neither low nor a conflict). */
  acceptable: number;
  onChoose: (entry: ReviewEntry) => void;
  onAcceptAll: () => Promise<unknown>;
  onOpenResearch: () => void;
};

/**
 * "n to review" (A6, owner 2026-11-17): the chip opens the list of fields holding an unchecked AI suggestion (label,
 * suggested value, confidence, conflict marked). Choosing one goes to its step, scrolls to it and opens its AI popover,
 * where Looks right / Clear (or a conflict's choices) are.
 */
export function ReviewList({ entries, acceptable, onChoose, onAcceptAll, onOpenResearch }: ReviewListProps) {
  const [open, setOpen] = useState(false);
  if (entries.length === 0) return null;
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button type="button" data-to-review aria-haspopup="dialog" className="rounded-full bg-(--ai-muted) px-2.5 py-0.5 text-xs font-semibold text-(--ai-foreground)">
          {entries.length} to review
        </button>
      </PopoverTrigger>
      {/* Focus does not go back to the chip on close: the chosen field's popover opens and keeps it. */}
      <PopoverContent align="end" className="w-96 p-0" aria-label="Fields to review" onCloseAutoFocus={event => event.preventDefault()}>
        <p className="border-b px-3 py-2 text-xs text-muted-foreground">Choose a field to check its suggestion.</p>
        <ul className="flex max-h-80 flex-col overflow-y-auto py-1" aria-label="Fields to review">
          {entries.map(entry => {
            const value = entry.field.value!;
            return (
              <li key={entry.field.definition.key}>
                <button
                  type="button"
                  data-review-field={entry.field.definition.key}
                  className="flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-muted"
                  onClick={() => {
                    setOpen(false);
                    onChoose(entry);
                  }}
                >
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="flex items-center gap-1.5 text-xs">
                      <span className="font-semibold">{entry.field.definition.label}</span>
                      <span className="text-muted-foreground">· {STEP_LABEL[entry.group]}</span>
                    </span>
                    <span className="truncate text-sm">{displayValue(value)}</span>
                  </span>
                  {value.conflict && <span className="rounded-full bg-amber-100 px-2 py-0.5 text-[11px] font-semibold text-amber-900">Conflict</span>}
                  <span className="shrink-0 text-[11px] text-(--ai-foreground)" data-confidence={value.ai_confidence ?? undefined}>
                    {CONFIDENCE[value.ai_confidence ?? ''] ?? 'AI'}
                  </span>
                  <ChevronRight className="size-3.5 shrink-0 text-muted-foreground" aria-hidden />
                </button>
              </li>
            );
          })}
        </ul>
        <div className="flex items-center gap-2 border-t px-3 py-2">
          <ActionButton size="sm" disabled={acceptable === 0} action={async () => { setOpen(false); await onAcceptAll(); }}>
            Accept all{acceptable > 0 ? ` (${acceptable})` : ''}
          </ActionButton>
          <Button size="sm" variant="ghost" onClick={() => { setOpen(false); onOpenResearch(); }}>
            <Sparkles /> See research
          </Button>
        </div>
        <p className="px-3 pb-2 text-[11px] text-muted-foreground">Accept all leaves low-confidence fields and conflicts for you to check.</p>
      </PopoverContent>
    </Popover>
  );
}
