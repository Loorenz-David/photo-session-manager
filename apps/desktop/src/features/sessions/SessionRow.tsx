import { Checkbox } from '@/components/ui/checkbox';
import { activityLabel, photoSummary } from '@/lib/format';
import type { SessionList } from '@/services/sessions';
import { HoverPreview, Thumb } from './HoverPreview';
import type { SessionResearch } from '@/services/research';
import { ResearchPill, WriterPill } from './ResearchPill';
import { ChangedPill } from './ChangedPill';
import { StatusPill } from './StatusPill';
import { SessionActionsMenu } from './DeleteSession';
import { SessionBadge } from '@/features/conversations/SessionBadge';
import type { ConversationSessionBadge } from '@/services/conversations';
import type { LabelsAccess, SessionLabel } from '@/services/labels';
import { LabelPill } from '@/features/labels/LabelPill';
import { markerText } from '@/features/labels/labelWords';

type Row = SessionList['groups'][number]['rows'][number];

/** Sessions that a batch publish could take (P15): ready, or failed and waiting for a retry. */
export function isSelectable(row: Row): boolean {
  return row.status === 'ready' || row.status === 'failed';
}

type SessionRowProps = {
  row: Row;
  selected: boolean;
  onToggle: () => void;
  onOpen: () => void;
  onPublish?: () => void;
  onRetry?: () => Promise<unknown> | void;
  now: number;
  /** A6: the session's research state (absent: none to show). */
  research?: SessionResearch;
  /** A10: after "Keep as published". */
  onChanged?: () => void;
  /** A17: after "Delete session". */
  onDeleted?: () => void;
  /** C2 (09B §6): the session's conversations badge (absent: no unresolved conversation). */
  conversations?: ConversationSessionBadge;
  /** H2: the row's picture when it differs from the answer's (a cover that arrived since). */
  picture?: string | null;
  /** L1 (MC-L3.6): the session's label marker (absent: no answer yet). */
  label?: SessionLabel;
  /** L1: the labels access of the last markers answer (for the ⋯ entry). */
  labelsAccess?: LabelsAccess | null;
};

export function SessionRow({ row, selected, onToggle, onOpen, onPublish, onRetry, now, research, onChanged, onDeleted, conversations, picture, label, labelsAccess }: SessionRowProps) {
  const name = row.item_type || row.title;
  const shown = picture ?? row.cover_preview_path;
  const marked = markerText(label?.marker, now) !== null;
  return (
    <div className="session-row" data-testid={`session-row-${row.id}`} data-selected={selected || undefined}>
      <div className="row-check">
        {isSelectable(row) && <Checkbox checked={selected} onCheckedChange={onToggle} aria-label={`Select ${name}`} />}
      </div>
      <HoverPreview row={row} picture={shown}>
        <button type="button" className="row-main" onClick={onOpen}>
          <Thumb path={shown} className="row-thumb" />
          <span className="row-name">
            <strong>{name}</strong>
            <small>{row.sku}</small>
            {(row.arranged || research || conversations || marked) && (
              <span className="row-pills">
                {row.arranged && <span className="pill">Arranged</span>}
                <ResearchPill research={research} />
                <WriterPill research={research} />
                <SessionBadge badge={conversations} />
                <LabelPill label={label} now={now} />
              </span>
            )}
          </span>
        </button>
      </HoverPreview>
      <span className="row-photos">{photoSummary(row.main_count, row.condition_count)}</span>
      {row.changes ? (
        <div className="row-status">
          <StatusPill label={row.label} onPublish={onPublish} onRetry={onRetry} sessionId={row.id} />
          <ChangedPill sessionId={row.id} changes={row.changes} onKept={onChanged} />
        </div>
      ) : (
        <StatusPill label={row.label} onPublish={onPublish} onRetry={onRetry} sessionId={row.id} />
      )}
      {/* Changelog 46 (1): the list is in creation order, so the row shows when the session was created. */}
      <time className="row-time" dateTime={new Date(row.created_at).toISOString()}>
        {activityLabel(row.created_at, now)}
      </time>
      <span className="row-menu">{onDeleted && <SessionActionsMenu sessionId={row.id} onDeleted={onDeleted} label={label} labelsAccess={labelsAccess} />}</span>
    </div>
  );
}
