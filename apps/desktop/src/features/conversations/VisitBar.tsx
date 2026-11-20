import { useCallback, useEffect, useRef, useState } from 'react';
import { Button } from '@/components/ui/button';
import { onSessionsChanged } from '@/services/events';
import { activeSession, listSessions } from '@/services/sessions';
import { useConversationsContext } from './useConversations';
import type { Route, Visit } from './visit';

/** The Unassigned workspace (session 1: no item, no sessions-list row). */
const UNASSIGNED_SESSION = 1;

/** `undefined` = not read yet. */
type Facts = { skus: Map<number, string> | undefined; capture: number | null | undefined };

/**
 * The SKUs of the sessions list and the session the backend reports active (MC-C3.5: the bar shows the backend's
 * truth, not `visit.capture`), read when the bar appears and on every `session_changed`.
 */
function useFacts(): Facts {
  const [facts, setFacts] = useState<Facts>({ skus: undefined, capture: undefined });
  const latest = useRef(0);
  const load = useCallback(() => {
    const mine = ++latest.current;
    Promise.resolve()
      .then(() => activeSession())
      .then(session => mine === latest.current && setFacts(prior => ({ ...prior, capture: session?.id ?? null })))
      .catch(() => undefined);
    Promise.resolve()
      .then(() => listSessions({ filter: 'all', query: null }))
      .then(list => {
        const skus = new Map(list.groups.flatMap(group => group.rows).map(row => [row.id, row.sku] as const));
        if (mine === latest.current) setFacts(prior => ({ ...prior, skus }));
      })
      .catch(() => undefined);
  }, []);
  useEffect(() => {
    load();
    let alive = true;
    let off: (() => void) | undefined;
    Promise.resolve()
      .then(() => onSessionsChanged(load))
      .then(unlisten => (alive ? (off = unlisten) : unlisten()))
      .catch(() => undefined);
    return () => {
      alive = false;
      off?.();
    };
  }, [load]);
  return facts;
}

/** A session's label: "Unassigned" for the Unassigned workspace, else its SKU (empty until its row has loaded). */
function sessionLabel(sessionId: number, skus: Map<number, string> | undefined): string {
  if (sessionId === UNASSIGNED_SESSION) return 'Unassigned';
  return skus?.get(sessionId) ?? '';
}

function originLabel(origin: Route, skus: Map<number, string> | undefined): string {
  if (origin.kind === 'session' && origin.sessionId !== undefined) return sessionLabel(origin.sessionId, skus);
  if (origin.kind === 'settings') return 'Settings';
  if (origin.kind === 'health') return 'Health';
  return 'Sessions';
}

function Bar({ visit, onBack, onShootHere }: { visit: Visit; onBack: () => void; onShootHere: () => void }) {
  const { skus, capture } = useFacts();
  const captureLabel = capture === undefined ? '' : capture === null ? 'Unassigned' : sessionLabel(capture, skus);
  return (
    <div className="visit-bar" role="status" aria-label="Visit">
      <span className="visit-bar-text">
        Visiting {sessionLabel(visit.visited, skus)}: new photos still go to {captureLabel}.
      </span>
      <Button variant="ghost" size="sm" onClick={onBack}>
        ← Back to {originLabel(visit.origin, skus)}
      </Button>
      <Button variant="outline" size="sm" onClick={onShootHere}>
        Shoot here instead
      </Button>
    </div>
  );
}

/** C4 (09B §7.2, MC-C3.5): the slim bar under the header, shown iff a visit exists. */
export function VisitBar() {
  const state = useConversationsContext();
  if (!state || state.visit === null) return null;
  return <Bar visit={state.visit} onBack={state.visitBack} onShootHere={state.shootHere} />;
}
