import { useEffect, useState } from 'react';
import { ArrowLeft, ClipboardList, CloudUpload, Info, Tag } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { listSessions } from '@/services/sessions';
import { CameraDot } from './CameraDot';
import { NotificationsBell } from './NotificationsBell';
import { ThemeToggle, type Theme } from './ThemeToggle';

type HeaderProps = {
  sessionId?: number;
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  onNewSession: () => void;
  goHome: () => void;
  /** Unassigned workspace: "Assign product" replaces Publish (ch06 §Unassigned). */
  onAssignProduct?: () => void;
};

type SessionIdentity = { name: string; sku: string };

/** Item type (or title until a type is set) and SKU of the open session, read from the sessions list. */
function useSessionIdentity(sessionId?: number): SessionIdentity | null {
  const [identity, setIdentity] = useState<SessionIdentity | null>(null);
  useEffect(() => {
    setIdentity(null);
    if (!sessionId || sessionId === 1) return;
    let alive = true;
    listSessions({ filter: 'all', query: null })
      .then(list => {
        const row = list.groups.flatMap(group => group.rows).find(candidate => candidate.id === sessionId);
        if (alive && row) setIdentity({ name: row.item_type || row.title, sku: row.sku });
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [sessionId]);
  return identity;
}

/** App header (COMPONENTS.md §App shell): Sessions page or session view. */
export function Header({ sessionId, theme, onThemeChange, onNewSession, goHome, onAssignProduct }: HeaderProps) {
  const identity = useSessionIdentity(sessionId);

  if (sessionId === undefined) {
    return (
      <header className="app-header">
        <h1 className="app-title">Session Manager</h1>
        <div className="header-actions">
          <CameraDot />
          <NotificationsBell />
          <ThemeToggle theme={theme} onChange={onThemeChange} />
          <Button className="new-session-button" onClick={onNewSession} title="New session (N)">
            New session
          </Button>
        </div>
      </header>
    );
  }

  return (
    <header className="app-header">
      <div className="header-identity">
        <Button variant="ghost" onClick={goHome} className="back-button">
          <ArrowLeft /> Sessions
        </Button>
        <span className="header-divider" aria-hidden />
        {sessionId === 1 ? (
          <>
            <strong className="header-name">Unassigned photos</strong>
            <span className="header-tag">No product</span>
          </>
        ) : (
          <>
            <strong className="header-name">{identity?.name ?? ''}</strong>
            <span className="header-sku">{identity?.sku ?? ''}</span>
            <Info className="header-info" aria-hidden />
          </>
        )}
      </div>
      <div className="header-actions">
        <CameraDot />
        <span className="header-divider" aria-hidden />
        <NotificationsBell />
        {sessionId === 1 ? (
          <Button onClick={onAssignProduct}>
            <Tag /> Assign product
          </Button>
        ) : (
          <>
            <Button variant="outline" disabled title="The details form arrives in a later phase">
              <ClipboardList /> Details
            </Button>
            <Button disabled title="Publishing arrives in a later phase">
              <CloudUpload /> Publish
            </Button>
          </>
        )}
      </div>
    </header>
  );
}
