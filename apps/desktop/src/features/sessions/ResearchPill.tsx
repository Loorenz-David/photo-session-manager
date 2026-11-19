import { useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';
import { onSessionsChanged } from '@/services/events';
import { researchOverview, type SessionResearch } from '@/services/research';

/** "1st", "2nd", "3rd", "4th"… */
export function ordinal(n: number): string {
  const tens = n % 100;
  if (tens >= 11 && tens <= 13) return `${n}th`;
  return `${n}${({ 1: 'st', 2: 'nd', 3: 'rd' } as Record<number, string>)[n % 10] ?? 'th'}`;
}

/** The short reason a blocked session shows (the full sentence is its tooltip). */
const SHORT_REASON: Record<string, string> = {
  NoIntegration: 'no shop connected',
  NoModelKey: 'no model key',
  NoSearchKey: 'no search key',
  PriceMissing: 'a price is missing',
  NothingToResearch: 'nothing to fill',
  NoFieldChosen: 'no fields chosen',
  DayLimit: 'day limit reached',
  Offline: 'offline',
};

/** The words of a session's research state (A6). */
export function researchLabel(research: SessionResearch): string {
  switch (research.state) {
    case 'researching':
      return 'AI researching…';
    case 'waiting':
      return research.position ? `AI waiting · ${ordinal(research.position)} in line` : 'AI waiting';
    case 'to_review':
      return `AI · ${research.to_review} to review`;
    case 'done':
      return 'AI done';
    case 'failed':
      return 'AI stopped';
    case 'blocked':
      return `AI can't run · ${SHORT_REASON[research.reason?.code ?? ''] ?? 'see research'}`;
    default:
      return 'AI';
  }
}

/**
 * A6 (owner 2026-11-17): a session's research state beside "Arranged" in the sessions list and in the session header:
 * researching, waiting (place in line), n to review, done, stopped, or why it cannot run.
 */
export function ResearchPill({ research, onClick }: { research: SessionResearch | undefined; onClick?: () => void }) {
  // A24: a session listed only for its writer (no research run) has no research words.
  if (!research || research.state === 'none') return null;
  const label = researchLabel(research);
  const title = research.state === 'blocked' ? research.reason?.sentence : research.state === 'waiting' ? 'Research runs one session at a time' : undefined;
  const content = (
    <>
      {research.state === 'researching' && <span className="research-pill-dot" aria-hidden />}
      {label}
    </>
  );
  const className = 'pill research-pill';
  if (onClick) {
    return (
      <button type="button" className={className} data-research-state={research.state} title={title} onClick={onClick}>
        {content}
      </button>
    );
  }
  return (
    <span className={className} data-research-state={research.state} title={title}>
      {content}
    </span>
  );
}

/**
 * A24 (owner 2026-11-19): the writer at work or stopped on a failure, beside the research state in the sessions list and
 * the session header. Nothing once the texts are written (they wait in AI review).
 */
export function WriterPill({ research }: { research: SessionResearch | undefined }) {
  const state = research?.writer?.state;
  if (state === 'writing') {
    return (
      <span className="pill research-pill" data-writer-state="writing" title="The writer is drafting the texts. They arrive in AI review.">
        <span className="research-pill-dot" aria-hidden />
        Writing texts…
      </span>
    );
  }
  if (state === 'failed') {
    return (
      <span className="pill research-pill" data-writer-state="failed" title="Open AI review to see why and try again.">
        Writing failed
      </span>
    );
  }
  return null;
}

/** A24: what a finished or failed writer run of `sku` says, once, when the app saw it writing. */
export function writerToast(state: string | undefined, sku: string): { kind: 'success' | 'error'; text: string } | null {
  if (state === 'done' || state === 'stopped') return { kind: 'success', text: `Texts ready for ${sku} — see AI review` };
  if (state === 'failed') return { kind: 'error', text: `Writing texts failed for ${sku} — open AI review to try again` };
  return null;
}

/**
 * A24: one toast when a writer run the app saw writing ends (texts ready, or failed). Reads the same overview the
 * sessions list reads, re-read on the existing change events; never toasts what was already over on load.
 */
export function WriterToasts() {
  const rows = useResearchOverview(null);
  const seen = useRef(new Map<number, string>());
  useEffect(() => {
    for (const row of rows.values()) {
      const now = row.writer?.state;
      const before = seen.current.get(row.session_id);
      if (before === 'writing' && now && now !== 'writing') {
        const said = writerToast(now, row.writer?.sku ?? '');
        if (said?.kind === 'success') toast.success(said.text);
        else if (said) toast.error(said.text);
      }
      if (now) seen.current.set(row.session_id, now);
    }
  }, [rows]);
  return null;
}

/**
 * The research state of every session (or of one), read in one call and re-read whenever sessions or research change.
 * Empty while research is not available. `sessionId`: null = every session, undefined = nothing is read.
 */
export function useResearchOverview(sessionId: number | null | undefined): Map<number, SessionResearch> {
  const [rows, setRows] = useState<Map<number, SessionResearch>>(() => new Map());
  const latest = useRef(0);
  useEffect(() => {
    setRows(new Map());
    if (sessionId === undefined) return;
    let alive = true;
    let off: (() => void) | undefined;
    const load = async () => {
      const mine = ++latest.current;
      try {
        const next = await researchOverview(sessionId);
        if (alive && mine === latest.current) setRows(new Map(next.map(row => [row.session_id, row])));
      } catch {
        if (alive && mine === latest.current) setRows(new Map());
      }
    };
    void load();
    Promise.resolve()
      .then(() => onSessionsChanged(() => void load()))
      .then(unlisten => (alive ? (off = unlisten) : unlisten()))
      .catch(() => undefined);
    return () => {
      alive = false;
      off?.();
    };
  }, [sessionId]);
  return rows;
}
