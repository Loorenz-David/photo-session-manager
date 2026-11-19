import { useCallback, useEffect, useRef, useState } from 'react';
import { MessageCircleQuestion, PenLine, X } from 'lucide-react';
import { toast } from 'sonner';
import { ActionButton } from '@/components/ui/action-button';
import { Input } from '@/components/ui/input';
import { writerFindingAnswer, writerFindingDismiss, writerRequest, writerStatus, type WriterFinding, type WriterStatus } from '@/services/writer';

/**
 * W3: the session's writer status (`writer_status`), re-read whenever `refresh` changes (the form passes its research
 * status, which it re-reads on `research_changed` for the session: every writer change emits it) and on demand. Null
 * while the writer is not available (no engine, no session): nothing about the writer is shown.
 */
export function useWriterStatus(sessionId: number | undefined, refresh?: unknown) {
  const [status, setStatus] = useState<WriterStatus | null>(null);
  const latest = useRef(0);
  const reload = useCallback(async () => {
    if (sessionId === undefined) return;
    const mine = ++latest.current;
    try {
      const next = await writerStatus(sessionId);
      if (mine === latest.current) setStatus(next);
    } catch {
      if (mine === latest.current) setStatus(null);
    }
  }, [sessionId]);
  useEffect(() => {
    void reload();
  }, [reload, refresh]);
  return { status, reload };
}

/** The latest writer run is waiting or writing. */
export function writing(status: WriterStatus | null): boolean {
  const state = status?.run?.state;
  return state === 'queued' || state === 'running';
}

/** One open ask (a question or a finding in `ask` mode): its text, a one-line answer and Answer. */
function Ask({ finding, onChanged }: { finding: WriterFinding; onChanged: () => Promise<unknown> }) {
  const [answer, setAnswer] = useState('');
  const send = async () => {
    const requested = await writerFindingAnswer(finding.id, answer.trim());
    // W5·U4: the answer is kept but no writing was queued; say why (as Write texts does).
    if (!requested.queued && requested.reason) toast.error(requested.reason.sentence);
    setAnswer('');
    await onChanged();
  };
  return (
    <li data-writer-ask={finding.id} className="flex flex-col gap-2 rounded-lg border bg-card px-3 py-2.5">
      <p className="flex items-start gap-2 text-sm">
        <MessageCircleQuestion className="mt-0.5 size-4 shrink-0 text-(--ai-foreground)" aria-hidden />
        <span>{finding.text}</span>
      </p>
      <form
        className="flex items-center gap-2"
        onSubmit={event => {
          event.preventDefault();
          if (answer.trim()) void send();
        }}
      >
        <Input aria-label={`Answer: ${finding.text}`} value={answer} onChange={event => setAnswer(event.target.value)} className="h-8" />
        <ActionButton type="button" size="sm" disabled={!answer.trim()} action={send}>
          Answer
        </ActionButton>
      </form>
    </li>
  );
}

/** One open note (a finding in `note` mode): its text and Dismiss. */
function Note({ finding, onChanged }: { finding: WriterFinding; onChanged: () => Promise<unknown> }) {
  return (
    <li data-writer-note={finding.id} className="flex items-start gap-2 rounded-lg border bg-card px-3 py-2.5 text-sm">
      <span className="flex-1 text-muted-foreground">{finding.text}</span>
      <ActionButton
        size="sm"
        variant="ghost"
        action={async () => {
          await writerFindingDismiss(finding.id);
          await onChanged();
        }}
      >
        <X /> Dismiss
      </ActionButton>
    </li>
  );
}

/** A24: why a requested text was not offered, in the owner's words (from the run's write outcome). */
const NOT_OFFERED: Record<string, { why: string; again: boolean }> = {
  'the photographer cleared this field': { why: 'you cleared it', again: true },
  'the photographer accepted this value': { why: 'you accepted the current text', again: true },
  'the photographer wrote this value': { why: 'you wrote the current text', again: true },
  'matches a text you dismissed': { why: 'it matched a text you dismissed', again: true },
  'already holds this text': { why: 'it came out the same as the current text', again: false },
  'no text': { why: 'the writer gave no text', again: true },
};

/** A24: "Description was not rewritten: you cleared it. Press Write texts to get a new one." */
export function notOfferedSentence(label: string, outcome: string): string {
  const known = NOT_OFFERED[outcome];
  const why = known?.why ?? outcome;
  const again = known ? known.again : true;
  return `${label} was not rewritten: ${why}.${again ? ' Press Write texts to get a new one.' : ''}`;
}

/** A24: the latest writer run failed: the reason in plain words, and Try again (a Write texts request). */
function Failed({ status, sessionId, onChanged }: { status: WriterStatus; sessionId?: number; onChanged: () => Promise<unknown> }) {
  const error = status.run?.error?.trim();
  return (
    <div role="alert" data-writer-failed className="flex items-center gap-3 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2.5 text-sm text-destructive">
      <span className="flex-1">Writing texts failed{error ? `: ${error}` : ''}.</span>
      {sessionId !== undefined && (
        <ActionButton size="sm" variant="secondary" action={async () => {
          const requested = await writerRequest(sessionId, 'write_texts');
          if (requested.queued) toast.success('Writing texts. They arrive in AI review.');
          else if (requested.reason) toast.error(requested.reason.sentence);
          await onChanged();
        }}>
          Try again
        </ActionButton>
      )}
    </div>
  );
}

/**
 * The writer's block at the top of the AI review tab (book 08C §5.5, §5.6, §6): "Writing texts…" while the latest
 * writer run waits or writes; each open question with an answer box (answering writes the texts again, the answer is
 * kept for the item); each open note with Dismiss (a dismissed note never comes back). A24: a failed run says why with
 * Try again; a finished run lists the texts it did not offer and why. Nothing when there is none.
 */
export function WriterBlock({ status, onChanged, sessionId }: { status: WriterStatus | null; onChanged: () => Promise<unknown>; sessionId?: number }) {
  if (!status) return null;
  const asks = status.findings.filter(f => f.mode === 'ask');
  const notes = status.findings.filter(f => f.mode !== 'ask');
  const busy = writing(status);
  const failed = !busy && status.run?.state === 'failed';
  const ended = !busy && (status.run?.state === 'finished' || status.run?.state === 'stopped_at_cap');
  const skipped = ended ? status.not_offered : [];
  if (!busy && !failed && skipped.length === 0 && asks.length === 0 && notes.length === 0) return null;
  return (
    <section aria-label="Writer" data-writer-block className="flex flex-col gap-2">
      {busy && (
        <p role="status" className="flex items-center gap-2 text-sm text-(--ai-foreground)" data-writing>
          <PenLine className="size-4 animate-pulse" aria-hidden /> Writing texts…
        </p>
      )}
      {failed && <Failed status={status} sessionId={sessionId} onChanged={onChanged} />}
      {skipped.length > 0 && (
        <ul className="flex flex-col gap-1 text-sm text-muted-foreground" aria-label="Texts not rewritten">
          {skipped.map(text => (
            <li key={text.key} data-writer-not-offered={text.key}>{notOfferedSentence(text.label, text.outcome)}</li>
          ))}
        </ul>
      )}
      {asks.length > 0 && (
        <ul className="flex flex-col gap-2" aria-label="Writer questions">
          {asks.map(finding => <Ask key={finding.id} finding={finding} onChanged={onChanged} />)}
        </ul>
      )}
      {notes.length > 0 && (
        <ul className="flex flex-col gap-2" aria-label="Writer notes">
          {notes.map(finding => <Note key={finding.id} finding={finding} onChanged={onChanged} />)}
        </ul>
      )}
    </section>
  );
}
