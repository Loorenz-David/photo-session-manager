import { cn } from '@/lib/utils';
import type { FormGroup, Group } from '@/services/attributes';

export const STEP_LABEL: Record<Group, string> = { details: 'Details', condition: 'Condition', product: 'Product' };

type StepTabsProps = { groups: FormGroup[]; step: Group; onStep: (step: Group) => void };

/** Details / Condition / Product, each with its filled/total count (evidence design_handoff §Form). */
export function StepTabs({ groups, step, onStep }: StepTabsProps) {
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
    </div>
  );
}
