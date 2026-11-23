import { toast } from 'sonner';
import { Checkbox } from '@/components/ui/checkbox';
import { activityLabel } from '@/lib/format';
import { labelsOpen, type LabelRow as LabelRowData, type LabelsAccess, type SessionLabel } from '@/services/labels';
import type { ConversationSessionBadge } from '@/services/conversations';
import type { SessionResearch } from '@/services/research';
import { SessionBadge } from '@/features/conversations/SessionBadge';
import { SessionActionsMenu } from '@/features/sessions/DeleteSession';
import { HoverPreview, Thumb } from '@/features/sessions/HoverPreview';
import { ResearchPill, WriterPill } from '@/features/sessions/ResearchPill';
import type { Pictures } from '@/features/sessions/usePictures';
import { LabelPill } from './LabelPill';
import { formatPrice, markerText } from './labelWords';

type LabelRowProps = {
  row: LabelRowData;
  now: number;
  selected: boolean;
  onToggle: () => void;
  navigate: (sessionId: number) => void;
  pictures: Pictures;
  research?: SessionResearch;
  conversations?: ConversationSessionBadge;
  labelsAccess: LabelsAccess | null;
  /** After "Delete session" of a matched row's session. */
  onChanged: () => void;
};

/**
 * L2 (09C §5 Rows, MC-L1.9): one Labels row. With a session on this Mac it looks and opens like that session (its name,
 * SKU, picture, usual pills and ⋯); without one it shows Pricing's title and picture and opening it adopts the shop
 * product first. Both show the price, the row's own marker (pill and Label column), when it was created, and a tick when
 * the product has a SKU.
 */
export function LabelRow({ row, now, selected, onToggle, navigate, pictures, research, conversations, labelsAccess, onChanged }: LabelRowProps) {
  const session = row.session;
  const label: SessionLabel = { session_id: session?.id ?? 0, product_id: row.product_id, marker: row.marker };
  const name = session ? session.item_type || session.title : row.title;
  const sku = session ? session.sku : row.sku;
  const picture = session
    ? pictures.cover(session.id) ?? session.cover_preview_path
    : (row.commerce_id ? pictures.search(row.commerce_id) : undefined) ?? row.picture_path;

  function open() {
    if (session) {
      navigate(session.id);
      return;
    }
    labelsOpen({ product_id: row.product_id }).then(
      opened => navigate(opened.session_id),
      (error: Error) => toast.error(error.message),
    );
  }

  const main = (
    <button type="button" className="row-main" onClick={open}>
      <Thumb path={picture} className="row-thumb" />
      <span className="row-name">
        <strong>{name}</strong>
        <small>{sku}</small>
        <span className="row-pills">
          {session?.arranged && <span className="pill">Arranged</span>}
          {session && <ResearchPill research={research} />}
          {session && <WriterPill research={research} />}
          {session && <SessionBadge badge={conversations} />}
          <LabelPill label={label} now={now} />
        </span>
      </span>
    </button>
  );

  return (
    <div className="session-row label-row" data-testid={`label-row-${row.product_id}`} data-selected={selected || undefined}>
      <div className="row-check">
        {row.sku && <Checkbox checked={selected} onCheckedChange={onToggle} aria-label={`Select ${name}`} />}
      </div>
      {session ? <HoverPreview row={session} picture={picture}>{main}</HoverPreview> : main}
      <span className="row-price">{formatPrice(row.price_minor)}</span>
      <span className="row-label">{markerText(row.marker, now)}</span>
      <span className="row-time">{row.created_at == null ? '' : activityLabel(row.created_at, now)}</span>
      <span className="row-menu">
        {session && <SessionActionsMenu sessionId={session.id} onDeleted={onChanged} label={label} labelsAccess={labelsAccess} />}
      </span>
    </div>
  );
}
