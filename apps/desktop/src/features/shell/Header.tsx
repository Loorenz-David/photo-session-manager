import { useEffect, useState } from 'react';
import { ArrowRight, Bell, ClipboardList, CloudUpload, FolderDown, Info, Settings, Tag } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Hint } from '@/components/ui/tooltip';
import { ActionButton } from '@/components/ui/action-button';
import { integrationKind, onExportProgress, openExportFolder, runExport } from '@/services/export';
import { arrangeNext, listSessions } from '@/services/sessions';
import { CameraDot } from './CameraDot';
import { ThemeToggle, type Theme } from './ThemeToggle';
import { PublishPopover } from '@/features/publish/PublishPopover';
import { PublishProgress } from '@/features/publish/PublishProgress';
import { NotificationsDrawer } from '@/features/publish/NotificationsDrawer';
import { usePublish } from '@/features/publish/usePublish';
import { PageHeader } from './PageHeader';
import { HistoryButtons } from '@/features/history/HistoryButtons';
import { DiskWarning } from '@/features/settings/StorageSection';
import { ResearchPill, WriterPill, useResearchOverview } from '@/features/sessions/ResearchPill';
import { ChangedPill } from '@/features/sessions/ChangedPill';
import { SessionActionsMenu } from '@/features/sessions/DeleteSession';
import { requestTexts, showsNextToReview } from '@/features/sessions/reviewQueue';
import { onSessionsChanged } from '@/services/events';
import type { PublishedChanges } from '@/services/sessions';

type HeaderProps = {
  sessionId?: number;
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  onNewSession: () => void;
  /** Unassigned workspace: "Assign product" replaces Publish (ch06 §Unassigned). */
  onAssignProduct?: () => void;
  onDetails?: () => void;
  /** A11: "AI · n to review" opens the details form on its AI review tab. */
  onReview?: () => void;
  /** A19: the review queue: how many other sessions wait for review, and opening the next one. */
  reviewQueue?: { left: number; onNext: () => void };
};

type SessionIdentity = { name: string; sku: string; arranged: boolean; archived: boolean; changes: PublishedChanges | null };

/** Item type (or title until a type is set) and SKU of the open session, read from the sessions list. */
function useSessionIdentity(sessionId?: number): [SessionIdentity | null, () => void] {
  const [identity, setIdentity] = useState<SessionIdentity | null>(null);
  const [version, setVersion] = useState(0);
  useEffect(() => {
    setIdentity(null);
  }, [sessionId]);
  useEffect(() => {
    if (!sessionId || sessionId === 1) return;
    let alive = true;
    listSessions({ filter: 'all', query: null })
      .then(list => {
        const row = list.groups.flatMap(group => group.rows).find(candidate => candidate.id === sessionId);
        if (alive && row) setIdentity({ name: row.item_type || row.title, sku: row.sku, arranged: row.arranged, archived: row.label.kind === 'archived', changes: row.changes ?? null });
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [sessionId, version]);
  // A10: "Changed after publishing" follows the session live (a late photo, an undo).
  useEffect(() => {
    if (!sessionId || sessionId === 1) return;
    let unlisten: (() => void) | undefined;
    let alive = true;
    onSessionsChanged(() => setVersion(v => v + 1))
      .then(stop => {
        if (alive) unlisten = stop;
        else stop();
      })
      .catch(() => undefined);
    return () => {
      alive = false;
      unlisten?.();
    };
  }, [sessionId]);
  return [identity, () => setVersion(v => v + 1)];
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

/** A refused export: `needs_review` carries the backend's own naming of the photos ("Re-frame first: Main 02", MC-27 §Revisions). */
function exportFailureText(error: unknown): string {
  const message = (error as Error).message;
  return (error as { code?: string }).code === 'needs_review' ? message : `Export failed: ${message}`;
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
export function Header({ sessionId, theme, onThemeChange, onNewSession, onAssignProduct, onDetails, onReview, reviewQueue }: HeaderProps) {
  const [identity, reloadIdentity] = useSessionIdentity(sessionId);
  // A6: the open session's research state beside its name; a tap opens Details (the form and its research).
  const research = useResearchOverview(sessionId !== undefined && sessionId !== 1 ? sessionId : undefined);
  const integration = useIntegration();
  const exporting = useExportToasts();
  const publish = usePublish(sessionId);
  const [publishOpen, setPublishOpen] = useState(false);
  const [notificationsOpen, setNotificationsOpen] = useState(false);
  const failedCount = publish.failedCount;
  const bell = <Button variant="ghost" size="icon" aria-label="Notifications" onClick={() => setNotificationsOpen(true)} className="relative"><Bell />{failedCount > 0 && <span className="bell-count">{failedCount}</span>}</Button>;
  const drawer = notificationsOpen && <NotificationsDrawer notifications={publish.notifications} onClose={() => setNotificationsOpen(false)} onOpen={id => { window.location.hash = `#/session/${id}`; setNotificationsOpen(false); }} />;

  if (sessionId === undefined) {
    return (
      <PageHeader title={<h1 className="app-title">Session Manager</h1>}>
          <DiskWarning />
          <CameraDot />
          {bell}{drawer}
          <Button variant="ghost" size="icon" aria-label="Settings" onClick={() => { window.location.hash = '#/settings'; }}><Settings /></Button>
          <ThemeToggle theme={theme} onChange={onThemeChange} />
          <Button className="new-session-button" onClick={onNewSession} title="New session (N)">
            New session
          </Button>
      </PageHeader>
    );
  }

  return (
    <PageHeader title={<>
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
            <ResearchPill research={research.get(sessionId)} onClick={research.get(sessionId)?.state === 'to_review' && onReview ? onReview : onDetails} />
            <WriterPill research={research.get(sessionId)} />
            <ChangedPill sessionId={sessionId} changes={identity?.changes} onKept={reloadIdentity} />
          </>
        )}
      </>}>
        <DiskWarning />
        <CameraDot />
        <span className="header-divider" aria-hidden />
        <HistoryButtons sessionId={sessionId} />
        {bell}{drawer}
        <Button variant="ghost" size="icon" aria-label="Settings" onClick={() => { window.location.hash = '#/settings'; }}><Settings /></Button>
        {sessionId === 1 ? (
          <ActionButton action={async () => { onAssignProduct?.(); }}>
            <Tag /> Assign product
          </ActionButton>
        ) : (
          <>
            <Hint text="Details (D)"><Button variant="outline" onClick={onDetails}><ClipboardList /> Details</Button></Hint>
            {identity && !identity.archived && (reviewQueue && showsNextToReview(identity.arranged, research.get(sessionId), reviewQueue.left) ? (
              // A19: reviewing — an arranged session whose research has finished, while others wait for review.
              // W3 (WC3): the writer drafts this session's texts while the owner moves on.
              <Button onClick={() => { requestTexts(sessionId, 'next_to_review'); reviewQueue.onNext(); }} title="Open the next session whose AI suggestions wait for review">
                Next to review ({reviewQueue.left} left) <ArrowRight />
              </Button>
            ) : (
              <ActionButton action={async () => {
                try {
                  const done = await arrangeNext(sessionId);
                  // A19 (owner 2026-11-18): back to the Sessions page, the New session field ready to type.
                  window.location.hash = '#/';
                  onNewSession();
                  toast.success(done.research_queued ? 'Arranged · AI research queued' : 'Arranged');
                } catch (error) {
                  toast.error((error as Error).message);
                }
              }}>{identity.arranged ? 'Arranged ✓ — next' : 'Arranged — next'}</ActionButton>
            ))}
            {integration === null && (
              <ActionButton action={() => runExport(sessionId)} onError={error => toast.error(exportFailureText(error), { id: EXPORT_TOAST_ID })} disabled={exporting.has(sessionId)} title="Export the main and condition photos to the export folder">
                <FolderDown /> Export
              </ActionButton>
            )}
            {publish.label?.kind === 'publishing' && <PublishProgress value={publish.label.value} sessionId={sessionId} />}
            {typeof integration === 'string' && <div className="relative"><ActionButton action={async () => { setPublishOpen(true); }}><CloudUpload /> Publish</ActionButton>{publishOpen && <PublishPopover sessionIds={[sessionId]} onClose={() => { setPublishOpen(false); void publish.refresh(); }} />}</div>}
            {/* A17: "Delete session…" (Session Manager only); afterwards the sessions list. */}
            <SessionActionsMenu sessionId={sessionId} className="header-menu-button" onDeleted={() => { window.location.hash = '#/'; }} />
          </>
        )}
    </PageHeader>
  );
}
