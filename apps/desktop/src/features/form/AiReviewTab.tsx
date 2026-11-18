import { useState } from 'react';
import { ArrowRight, Check, CheckCheck, PencilLine, Sparkles, Undo2, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { ExternalLink } from '@/components/ExternalLink';
import { cn } from '@/lib/utils';
import type { FormField, Group } from '@/services/attributes';
import type { ResearchStatus } from '@/services/research';
import { CONFIDENCE_LABEL, displayValue, jsonText, priorOrigin, type ConflictActions } from './AiPopover';
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

/** The site a source address is on, for its chip. */
function siteOf(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, '');
  } catch {
    return url;
  }
}

/** Reasons longer than this show their first lines with "More". */
const LONG_REASON = 140;

const CONFIDENCE_TONE: Record<string, string> = {
  high: 'bg-(--ai-muted) text-(--ai-foreground)',
  medium: 'bg-(--ai-muted)/70 text-(--ai-foreground)',
  low: 'bg-[color-mix(in_oklch,var(--warn)_18%,transparent)] text-[color-mix(in_oklch,var(--warn),var(--color-text)_45%)]',
};

type RowActions = {
  onAccept: () => Promise<unknown>;
  /** Present when the field held a value before: the suggestion goes and that value comes back. */
  onKeepPrevious?: () => Promise<unknown>;
  /** Present when the field was empty: the suggestion goes and the field stays empty. */
  onDismiss?: () => Promise<unknown>;
  onEdit: () => void;
  onSeeResearch: () => void;
};

/**
 * A18 (owner 2026-11-18): one suggestion, quiet and readable: the field and its step; what it held → what AI suggests;
 * how sure AI is in words; why, in one sentence (more on demand); where from, as small site chips; and three clear
 * actions: Accept, Keep previous (or Dismiss when the field was empty, so "keep what I had" is always one tap), Edit.
 */
function ReviewRow({ entry, conflict, actions }: { entry: ReviewEntry; conflict: boolean; actions: RowActions }) {
  const { definition } = entry.field;
  const value = entry.field.value!;
  const prior = value.ai_prior;
  const [more, setMore] = useState(false);
  const reason = value.ai_reason?.trim() ?? '';
  const long = reason.length > LONG_REASON;
  const confidence = value.ai_confidence ?? '';
  const sources = value.ai_sources ?? [];
  return (
    <li data-review-row={definition.key} aria-label={definition.label} className="flex flex-col gap-3 rounded-xl border bg-card px-4 py-3.5 shadow-(--shadow-sm)">
      <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
        <span className="text-sm font-semibold">{definition.label}</span>
        <span className="text-xs text-muted-foreground">· {STEP_LABEL[entry.group]}</span>
        <span className="ml-auto flex items-center gap-1.5">
          {conflict && (
            <span className="rounded-full bg-[color-mix(in_oklch,var(--warn)_18%,transparent)] px-2 py-0.5 text-[11px] font-medium text-[color-mix(in_oklch,var(--warn),var(--color-text)_45%)]" title="It replaces a value the field already had">
              Conflict
            </span>
          )}
          <span className={cn('rounded-full px-2 py-0.5 text-[11px] font-medium', CONFIDENCE_TONE[confidence] ?? CONFIDENCE_TONE.medium)} data-confidence-label>
            {CONFIDENCE_LABEL[confidence] ?? 'AI suggestion'}
          </span>
        </span>
      </div>

      <div className="flex flex-wrap items-center gap-2 text-sm" data-change>
        {prior ? (
          <span className="flex min-w-0 items-center gap-1.5 text-muted-foreground" data-conflict-prior title={`Replaces ${jsonText(prior.value)} · ${priorOrigin(prior.provenance)}`}>
            <span className="truncate line-through decoration-muted-foreground/50">{jsonText(prior.value)}</span>
            <span className="sr-only">Replaces {jsonText(prior.value)} · {priorOrigin(prior.provenance)}</span>
            <span className="shrink-0 text-[11px]">({priorOrigin(prior.provenance)})</span>
          </span>
        ) : (
          <span className="text-muted-foreground italic" data-prior-empty>
            The field was empty
          </span>
        )}
        <ArrowRight className="size-3.5 shrink-0 text-muted-foreground" aria-hidden />
        <span className="min-w-0 rounded-md bg-(--ai-muted)/60 px-2 py-0.5 font-semibold text-foreground" data-suggested>
          {displayValue(value)}
        </span>
      </div>

      {(reason || sources.length > 0) && (
        <div className="flex flex-col gap-1.5">
          {reason && (
            <p className="text-[13px] leading-relaxed text-muted-foreground">
              <span className={cn(long && !more && 'line-clamp-2')}>{reason}</span>
              {long && (
                <button type="button" className="ml-1 text-xs font-medium text-foreground underline-offset-2 hover:underline" onClick={() => setMore(m => !m)}>
                  {more ? 'Less' : 'More'}
                </button>
              )}
            </p>
          )}
          <div className="flex flex-wrap items-center gap-1.5" aria-label="Sources">
            {sources.map(source => (
              <ExternalLink key={source} href={source} className="inline-flex h-6 max-w-[220px] items-center rounded-full border bg-background px-2 text-[11px] no-underline hover:bg-muted">
                <span className="truncate">{siteOf(source)}</span>
              </ExternalLink>
            ))}
            <button type="button" className="inline-flex h-6 items-center gap-1 rounded-full px-1.5 text-[11px] text-(--ai-foreground) hover:bg-(--ai-muted)" onClick={actions.onSeeResearch}>
              <Sparkles className="size-3" aria-hidden /> See research
            </button>
          </div>
        </div>
      )}

      <div className="flex flex-wrap items-center gap-2 border-t pt-3">
        <ActionButton size="sm" action={actions.onAccept} title="Use the suggested value">
          <Check /> Accept
        </ActionButton>
        {actions.onKeepPrevious ? (
          <ActionButton size="sm" variant="outline" action={actions.onKeepPrevious} title={prior ? `Keep ${jsonText(prior.value)}` : 'Keep the previous value'}>
            <Undo2 /> Keep previous
          </ActionButton>
        ) : actions.onDismiss ? (
          <ActionButton size="sm" variant="outline" action={actions.onDismiss} title="Don't use the suggestion; the field stays empty">
            <X /> Dismiss
          </ActionButton>
        ) : null}
        <Button size="sm" variant="ghost" onClick={actions.onEdit} title="Type your own value in the field">
          <PencilLine /> Edit
        </Button>
        {!reason && sources.length === 0 && (
          <Button size="sm" variant="ghost" className="ml-auto text-(--ai-foreground)" onClick={actions.onSeeResearch}>
            <Sparkles /> See research
          </Button>
        )}
      </div>
    </li>
  );
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
  /** A18: the field gets back the value it held before the suggestion (any suggestion with a prior). */
  onKeepPrevious: (field: FormField) => Promise<unknown>;
  conflictActions: (entry: ReviewEntry) => ConflictActions | undefined;
  onSeeResearch: (key: string | null) => void;
  onShowField: (entry: ReviewEntry) => void;
  onAcceptAll: () => Promise<unknown>;
  /** A19: the review queue: how many other sessions wait, and opening the next one (or the Sessions page). */
  nextToReview?: { left: number; onNext: () => void };
};

/**
 * AI review (A11, owner 2026-11-18; A18 redesign): every field holding an unchecked AI suggestion in one place, in step
 * and form order. Each row offers Accept, Keep previous (or Dismiss when the field was empty) and Edit, under the
 * pill's rules and undo. A row leaves the list once its suggestion is checked.
 */
export function AiReviewTab({ entries, acceptable, status, anyAi, onLooksRight, onClear, onKeepPrevious, conflictActions, onSeeResearch, onShowField, onAcceptAll, nextToReview }: AiReviewTabProps) {
  const running = status?.run?.state === 'running' || status?.run?.state === 'queued';
  if (entries.length === 0 && anyAi && !running && nextToReview) {
    // A19: the last suggestion is checked: on to the next session waiting for review.
    return (
      <div className="flex flex-col items-center gap-3 py-12 text-center" data-review-empty data-all-checked>
        <span className="flex size-11 items-center justify-center rounded-full bg-(--ai-muted) text-(--ai-foreground)" aria-hidden>
          <CheckCheck className="size-5" />
        </span>
        <div>
          <p className="text-base font-semibold">All checked ✓</p>
          <p className="text-sm text-muted-foreground">
            {nextToReview.left === 0
              ? 'No other session waits for review.'
              : nextToReview.left === 1
                ? '1 more session waits for review.'
                : `${nextToReview.left} more sessions wait for review.`}
          </p>
        </div>
        <Button onClick={nextToReview.onNext}>
          {nextToReview.left === 0 ? 'Back to sessions' : 'Next to review'} <ArrowRight />
        </Button>
      </div>
    );
  }
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
        <div className="mr-auto">
          <p className="text-sm font-semibold">{entries.length === 1 ? '1 suggestion to check' : `${entries.length} suggestions to check`}</p>
          <p className="text-xs text-muted-foreground">Accept all leaves low-confidence fields and conflicts for you.</p>
        </div>
        <Button size="sm" variant="ghost" className="text-(--ai-foreground)" onClick={() => onSeeResearch(null)}>
          <Sparkles /> See research
        </Button>
        <ActionButton size="sm" disabled={acceptable === 0} action={onAcceptAll}>
          <Check /> Accept all{acceptable > 0 ? ` (${acceptable})` : ''}
        </ActionButton>
      </div>
      <ul className="flex flex-col gap-3" aria-label="Suggestions to confirm">
        {entries.map(entry => {
          const field = entry.field;
          const conflict = conflictActions(entry);
          const hasPrior = field.value?.ai_prior != null;
          const actions: RowActions = {
            onAccept: conflict ? conflict.onUseSuggestion : () => onLooksRight(field),
            onKeepPrevious: conflict ? conflict.onKeepPrevious : hasPrior ? () => onKeepPrevious(field) : undefined,
            onDismiss: hasPrior ? undefined : () => onClear(field),
            onEdit: conflict ? conflict.onEdit : () => onShowField(entry),
            onSeeResearch: () => onSeeResearch(field.definition.key),
          };
          return <ReviewRow key={field.definition.key} entry={entry} conflict={conflict !== undefined} actions={actions} />;
        })}
      </ul>
    </div>
  );
}
