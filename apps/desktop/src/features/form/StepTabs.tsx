import { Sparkles } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { FormGroup, Group } from '@/services/attributes';

export const STEP_LABEL: Record<Group, string> = { details: 'Details', condition: 'Condition', product: 'Product' };

/** A11 (owner 2026-11-18): the tab gathering every field with an AI suggestion to confirm. */
export const REVIEW_STEP = 'review';
export const REVIEW_LABEL = 'AI review';

/** A form tab: one of the three steps, or the AI review tab. */
export type Step = Group | typeof REVIEW_STEP;

type StepTabsProps = { groups: FormGroup[]; step: Step; onStep: (step: Step) => void; toReview: number };

/**
 * Details / Condition / Product, each with its filled/total count (evidence design_handoff §Form), then AI review with
 * the number of suggestions still to confirm (A11). AI review stays visible at 0 and says why it is empty.
 */
export function StepTabs({ groups, step, onStep, toReview }: StepTabsProps) {
  const reviewSelected = step === REVIEW_STEP;
  return (
    <div role="tablist" aria-label="Steps" className="flex gap-5 border-b px-5">
      {groups.map((group, index) => {
        const selected = group.group === step;
        return (
          <button
            key={group.group}
            type="button"
            role="tab"
            aria-selected={selected}
            className={cn('-mb-px flex items-center gap-1.5 border-b-2 py-2.5 text-sm', selected ? 'border-foreground font-semibold' : 'border-transparent text-muted-foreground')}
            onClick={() => onStep(group.group)}
          >
            <span aria-hidden className={cn('grid size-4.5 place-content-center rounded-full text-[10px]', selected ? 'bg-foreground text-background' : 'bg-muted')}>
              {index + 1}
            </span>
            {STEP_LABEL[group.group]}{' '}
            <span className="text-xs text-muted-foreground">
              {group.filled}/{group.total}
            </span>
          </button>
        );
      })}
      <button
        type="button"
        role="tab"
        aria-selected={reviewSelected}
        data-review-tab
        className={cn('-mb-px ml-auto flex items-center gap-1.5 border-b-2 py-2.5 text-sm', reviewSelected ? 'border-(--ai-foreground) font-semibold text-(--ai-foreground)' : 'border-transparent text-muted-foreground')}
        onClick={() => onStep(REVIEW_STEP)}
      >
        <Sparkles className="size-3.5" aria-hidden />
        {REVIEW_LABEL}{' '}
        <span
          data-review-count={toReview}
          aria-label={toReview === 1 ? '1 to confirm' : `${toReview} to confirm`}
          className={cn('rounded-full px-1.5 text-[11px] font-semibold', toReview > 0 ? 'bg-(--ai-muted) text-(--ai-foreground)' : 'bg-muted text-muted-foreground')}
        >
          {toReview}
        </span>
      </button>
    </div>
  );
}
