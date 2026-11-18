import { CornerDownRight, Sparkles } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import type { FormField, Group } from '@/services/attributes';
import type { ResearchStatus } from '@/services/research';
import { ConfidenceBars, CONFIDENCE_BARS } from './AiPill';
import { SuggestionBody, type ConflictActions } from './AiPopover';
import { STEP_LABEL } from './StepTabs';

/** A field holding an AI suggestion to confirm, and the step it is on. */
export type ReviewEntry = { field: FormField; group: Group };

/** Why the AI review tab is empty (A11): nothing pending, said in the words of the research state. */
export function emptyReason(status: ResearchStatus | null, anyAi: boolean): string {
  const run = status?.run ?? null;
  if (run && (run.state === 'running' || run.state === 'queued')) return 'Research is running. Its suggestions appear here as they come in.';
  if (anyAi) return 'Every AI suggestion on this item has been checked.';
  if (!run) {
    if (status?.reason?.sentence) return `Research hasn't run for this item yet. ${status.reason.sentence}`;
    return "Research hasn't run for this item yet.";
  }
  return 'The last research run suggested nothing to confirm.';
}

type AiReviewTabProps = {
  entries: ReviewEntry[];
  /** What Accept all would accept (neither low nor a conflict). */
  acceptable: number;
  status: ResearchStatus | null;
  /** Some field holds an AI value (accepted ones included). */
  anyAi: boolean;
  onLooksRight: (field: FormField) => Promise<unknown>;
  onClear: (field: FormField) => Promise<unknown>;
  conflictActions: (entry: ReviewEntry) => ConflictActions | undefined;
  onSeeResearch: (key: string | null) => void;
  onShowField: (entry: ReviewEntry) => void;
  onAcceptAll: () => Promise<unknown>;
};

/**
 * AI review (A11, owner 2026-11-18): every field holding an unchecked AI suggestion in one place, in step and form
 * order. Each row has the field, its step, what it held before, the suggestion, confidence, reason and sources, and the
 * same actions as the field's AI pill (Looks right / Clear, or a conflict's Keep previous / Use suggestion / Edit), so
 * the rules and the undo are those of the pill. A row leaves the list once its suggestion is checked.
 */
export function AiReviewTab({ entries, acceptable, status, anyAi, onLooksRight, onClear, conflictActions, onSeeResearch, onShowField, onAcceptAll }: AiReviewTabProps) {
  if (entries.length === 0) {
    return (
      <div className="flex flex-col items-start gap-2 py-6" data-review-empty>
        <p className="text-sm font-semibold">Nothing to confirm</p>
        <p className="text-sm text-muted-foreground">{emptyReason(status, anyAi)}</p>
        <Button size="sm" variant="ghost" className="text-(--ai-foreground)" onClick={() => onSeeResearch(null)}>
          <Sparkles /> See research
        </Button>
      </div>
    );
  }
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-2">
        <p className="mr-auto text-sm text-muted-foreground">
          {entries.length === 1 ? '1 suggestion to confirm.' : `${entries.length} suggestions to confirm.`} Accept all leaves low-confidence fields and conflicts for you to check.
        </p>
        <ActionButton size="sm" disabled={acceptable === 0} action={onAcceptAll}>
          Accept all{acceptable > 0 ? ` (${acceptable})` : ''}
        </ActionButton>
        <Button size="sm" variant="ghost" className="text-(--ai-foreground)" onClick={() => onSeeResearch(null)}>
          <Sparkles /> See research
        </Button>
      </div>
      <ul className="flex flex-col gap-3" aria-label="Suggestions to confirm">
        {entries.map(entry => {
          const { definition } = entry.field;
          const value = entry.field.value!;
          const conflict = conflictActions(entry);
          return (
            <li key={definition.key} data-review-row={definition.key} aria-label={definition.label} className="flex flex-col gap-1.5 rounded-lg border p-3">
              <div className="flex items-center gap-2 text-xs">
                <span className="text-sm font-semibold">{definition.label}</span>
                <span className="text-muted-foreground">· {STEP_LABEL[entry.group]}</span>
                {conflict && <span className="rounded-full bg-amber-100 px-2 py-0.5 text-[11px] font-semibold text-amber-900">Conflict</span>}
                <span className="text-(--ai-foreground)">
                  <ConfidenceBars filled={CONFIDENCE_BARS[value.ai_confidence ?? ''] ?? 1} />
                </span>
                <button type="button" className="ml-auto flex items-center gap-1 rounded px-1.5 py-0.5 text-muted-foreground hover:bg-muted" onClick={() => onShowField(entry)}>
                  <CornerDownRight className="size-3" aria-hidden /> Show field
                </button>
              </div>
              <SuggestionBody
                value={value}
                showPrior
                onLooksRight={() => onLooksRight(entry.field)}
                onClear={() => onClear(entry.field)}
                onSeeResearch={() => onSeeResearch(definition.key)}
                conflict={conflict}
              />
            </li>
          );
        })}
      </ul>
    </div>
  );
}
