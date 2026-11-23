import { Printer, X } from 'lucide-react';

/**
 * L2 (09C §5 Selection, MC-L1.9): the Labels box's selection bar: `<N> selected` (`All <N> selected` after Select all),
 * `Select all <count>` when offered, `Generate labels for <N>` and the clear button. Nothing when nothing is selected.
 * L3 (09C §6): `Generate labels for <N>` opens the label dialog for the selection.
 */
export function LabelsBatchBar({
  count,
  all,
  offer,
  onSelectAll,
  onClear,
  onGenerate,
}: {
  count: number;
  /** The selection is what Select all answered. */
  all: boolean;
  /** The filter's count when Select all is offered, else null. */
  offer: number | null;
  onSelectAll: () => void;
  onClear: () => void;
  /** L3: opens the label dialog with the selected products. */
  onGenerate: () => void;
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
      <button type="button" className="batch-primary" onClick={onGenerate}>
        <Printer aria-hidden /> Generate labels for {count}
      </button>
      <button type="button" className="batch-close" aria-label="Clear selection" onClick={onClear}>
        <X aria-hidden />
      </button>
    </div>
  );
}
