import { Checkbox } from '@/components/ui/checkbox';
import { activityLabel, photoSummary } from '@/lib/format';
import type { SessionList } from '@/services/sessions';
import { HoverPreview, Thumb } from './HoverPreview';
import { StatusPill } from './StatusPill';

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
  now: number;
};

export function SessionRow({ row, selected, onToggle, onOpen, now }: SessionRowProps) {
  const name = row.item_type || row.title;
  return (
    <div className="session-row" data-testid={`session-row-${row.id}`} data-selected={selected || undefined}>
      <div className="row-check">
        {isSelectable(row) && <Checkbox checked={selected} onCheckedChange={onToggle} aria-label={`Select ${name}`} />}
      </div>
      <HoverPreview row={row}>
        <button type="button" className="row-main" onClick={onOpen}>
          <Thumb path={row.cover_preview_path} className="row-thumb" />
          <span className="row-name">
            <strong>{name}</strong>
            <small>{row.sku}</small>
          </span>
        </button>
      </HoverPreview>
      <span className="row-photos">{photoSummary(row.main_count, row.condition_count)}</span>
      <StatusPill label={row.label} />
      <time className="row-time" dateTime={new Date(row.last_activity_at).toISOString()}>
        {activityLabel(row.last_activity_at, now)}
      </time>
    </div>
  );
}
