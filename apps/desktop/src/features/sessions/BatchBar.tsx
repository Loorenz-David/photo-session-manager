import { CloudUpload, X } from 'lucide-react';

/** Batch selection bar. Selection works now; "Review & publish" is wired with publishing (P15). */
export function BatchBar({
  count,
  selectableCount,
  onSelectAll,
  onClear,
  onReview,
}: {
  count: number;
  selectableCount: number;
  onSelectAll: () => void;
  onClear: () => void;
  onReview?: () => void;
}) {
  if (count === 0) return null;
  return (
    <div className="batch-bar" role="toolbar" aria-label="Selected sessions">
      <strong>{count} selected</strong>
      {count < selectableCount && (
        <button type="button" className="batch-link" onClick={onSelectAll}>
          Select all {selectableCount}
        </button>
      )}
      <button type="button" className="batch-primary" disabled={!onReview} onClick={onReview}>
        <CloudUpload aria-hidden /> Review &amp; publish {count}
      </button>
      <button type="button" className="batch-close" aria-label="Clear selection" onClick={onClear}>
        <X aria-hidden />
      </button>
    </div>
  );
}
