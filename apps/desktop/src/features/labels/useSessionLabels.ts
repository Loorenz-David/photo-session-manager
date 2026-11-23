import { useEffect, useRef, useState } from 'react';
import { onLabelsChanged } from '@/services/events';
import { labelsStatus, type LabelsAccess, type SessionLabel } from '@/services/labels';

export type SessionLabels = {
  /** The last answer's labels by session id. */
  labels: Map<number, SessionLabel>;
  /** The last answer's labels access (`null` before any answer). */
  access: LabelsAccess | null;
};

const EMPTY: SessionLabels = { labels: new Map(), access: null };

/**
 * MC-L3.7: the markers of the sessions on screen. Asks `labels_status(ids, refresh)` each time `answer` changes (a page
 * answer, or the session opened), never delaying the rows; `refresh` is true when the id list differs from this mount's
 * last refreshing call (a reload of the same page asks no provider). On `labels_changed` it asks once more without
 * refresh, or with refresh when the access changed. A failed call leaves the markers as they were; nothing here throws
 * into the page. `refreshOnOpen` (the session page) asks with refresh on every `answer`, so a reopen of the same session
 * still asks its provider (MC-L3.7).
 */
export function useSessionLabels(ids: number[] | null, answer: unknown, refreshOnOpen = false): SessionLabels {
  const [state, setState] = useState<SessionLabels>(EMPTY);
  const shown = useRef<string | null>(null);
  const shownIds = useRef<number[]>([]);
  const lastRefreshed = useRef<string | null>(null);

  function ask(sessionIds: number[], refresh: boolean) {
    const key = sessionIds.join(',');
    if (refresh) lastRefreshed.current = key;
    try {
      labelsStatus({ session_ids: sessionIds, refresh }).then(
        got => {
          if (shown.current !== key) return;
          setState({ labels: new Map(got.labels.map(label => [label.session_id, label])), access: got.access });
        },
        () => undefined,
      );
    } catch {
      // The markers stay as they were.
    }
  }
  const latestAsk = useRef(ask);
  latestAsk.current = ask;

  useEffect(() => {
    if (!ids || ids.length === 0) return;
    const key = ids.join(',');
    if (shown.current !== key) setState(EMPTY);
    shown.current = key;
    shownIds.current = ids;
    latestAsk.current(ids, refreshOnOpen || key !== lastRefreshed.current);
    // Only a new answer asks again.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [answer]);

  useEffect(() => {
    let alive = true;
    let unlisten: (() => void) | undefined;
    try {
      onLabelsChanged(event => {
        if (shownIds.current.length > 0) latestAsk.current(shownIds.current, event.access);
      })
        .then(off => (alive ? (unlisten = off) : off()))
        .catch(() => undefined);
    } catch {
      // No listener: the markers follow the next page answer.
    }
    return () => {
      alive = false;
      unlisten?.();
    };
  }, []);

  return state;
}
