import { useCallback, useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';
import { Toaster } from './components/ui/sonner';
import { Header } from './features/shell/Header';
import { useTheme } from './features/shell/ThemeToggle';
import { HealthPage } from './features/health/HealthPage';
import { NewSessionDialog } from './features/sessions/NewSessionDialog';
import { SessionsPage } from './features/sessions/SessionsPage';
import { useKeyboard } from './lib/keyboard';
import { activateSession, deactivateSession } from './services/sessions';

/** Routes: `#/` sessions, `#/session/:id` session view, `#/health` health report (hash routing works for the bundled app's file URLs). */
function routeSessionId(): number | undefined {
  const match = window.location.hash.match(/^#\/session\/(\d+)$/);
  return match ? Number(match[1]) : undefined;
}

export function sessionPath(sessionId?: number): string {
  return sessionId === undefined ? '#/' : `#/session/${sessionId}`;
}

function useRoute(): [number | undefined, (sessionId?: number) => void, boolean] {
  const [sessionId, setSessionId] = useState(routeSessionId);
  const [health, setHealth] = useState(() => window.location.hash === '#/health');
  useEffect(() => {
    const follow = () => {
      setSessionId(routeSessionId());
      setHealth(window.location.hash === '#/health');
    };
    window.addEventListener('hashchange', follow);
    return () => window.removeEventListener('hashchange', follow);
  }, []);
  const navigate = useCallback((next?: number) => {
    window.location.hash = sessionPath(next);
    setSessionId(next);
    setHealth(false);
  }, []);
  return [sessionId, navigate, health];
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

export default function App() {
  const [sessionId, navigate, health] = useRoute();
  const [theme, setTheme] = useTheme();
  const [newSessionOpen, setNewSessionOpen] = useState(false);
  useCaptureTarget(sessionId);

  useKeyboard({ n: () => setNewSessionOpen(true) }, sessionId === undefined && !health && !newSessionOpen);

  // Health report from P00, kept reachable until the health screen of P16.
  if (health) return <HealthPage />;

  return (
    <>
      <Header
        sessionId={sessionId}
        theme={theme}
        onThemeChange={setTheme}
        onNewSession={() => setNewSessionOpen(true)}
        goHome={() => navigate()}
      />
      {sessionId === undefined ? (
        <SessionsPage navigate={navigate} />
      ) : (
        <main className="workspace-placeholder">
          <h1>Ready for the first photo</h1>
          <p>Capture and review arrive with the workspace.</p>
        </main>
      )}
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
