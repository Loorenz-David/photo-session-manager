import { useCallback, useEffect, useReducer, useRef, useState, type ReactNode } from 'react';
import { toast } from 'sonner';
import { Toaster } from './components/ui/sonner';
import { Header } from './features/shell/Header';
import { useTheme } from './features/shell/ThemeToggle';
import { HealthPage } from './features/health/HealthPage';
import { SettingsPage } from './features/settings/SettingsPage';
import { NewSessionDialog } from './features/sessions/NewSessionDialog';
import { SessionsPage } from './features/sessions/SessionsPage';
import { ArrivalToasts } from './features/shell/ArrivalToasts';
import { WriterToasts } from './features/sessions/ResearchPill';
import { WorkspacePage } from './features/workspace/WorkspacePage';
import { REVIEW_STEP, type Step } from './features/form/StepTabs';
import type { FormFieldJump } from './features/form/FormDialog';
import { useHistory } from './features/history/useHistory';
import { useHistoryKeys } from './features/history/useHistoryKeys';
import { useKeyboard } from './lib/keyboard';
import { leaveThen } from './lib/routes';
export { parentRoute, routeLabel } from './lib/routes';
import { activateSession, activeSession, deactivateSession } from './services/sessions';
import { useReviewFromSearch } from './features/search/useReviewFromSearch';
import { useReviewQueue } from './features/sessions/reviewQueue';
import { ConversationsContext, useConversations } from './features/conversations/useConversations';
import { ConversationsPanel } from './features/conversations/ConversationsPanel';
import { ConversationToasts } from './features/conversations/ConversationToasts';
import { VisitBar } from './features/conversations/VisitBar';
import {
  captureTarget,
  hashOf,
  routeOf,
  routeSession,
  sameRoute,
  sessionRoute,
  visitStep,
  type VisitAction,
  type VisitState,
} from './features/conversations/visit';

/** Routes: `#/` sessions, `#/session/:id` session view, `#/health` health screen, `#/settings` settings (hash routing works for the bundled app's file URLs). */
export function sessionPath(sessionId?: number): string {
  return sessionId === undefined ? '#/' : `#/session/${sessionId}`;
}

type Nav = {
  state: VisitState;
  /** Ordinary navigation (ends a visit; opening a session this way points the camera at it, as before C4). */
  navigate: (sessionId?: number) => void;
  openSession: (sessionId: number) => void;
  visitBack: () => void;
  shootHere: () => void;
};

/**
 * C4 (MC-C3.4): route and visit are one state, changed by `visitStep` only. A step computes the new state from a ref of
 * the current one, writes its hash, then dispatches, so the hash already equals the route when it renders (`PageHeader`
 * reads it during render). The `hashchange` handler reads the hash when it runs: one that only confirms the state's
 * route does nothing; any other is an ordinary navigation (`route-changed`). No hash is written at mount.
 */
function useNav(): Nav {
  const [state, dispatch] = useReducer(visitStep, undefined, () => ({ route: routeOf(window.location.hash), visit: null }));
  const current = useRef<VisitState>(state);
  const step = useCallback((action: VisitAction) => {
    const next = visitStep(current.current, action);
    if (!sameRoute(routeOf(window.location.hash), next.route)) window.location.hash = hashOf(next.route);
    if (next === current.current) return;
    current.current = next;
    dispatch(action);
  }, []);
  useEffect(() => {
    const follow = () => {
      const route = routeOf(window.location.hash);
      if (!sameRoute(route, current.current.route)) step({ type: 'route-changed', route });
    };
    window.addEventListener('hashchange', follow);
    return () => window.removeEventListener('hashchange', follow);
  }, [step]);
  const navigate = useCallback(
    (sessionId?: number) => step({ type: 'route-changed', route: sessionId === undefined ? { kind: 'sessions' } : sessionRoute(sessionId) }),
    [step],
  );
  // The jump asks the page being left first (Settings with unsaved changes), which runs it from Discard / Save.
  const openSession = useCallback(
    (sessionId: number) => leaveThen(sessionPath(sessionId), () => step({ type: 'open-session', sessionId })),
    [step],
  );
  const visitBack = useCallback(() => step({ type: 'back' }), [step]);
  const shootHere = useCallback(() => step({ type: 'shoot-here' }), [step]);
  return { state, navigate, openSession, visitBack, shootHere };
}

/**
 * Opening a session points the editing provider's capture folder at it (ch04 S-B, P04 `session_activate`);
 * back on the Sessions page new captures go to Unassigned (S-E, `session_deactivate`).
 */
function useCaptureTarget(sessionId?: number) {
  const active = useRef(false);
  useEffect(() => {
    if (sessionId !== undefined && sessionId !== 1) {
      activateSession(sessionId)
        .then(() => {
          active.current = true;
        })
        .catch(error => toast.error(`Captures will not reach this session: ${(error as Error).message}`));
      return;
    }
    if (active.current) {
      active.current = false;
      deactivateSession().catch(error => toast.error(`Could not switch captures to Unassigned: ${(error as Error).message}`));
    }
  }, [sessionId]);
}

/**
 * At launch the backend re-applies the capture folder of the session that was active last time (P04 startup), so the
 * window must show that session; otherwise shots would land in a session the screen does not show (ch04 S-E).
 * Reading the stored id (not writing it) leaves nothing to race with the startup re-apply. A route the user has
 * already moved to (a session, the health page) is left alone.
 */
function useResumeActiveSession(navigate: (sessionId?: number) => void) {
  useEffect(() => {
    let cancelled = false;
    activeSession()
      .then(session => {
        const atDefaultRoute = window.location.hash === '' || window.location.hash === '#/';
        if (!cancelled && session && atDefaultRoute) navigate(session.id);
      })
      .catch(error => toast.error(`Could not tell which session captures go to: ${(error as Error).message}`));
    return () => {
      cancelled = true;
    };
  }, [navigate]);
}

export default function App() {
  const { state: nav, navigate, openSession, visitBack, shootHere } = useNav();
  const sessionId = routeSession(nav.route);
  const health = nav.route.kind === 'health';
  const settings = nav.route.kind === 'settings';
  useResumeActiveSession(navigate);
  const [theme, setTheme] = useTheme();
  const [newSessionOpen, setNewSessionOpen] = useState(false);
  const [assignOpen, setAssignOpen] = useState(false);
  const [detailsOpen, setDetailsOpen] = useState(false);
  // A11: the tab the details form opens on; reset whenever the form closes.
  const [detailsStep, setDetailsStep] = useState<Step | null>(null);
  // C4 (09B §7 quote chip jump): the field the details form opens at; `n` counts the requests so each one jumps.
  const [detailsField, setDetailsField] = useState<FormFieldJump | null>(null);
  const changeDetailsOpen = useCallback((open: boolean) => {
    setDetailsOpen(open);
    if (!open) {
      setDetailsStep(null);
      setDetailsField(null);
    }
  }, []);
  const openField = useCallback((key: string) => {
    setDetailsField(current => ({ key, n: (current?.n ?? 0) + 1 }));
    setDetailsOpen(true);
  }, []);
  useEffect(() => { changeDetailsOpen(false); }, [sessionId, changeDetailsOpen]);
  // A14: "Open AI review" in a search window (declared after the reset above, so it opens after it).
  useReviewFromSearch(sessionId, navigate, useCallback(() => { setDetailsStep(REVIEW_STEP); setDetailsOpen(true); }, []));
  // C4 (MC-C3.2): a visit keeps the capture folder where it was; only this value moves it.
  useCaptureTarget(captureTarget(nav));
  useHistory(sessionId);
  useHistoryKeys(sessionId);
  // C2 (09B §6, §7, §9): one conversations state for every route (header icon, panel, toasts).
  const conversations = useConversations(sessionId, { visit: nav.visit, openSession, visitBack, shootHere, openField });
  const withConversations = (page: ReactNode) => (
    <ConversationsContext.Provider value={conversations}>
      {page}
      <ConversationsPanel />
      <ConversationToasts />
    </ConversationsContext.Provider>
  );

  useKeyboard({ n: () => setNewSessionOpen(true) }, sessionId === undefined && !health && !settings && !newSessionOpen);

  // A19: the review queue (sessions whose AI suggestions wait, in the sessions list's order).
  const queue = useReviewQueue(sessionId);
  const nextToReview = queue.next;
  const goNextToReview = useCallback(() => {
    changeDetailsOpen(false);
    if (nextToReview !== null) {
      navigate(nextToReview);
    } else {
      navigate(undefined);
      toast.success('All reviewed');
    }
  }, [changeDetailsOpen, navigate, nextToReview]);

  // Health (ch12) and settings screens: plain pages reached from the header gear.
  if (health) return withConversations(<><HealthPage /><Toaster theme={theme} position="bottom-right" visibleToasts={3} closeButton /></>);
  if (settings) return withConversations(<><SettingsPage /><Toaster theme={theme} position="bottom-right" visibleToasts={3} closeButton /></>);

  return withConversations(
    <>
      <Header
        sessionId={sessionId}
        theme={theme}
        onThemeChange={setTheme}
        onNewSession={() => setNewSessionOpen(true)}
        onAssignProduct={() => setAssignOpen(true)}
        onDetails={() => { setDetailsStep(null); setDetailsOpen(true); }}
        onReview={() => { setDetailsStep(REVIEW_STEP); setDetailsOpen(true); }}
        reviewQueue={{ left: queue.left, onNext: goNextToReview }}
      />
      <VisitBar />
      {sessionId === undefined ? (
        <SessionsPage navigate={navigate} />
      ) : (
        <WorkspacePage
          key={sessionId}
          sessionId={sessionId}
          assignOpen={assignOpen}
          onAssignOpenChange={setAssignOpen}
          detailsOpen={detailsOpen}
          onDetailsOpenChange={changeDetailsOpen}
          detailsStep={detailsStep}
          detailsField={detailsField}
          navigate={navigate}
          reviewQueue={{ left: queue.left, onNext: goNextToReview }}
        />
      )}
      <ArrivalToasts sessionId={sessionId} navigate={navigate} />
      <WriterToasts />
      <NewSessionDialog
        open={newSessionOpen}
        onOpenChange={setNewSessionOpen}
        onOpened={id => {
          setNewSessionOpen(false);
          navigate(id);
        }}
      />
      <Toaster theme={theme} position="bottom-right" visibleToasts={3} closeButton />
    </>
  );
}
