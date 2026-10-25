import { X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import type { FormField } from '@/services/attributes';
import { ConfidenceBars, CONFIDENCE_BARS } from './AiPill';
import { displayValue } from './AiPopover';

type ResearchPanelProps = {
  /** Fields whose stored value has provenance `ai`, in form order. */
  fields: FormField[];
  /** A field focused from its popover: the steps that did not lead to it are dimmed. */
  focusKey: string | null;
  onFocus: (key: string | null) => void;
  onAcceptAll: () => void;
  onClose: () => void;
};

/**
 * The research panel (NOTES §Screens 7): a static timeline built from the stored provenance, one step per AI
 * field with what it read (`ai_sources`), the reason and the value it set. No agent runs in the MVP.
 */
export function ResearchPanel({ fields, focusKey, onFocus, onAcceptAll, onClose }: ResearchPanelProps) {
  const suggested = fields.filter(field => field.value?.ai_status === 'suggested' && field.value.ai_confidence !== 'low').length;
  return (
    <aside aria-label="Research" className="flex w-80 shrink-0 flex-col border-l bg-background">
      <header className="flex items-center justify-between border-b px-4 py-3">
        <h3 className="text-sm font-semibold">Research</h3>
        <Button variant="ghost" size="icon-sm" aria-label="Close research" onClick={onClose}>
          <X />
        </Button>
      </header>
      {focusKey && (
        <button type="button" className="mx-4 mt-3 self-start rounded-full bg-(--ai-muted) px-2.5 py-0.5 text-xs text-(--ai-foreground)" onClick={() => onFocus(null)}>
          Focused on {fields.find(f => f.definition.key === focusKey)?.definition.label ?? focusKey} ×
        </button>
      )}
      <ol className="flex flex-1 flex-col gap-3 overflow-y-auto px-4 py-3">
        {fields.length === 0 && <li className="text-xs text-muted-foreground">No AI research for this item.</li>}
        {fields.map((field, index) => {
          const value = field.value!;
          const dimmed = focusKey !== null && focusKey !== field.definition.key;
          return (
            <li key={field.definition.key} data-step={field.definition.key} data-dimmed={dimmed || undefined} className={cn('flex flex-col gap-1 border-l-2 border-(--ai) pl-3', dimmed && 'opacity-35')}>
              <span className="text-[11px] text-muted-foreground">Step {index + 1}</span>
              {(value.ai_sources ?? []).map(source => (
                <span key={source} className="truncate text-xs">
                  Read <a href={source} target="_blank" rel="noreferrer" className="underline underline-offset-2">{source}</a>
                </span>
              ))}
              {value.ai_reason && <span className="text-xs text-muted-foreground">{value.ai_reason}</span>}
              <span className="flex items-center gap-1.5 text-xs font-semibold text-(--ai-foreground)">
                Set {field.definition.label}: {displayValue(value)} <ConfidenceBars filled={CONFIDENCE_BARS[value.ai_confidence ?? ''] ?? 1} />
              </span>
            </li>
          );
        })}
      </ol>
      <footer className="border-t px-4 py-3">
        <Button className="w-full" disabled={suggested === 0} onClick={onAcceptAll}>
          Accept all{suggested > 0 ? ` (${suggested})` : ''}
        </Button>
        <p className="mt-1.5 text-[11px] text-muted-foreground">Low-confidence fields are left for you to check.</p>
      </footer>
    </aside>
  );
}
