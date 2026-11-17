import { useEffect, useState } from 'react';
import { CircleCheck, CircleHelp, Eye, FileText, Scale, Search, TriangleAlert, X, type LucideIcon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { cn } from '@/lib/utils';
import { ExternalLink } from '@/components/ExternalLink';
import { ordinal } from '@/features/sessions/ResearchPill';
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

/**
 * How each step looks (owner correction A7, 2026-11-17): the kinds were one colour and hard to tell apart. Each look
 * has its own icon, label and rail (`.research-step[data-look]` in globals.css), so it never relies on colour alone.
 * Gathering steps (looked, searched) are quiet, reading and comparing carry the AI colour, a conclusion stands out
 * most, and a page that could not be read is a muted warning.
 */
export type StepLook = 'gather' | 'read' | 'unread' | 'compare' | 'conclude' | 'undetermined';
export const STEP_LOOK: Record<StepLook, { icon: LucideIcon; label?: string }> = {
  gather: { icon: Search },
  read: { icon: FileText },
  unread: { icon: TriangleAlert, label: 'Could not read' },
  compare: { icon: Scale },
  conclude: { icon: CircleCheck },
  undetermined: { icon: CircleHelp },
};

export function stepLook(step: Pick<ResearchStep, 'kind' | 'unread'>): StepLook {
  if (step.kind === 'read') return step.unread ? 'unread' : 'read';
  if (step.kind === 'compared') return 'compare';
  if (step.kind === 'concluded') return 'conclude';
  if (step.kind === 'not_determined') return 'undetermined';
  return 'gather';
}

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

/**
 * A6 (owner 2026-11-17): a queued run says it is waiting, behind which session and where in the line; research runs
 * one session at a time (MC-A6).
 */
export function waitingLine(queue: NonNullable<ResearchStatus['queue']>): string {
  const place = `${ordinal(queue.position)} in line`;
  return queue.running_name ? `Waiting — researching ${queue.running_name} first (${place})` : `Waiting to start (${place})`;
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
  const look = stepLook(step);
  const { label } = STEP_LOOK[look];
  const Icon = step.kind === 'looked' ? Eye : STEP_LOOK[look].icon;
  const verb = label ?? STEP_VERB[step.kind] ?? step.kind;
  return (
    <li data-step={step.seq} data-kind={step.kind} data-look={look} data-dimmed={dimmed || undefined} className={cn('research-step', dimmed && 'opacity-35')}>
      <span className="research-step-head">
        <Icon className="research-step-icon" aria-hidden />
        <span className="research-step-verb">{verb}</span>
        {step.summary && !step.unread && <>{' '}<span className="research-step-summary">{step.summary}</span></>}
      </span>
      {step.kind === 'read' && step.source_url && (
        <ExternalLink href={step.source_url} className="truncate text-xs">
          {step.source_title || step.source_url}
        </ExternalLink>
      )}
      {step.detail && <span className="research-step-detail">{step.detail}</span>}
    </li>
  );
}

/**
 * The research panel (08A §1, §8; NOTES §Screens 7): the steps of the session's latest run in order, live while it
 * runs; its outcome when it ended; why research is not running when it cannot; Research / Run again; Accept all.
 */
export function ResearchPanel({ status, fields, focusKey, onFocus, onRun, onAcceptAll, onClose }: ResearchPanelProps) {
  // What Accept all accepts (MC-A12, R-IA1): suggestions that are neither low nor a conflict.
  const suggested = fields.filter(field => field.value?.provenance === 'ai' && field.value.ai_status === 'suggested' && field.value.ai_confidence !== 'low' && !field.value.conflict).length;
  // MC-A6: "Waiting for Capture One" is said once the run's wait for the previews has ended (the backend's end time).
  const waitOver = useNowPast(status?.wait_ends_at);
  const run = status?.run ?? null;
  const steps = [...(status?.steps ?? [])].sort((a, b) => a.seq - b.seq);
  const reason = status?.reason ?? null;
  const showReason = reason !== null && !!reason.sentence && (reason.code !== 'WaitingForEditor' || waitOver);
  const running = run?.state === 'running';
  const ended = run !== null && run.state !== 'running' && run.state !== 'queued';
  const queued = run?.state === 'queued';

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
          {queued && status.queue && <p className="font-semibold" data-waiting>{waitingLine(status.queue)}</p>}
          {queued && status.queue?.manual && <p className="text-muted-foreground">You asked for it, so it goes before the sessions marked arranged.</p>}
          {showReason && <p className="text-muted-foreground">{reason!.sentence}</p>}
          {!run && !reason && status.can_run && <p className="text-muted-foreground">Research starts when you press Arranged — next.</p>}
          {status.can_run && !(queued && status.queue?.manual) && (
            <ActionButton size="sm" variant="secondary" className="self-start" action={onRun}>
              {ended ? 'Run again' : queued ? 'Research this next' : 'Research'}
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
