import { createContext, useCallback, useContext, useEffect, useRef, useState } from 'react';
import { onConversationsChanged } from '@/services/events';
import type { Visit } from './visit';
import {
  conversationsCounts,
  type ConversationCounts,
  type ConversationListRow,
  type ConversationScope,
} from '@/services/conversations';

/** The selected conversation: its id, and its row when it was picked from the list. */
export type SelectedConversation = { id: string; row?: ConversationListRow };

/**
 * C2 (09B §6, §7): the conversations panel's state, one instance in `App.tsx` for every route (master plan MC-C3.6:
 * it lives outside the route, so it survives a jump). `sessionId` = the session on screen (undefined off a session).
 */
export type ConversationsState = {
  sessionId: number | undefined;
  open: boolean;
  selected: SelectedConversation | null;
  scope: ConversationScope;
  counts: ConversationCounts;
  /** Opens the panel; with an id, on that conversation (a toast's Open). */
  openPanel: (conversationId?: string) => void;
  close: () => void;
  select: (row: ConversationListRow) => void;
  back: () => void;
  /** The footer's "All conversations": the list switches to every session. */
  showAll: () => void;
} & Jump;

/**
 * C4 (09B §7, MC-C3): the jump and the way back, and the quote chip's jump to a field, offered by `App.tsx` (which owns route and visit). `visitBack` is the
 * visit's "← Back" (`back` above already means "back to the list").
 */
export type Jump = {
  /** The visit on screen, or null. */
  visit: Visit | null;
  /** "Open session": that session as a visit (the capture folder does not move). */
  openSession: (sessionId: number) => void;
  visitBack: () => void;
  shootHere: () => void;
  /** The quote chip: opens the details form of the session on screen at that field. */
  openField: (key: string) => void;
};

const NO_JUMP: Jump = {
  visit: null,
  openSession: () => undefined,
  visitBack: () => undefined,
  shootHere: () => undefined,
  openField: () => undefined,
};

/** `null` = no provider (a component rendered alone reads nothing and shows nothing). */
export const ConversationsContext = createContext<ConversationsState | null>(null);

export function useConversationsContext(): ConversationsState | null {
  return useContext(ConversationsContext);
}

function routeScope(sessionId: number | undefined): ConversationScope {
  return sessionId === undefined ? { kind: 'all' } : { kind: 'session', session_id: sessionId };
}

/**
 * Calls `refresh` on every `conversations_changed` while mounted. A missing listener never breaks the page (the same
 * guarded pattern as `useResearchOverview`).
 */
export function useConversationsChanged(refresh: () => void) {
  const latest = useRef(refresh);
  latest.current = refresh;
  useEffect(() => {
    let alive = true;
    let off: (() => void) | undefined;
    Promise.resolve()
      .then(() => onConversationsChanged(() => latest.current()))
      .then(unlisten => (alive ? (off = unlisten) : unlisten()))
      .catch(() => undefined);
    return () => {
      alive = false;
      off?.();
    };
  }, []);
}

const NO_COUNTS: ConversationCounts = { here: 0, elsewhere: 0 };

/** The header count of the session on screen (all sessions off a session), re-read on `conversations_changed`. */
function useCounts(sessionId: number | undefined): ConversationCounts {
  const [counts, setCounts] = useState<ConversationCounts>(NO_COUNTS);
  const latest = useRef(0);
  const load = useCallback(() => {
    const mine = ++latest.current;
    Promise.resolve()
      .then(() => conversationsCounts(sessionId ?? null))
      .then(next => {
        if (mine === latest.current) setCounts(next);
      })
      .catch(() => {
        if (mine === latest.current) setCounts(NO_COUNTS);
      });
  }, [sessionId]);
  useEffect(load, [load]);
  useConversationsChanged(load);
  return counts;
}

export function useConversations(sessionId: number | undefined, jump: Jump = NO_JUMP): ConversationsState {
  const [open, setOpen] = useState(false);
  const [selected, setSelected] = useState<SelectedConversation | null>(null);
  const [scope, setScope] = useState<ConversationScope>(() => routeScope(sessionId));
  const counts = useCounts(sessionId);
  // The list follows the session on screen; "All conversations" lasts until the route changes.
  useEffect(() => setScope(routeScope(sessionId)), [sessionId]);
  const openPanel = useCallback((conversationId?: string) => {
    setOpen(true);
    if (conversationId !== undefined) {
      setSelected({ id: conversationId });
      // A toast may name a conversation of another session: the list it returns to holds every one.
      setScope({ kind: 'all' });
    }
  }, []);
  const close = useCallback(() => {
    setOpen(false);
    setSelected(null);
  }, []);
  const select = useCallback((row: ConversationListRow) => setSelected({ id: row.id, row }), []);
  const back = useCallback(() => setSelected(null), []);
  const showAll = useCallback(() => setScope({ kind: 'all' }), []);
  return { sessionId, open, selected, scope, counts, openPanel, close, select, back, showAll, ...jump };
}
