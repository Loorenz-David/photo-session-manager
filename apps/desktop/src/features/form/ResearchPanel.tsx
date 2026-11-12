import { useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { cn } from '@/lib/utils';
import type { FormField } from '@/services/attributes';
import type { ResearchStatus, ResearchStep } from '@/services/research';

/** The verb each step kind is shown with (08A §8). */
export const STEP_VERB: Record<string, string> = {
  looked: 'Looked',
  searched: 'Searched',
  read: 'Read',
  compared: 'Compared',
  concluded: 'Concluded',
  not_determined: 'Not determined',
};

/** The run states whose run is over with an outcome (08A §3). */
const DONE = new Set(['finished', 'stopped_at_cap']);

function plural(n: number, one: string, many: string): string {
  return n === 1 ? `1 ${one}` : `${n} ${many}`;
}

/**
 * "Finished · n fields filled · n to review" (08A §3 mock-up). A run that confirmed values already there wrote nothing,
 * so 0 reads "no fields changed" rather than suggesting research did nothing (R4 review).
 */
export function finishedLine(status: ResearchStatus): string {
  const filled = status.fields_filled === 0 ? 'no fields changed' : plural(status.fields_filled, 'field filled', 'fields filled');
  const capped = status.run?.state === 'stopped_at_cap' ? ' · stopped at the cost limit' : '';
  return `Finished · ${filled} · ${status.to_review} to review${capped}`;
}

type ResearchPanelProps = {
  /** The session's research status (`research_status`); null while research is not available: no state line, no button. */
  status: ResearchStatus | null;
  /** The form's fields: labels for the focus chip, and Accept all's count. */
  fields: FormField[];
  /** A field focused from its popover: the steps that did not lead to it are dimmed. */
  focusKey: string | null;
  onFocus: (key: string | null) => void;
  /** Research / Run again (`research_run`). */
  onRun: () => Promise<unknown>;
  onAcceptAll: () => Promise<unknown>;
  onClose: () => void;
  /**
   * When the wait for Capture One's previews ends (ms since the epoch): until then "Waiting for Capture One" is not said
   * (MC-A6 says it when the wait ends).
   */
  waitEndsAt?: number | null;
};

/** Re-renders at `at` (ms since the epoch) so a time-dependent line appears on its own. */
function useNowPast(at: number | null | undefined): boolean {
  const [past, setPast] = useState(() => !at || Date.now() >= at);
  useEffect(() => {
    if (!at || Date.now() >= at) {
      setPast(true);
      return;
    }
    setPast(false);
    const handle = window.setTimeout(() => setPast(true), at - Date.now());
    return () => window.clearTimeout(handle);
  }, [at]);
  return past;
}

function Step({ step, dimmed }: { step: ResearchStep; dimmed: boolean }) {
  const verb = STEP_VERB[step.kind] ?? step.kind;
  return (
    <li data-step={step.seq} data-kind={step.kind} data-dimmed={dimmed || undefined} className={cn('flex flex-col gap-1 border-l-2 border-(--ai) pl-3', dimmed && 'opacity-35')}>
      <span className="text-xs">
        <span className="font-semibold text-(--ai-foreground)">{verb}</span>
        {step.summary && <> {step.summary}</>}
      </span>
      {step.kind === 'read' && step.source_url && (
        <a href={step.source_url} target="_blank" rel="noreferrer" className="truncate text-xs underline underline-offset-2">
          {step.source_title || step.source_url}
        </a>
      )}
      {step.detail && <span className="text-xs text-muted-foreground">{step.detail}</span>}
    </li>
  );
}

/**
 * The research panel (08A §1, §8; NOTES §Screens 7): the steps of the session's latest run in order, live while it
 * runs; its outcome when it ended; why research is not running when it cannot; Research / Run again; Accept all.
 */
export function ResearchPanel({ status, fields, focusKey, onFocus, onRun, onAcceptAll, onClose, waitEndsAt }: ResearchPanelProps) {
  const suggested = fields.filter(field => field.value?.provenance === 'ai' && field.value.ai_status === 'suggested' && field.value.ai_confidence !== 'low').length;
  const waitOver = useNowPast(waitEndsAt);
  const run = status?.run ?? null;
  const steps = [...(status?.steps ?? [])].sort((a, b) => a.seq - b.seq);
  const reason = status?.reason ?? null;
  const showReason = reason !== null && !!reason.sentence && (reason.code !== 'WaitingForEditor' || waitOver);
  const running = run?.state === 'running';
  const ended = run !== null && run.state !== 'running' && run.state !== 'queued';

  return (
    <aside aria-label="Research" className="flex w-80 shrink-0 flex-col border-l bg-background">
      <header className="flex items-center justify-between border-b px-4 py-3">
        <h3 className="flex items-center gap-2 text-sm font-semibold">
          Research
          {running && (
            <span role="status" data-live className="flex items-center gap-1 text-[11px] font-normal text-(--ai-foreground)">
              <span className="size-1.5 animate-pulse rounded-full bg-(--ai)" aria-hidden /> Researching…
            </span>
          )}
        </h3>
        <Button variant="ghost" size="icon-sm" aria-label="Close research" onClick={onClose}>
          <X />
        </Button>
      </header>
      {focusKey && (
        <button type="button" className="mx-4 mt-3 self-start rounded-full bg-(--ai-muted) px-2.5 py-0.5 text-xs text-(--ai-foreground)" onClick={() => onFocus(null)}>
          Focused on {fields.find(f => f.definition.key === focusKey)?.definition.label ?? focusKey} ×
        </button>
      )}
      {status && (
        <div className="flex flex-col gap-2 border-b px-4 py-3 text-xs" data-research-state>
          {run && DONE.has(run.state) && <p className="font-semibold">{finishedLine(status)}</p>}
          {run?.state === 'failed' && (
            <p className="text-destructive">
              Stopped at {run.stage_reached ?? 'the start'}{run.error ? `: ${run.error}` : ''}
            </p>
          )}
          {showReason && <p className="text-muted-foreground">{reason!.sentence}</p>}
          {status.can_run && (
            <ActionButton size="sm" variant="secondary" className="self-start" action={onRun}>
              {ended ? 'Run again' : 'Research'}
            </ActionButton>
          )}
        </div>
      )}
      <ol className="flex flex-1 flex-col gap-3 overflow-y-auto px-4 py-3" aria-label="Research steps">
        {status && steps.length === 0 && !running && <li className="text-xs text-muted-foreground">No research steps yet.</li>}
        {steps.map(step => (
          <Step key={step.seq} step={step} dimmed={focusKey !== null && !step.field_keys.includes(focusKey)} />
        ))}
      </ol>
      <footer className="border-t px-4 py-3">
        <ActionButton className="w-full" disabled={suggested === 0} action={onAcceptAll}>
          Accept all{suggested > 0 ? ` (${suggested})` : ''}
        </ActionButton>
        <p className="mt-1.5 text-[11px] text-muted-foreground">Low-confidence fields are left for you to check.</p>
      </footer>
    </aside>
  );
}
