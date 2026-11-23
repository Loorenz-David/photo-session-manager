import { Printer, X } from 'lucide-react';

/**
 * L2 (09C §5 Selection, MC-L1.9): the Labels box's selection bar: `<N> selected` (`All <N> selected` after Select all),
 * `Select all <count>` when offered, `Generate labels for <N>` (greyed out until label generation lands, L3) and the
 * clear button. Nothing when nothing is selected.
 */
export function LabelsBatchBar({
  count,
  all,
  offer,
  onSelectAll,
  onClear,
}: {
  count: number;
  /** The selection is what Select all answered. */
  all: boolean;
  /** The filter's count when Select all is offered, else null. */
  offer: number | null;
  onSelectAll: () => void;
  onClear: () => void;
}) {
  if (count === 0) return null;
  return (
    <div className="batch-bar" role="toolbar" aria-label="Selected products">
      <strong>{all ? `All ${count} selected` : `${count} selected`}</strong>
      {offer !== null && (
        <button type="button" className="batch-link" onClick={onSelectAll}>
          Select all {offer}
        </button>
      )}
      <button type="button" className="batch-primary" disabled>
        <Printer aria-hidden /> Generate labels for {count}
      </button>
      <button type="button" className="batch-close" aria-label="Clear selection" onClick={onClear}>
        <X aria-hidden />
      </button>
    </div>
  );
}
