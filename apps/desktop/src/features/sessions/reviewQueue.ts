import { useEffect, useMemo, useRef, useState } from 'react';
import { onSessionsChanged } from '@/services/events';
import type { SessionResearch } from '@/services/research';
import { listSessions } from '@/services/sessions';
import { writerRequest, type WriterOrigin } from '@/services/writer';
import { useResearchOverview } from './ResearchPill';

/**
 * W3 (WC3, A19): asks the writer for the session's texts from an automatic origin ("Next to review", "All checked").
 * Fire and forget: a refusal or a failure is never shown and never holds up the owner's next step.
 */
export function requestTexts(sessionId: number, origin: Exclude<WriterOrigin, 'write_texts'>): void {
  try {
    void Promise.resolve(writerRequest(sessionId, origin)).catch(() => undefined);
  } catch {
    // Never shown, never in the way.
  }
}

/** A19: whether a session's research finished with AI suggestions waiting (the research pill's "n to review"). */
export function waitingForReview(research: SessionResearch | undefined): boolean {
  return research?.state === 'to_review';
}

/**
 * A19 (owner 2026-11-18): the session "Next to review" opens: the first session after `current` in the sessions list's
 * order whose suggestions wait, wrapping round to the top; never `current` itself. Null when none waits.
 */
export function nextToReview(order: readonly number[], waiting: (id: number) => boolean, current?: number): number | null {
  const others = order.filter(id => id !== current && waiting(id));
  if (others.length === 0) return null;
  const at = current === undefined ? -1 : order.indexOf(current);
  if (at < 0) return others[0];
  return others.find(id => order.indexOf(id) > at) ?? others[0];
}

/**
 * When the session header shows "Next to review (n left)" in place of "Arranged — next" (A19): the session is arranged,
 * its research has finished (suggestions waiting or all checked), and another session waits for review. Otherwise
 * the header keeps "Arranged — next".
 */
export function showsNextToReview(arranged: boolean, research: SessionResearch | undefined, othersWaiting: number): boolean {
  return arranged && (research?.state === 'to_review' || research?.state === 'done') && othersWaiting > 0;
}

export type ReviewQueue = {
  /** Sessions waiting for review, in the sessions list's order (the current one included). */
  waiting: number[];
  /** How many other sessions than the current one wait. */
  left: number;
  /** The session "Next to review" opens (null: none waits). */
  next: number | null;
  /** The research state of every session. */
  research: Map<number, SessionResearch>;
};

/** A19: the review queue, in the sessions list's order, kept current as sessions and research change. */
export function useReviewQueue(current?: number): ReviewQueue {
  const research = useResearchOverview(null);
  const [order, setOrder] = useState<number[]>([]);
  const latest = useRef(0);
  useEffect(() => {
    let alive = true;
    let off: (() => void) | undefined;
    const load = async () => {
      const mine = ++latest.current;
      try {
        const list = await listSessions({ filter: 'all', query: null });
        if (alive && mine === latest.current) setOrder(list.groups.flatMap(group => group.rows).map(row => row.id));
      } catch {
        // Keep the last order.
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
  }, []);
  return useMemo(() => {
    const waits = (id: number) => waitingForReview(research.get(id));
    const waiting = order.filter(waits);
    return { waiting, left: waiting.filter(id => id !== current).length, next: nextToReview(order, waits, current), research };
  }, [order, research, current]);
}
