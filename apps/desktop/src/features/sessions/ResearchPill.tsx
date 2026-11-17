import { useEffect, useRef, useState } from 'react';
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
  if (!research) return null;
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
