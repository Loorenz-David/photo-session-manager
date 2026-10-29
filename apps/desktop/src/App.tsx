import { useCallback, useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';
import { Toaster } from './components/ui/sonner';
import { Header } from './features/shell/Header';
import { useTheme } from './features/shell/ThemeToggle';
import { HealthPage } from './features/health/HealthPage';
import { SettingsPage } from './features/settings/SettingsPage';
import { NewSessionDialog } from './features/sessions/NewSessionDialog';
import { SessionsPage } from './features/sessions/SessionsPage';
import { ArrivalToasts } from './features/shell/ArrivalToasts';
import { WorkspacePage } from './features/workspace/WorkspacePage';
import { useKeyboard } from './lib/keyboard';
import { activateSession, activeSession, deactivateSession } from './services/sessions';

/** Routes: `#/` sessions, `#/session/:id` session view, `#/health` health screen, `#/settings` settings (hash routing works for the bundled app's file URLs). */
function routeSessionId(): number | undefined {
  const match = window.location.hash.match(/^#\/session\/(\d+)$/);
  return match ? Number(match[1]) : undefined;
}

export function sessionPath(sessionId?: number): string {
  return sessionId === undefined ? '#/' : `#/session/${sessionId}`;
}

export function parentRoute(hash: string): string | null {
  if (hash === '' || hash === '#/') return null;
  if (hash === '#/health') return '#/settings';
  return '#/';
}

function useRoute(): [number | undefined, (sessionId?: number) => void, boolean, boolean] {
  const [sessionId, setSessionId] = useState(routeSessionId);
  const [health, setHealth] = useState(() => window.location.hash === '#/health');
  const [settings, setSettings] = useState(() => window.location.hash === '#/settings');
  useEffect(() => {
    const follow = () => {
      setSessionId(routeSessionId());
      setHealth(window.location.hash === '#/health');
      setSettings(window.location.hash === '#/settings');
    };
    window.addEventListener('hashchange', follow);
    return () => window.removeEventListener('hashchange', follow);
  }, []);
  const navigate = useCallback((next?: number) => {
    window.location.hash = sessionPath(next);
    setSessionId(next);
    setHealth(false);
    setSettings(false);
  }, []);
  return [sessionId, navigate, health, settings];
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
  const [sessionId, navigate, health, settings] = useRoute();
  useResumeActiveSession(navigate);
  const [theme, setTheme] = useTheme();
  const [newSessionOpen, setNewSessionOpen] = useState(false);
  const [assignOpen, setAssignOpen] = useState(false);
  useCaptureTarget(sessionId);

  useKeyboard({ n: () => setNewSessionOpen(true) }, sessionId === undefined && !health && !settings && !newSessionOpen);

  // Health (ch12) and settings screens: plain pages reached from the header gear.
  if (health) return <HealthPage />;
  if (settings) return <><SettingsPage /><Toaster theme={theme} position="bottom-right" visibleToasts={3} closeButton /></>;

  return (
    <>
      <Header
        sessionId={sessionId}
        theme={theme}
        onThemeChange={setTheme}
        onNewSession={() => setNewSessionOpen(true)}
        goHome={() => navigate()}
        onAssignProduct={() => setAssignOpen(true)}
      />
      {sessionId === undefined ? (
        <SessionsPage navigate={navigate} />
      ) : (
        <WorkspacePage
          key={sessionId}
          sessionId={sessionId}
          assignOpen={assignOpen}
          onAssignOpenChange={setAssignOpen}
          navigate={navigate}
        />
      )}
      <ArrivalToasts sessionId={sessionId} navigate={navigate} />
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
