import { useEffect, useState } from 'react';
import { ArrowLeft, ClipboardList, CloudUpload, FolderDown, Info, Tag } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { integrationKind, onExportProgress, openExportFolder, runExport } from '@/services/export';
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

export const EXPORT_TOAST_ID = 'export';

/** `settings.integration`: undefined while loading, null when none is configured. */
function useIntegration(): string | null | undefined {
  const [integration, setIntegration] = useState<string | null | undefined>(undefined);
  useEffect(() => {
    let alive = true;
    integrationKind()
      .then(kind => alive && setIntegration(kind))
      .catch(() => alive && setIntegration(null));
    return () => {
      alive = false;
    };
  }, []);
  return integration;
}

function files(n: number): string {
  return n === 1 ? '1 file' : `${n} files`;
}

/**
 * Export progress and outcome as one toast (ch07 §Completion without an integration): "Exporting n of N", then
 * "Exported n files" with "Open folder", or the failure. Returns the sessions with an export in flight.
 */
function useExportToasts(): Set<number> {
  const [running, setRunning] = useState<Set<number>>(() => new Set());
  useEffect(() => {
    let alive = true;
    let stop: (() => void) | undefined;
    const settle = (sessionId: number, isRunning: boolean) =>
      setRunning(prior => {
        const next = new Set(prior);
        if (isRunning) next.add(sessionId);
        else next.delete(sessionId);
        return next;
      });
    onExportProgress(progress => {
      if (progress.state === 'running') {
        settle(progress.session_id, true);
        toast.loading(`Exporting ${progress.done} of ${progress.total}`, { id: EXPORT_TOAST_ID });
      } else if (progress.state === 'done') {
        settle(progress.session_id, false);
        toast.success(`Exported ${files(progress.total)}`, {
          id: EXPORT_TOAST_ID,
          duration: 8000,
          action: {
            label: 'Open folder',
            onClick: () => {
              openExportFolder(progress.session_id).catch(error => toast.error(`Could not open the folder: ${(error as Error).message}`));
            },
          },
        });
      } else {
        settle(progress.session_id, false);
        toast.error('Export failed', { id: EXPORT_TOAST_ID });
      }
    })
      .then(unlisten => {
        if (alive) stop = unlisten;
        else unlisten();
      })
      .catch(() => undefined);
    return () => {
      alive = false;
      stop?.();
    };
  }, []);
  return running;
}

/** App header (COMPONENTS.md §App shell): Sessions page or session view. */
export function Header({ sessionId, theme, onThemeChange, onNewSession, goHome, onAssignProduct }: HeaderProps) {
  const identity = useSessionIdentity(sessionId);
  const integration = useIntegration();
  const exporting = useExportToasts();

  const startExport = (id: number) => {
    runExport(id).catch(error => toast.error(`Export failed: ${(error as Error).message}`, { id: EXPORT_TOAST_ID }));
  };

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
            {integration === null && (
              <Button onClick={() => startExport(sessionId)} disabled={exporting.has(sessionId)} title="Export the main and condition photos to the export folder">
                <FolderDown /> Export
              </Button>
            )}
            {typeof integration === 'string' && (
              <Button disabled title="Publishing arrives in a later phase">
                <CloudUpload /> Publish
              </Button>
            )}
          </>
        )}
      </div>
    </header>
  );
}
