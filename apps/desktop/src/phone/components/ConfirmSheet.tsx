type Props = {
  open: boolean;
  /** The question. */
  text: string;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
};

/**
 * A question with two answers, as a bottom sheet (the design's sheet motion and scrim). Book 10J
 * LV-D20 (1): a template shot's photo leaves its shot for the extra Main photos only after "Move";
 * the backdrop is Cancel.
 */
export function ConfirmSheet({ open, text, confirmLabel, onConfirm, onCancel }: Props) {
  return <>
    <div className="ph-backdrop ph-backdrop-confirm" data-testid="confirm-backdrop" style={{ opacity: open ? 1 : 0, pointerEvents: open ? 'auto' : 'none' }} onClick={onCancel} />
    {open && <div className="ph-sheet ph-sheet-confirm" role="dialog" aria-label={text} data-testid="confirm-sheet">
      <div className="ph-grab" />
      <span className="ph-confirm-text">{text}</span>
      <div className="ph-preview-pair">
        <button type="button" className="ph-button ph-button-outline" onClick={onCancel}>Cancel</button>
        <button type="button" className="ph-button ph-button-solid" onClick={onConfirm}>{confirmLabel}</button>
      </div>
    </div>}
  </>;
}
