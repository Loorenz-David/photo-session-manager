type Props = {
  open: boolean;
  itemName: string;
  macName: string;
  rows: { label: string; value: string }[];
  onClose: () => void;
  onProperties: () => void;
};

/** Design §6 and LV-D17: the item's information and its actions, in a bottom sheet behind the menu button. */
export function ItemDrawer({ open, itemName, macName, rows, onClose, onProperties }: Props) {
  return <>
    <div className="ph-backdrop ph-backdrop-menu" style={{ opacity: open ? 1 : 0, pointerEvents: open ? 'auto' : 'none' }} onClick={onClose} />
    <div className="ph-sheet ph-sheet-menu" role="dialog" aria-label="Item" aria-hidden={!open} data-testid="item-drawer" style={{ transform: `translateY(${open ? 0 : 105}%)` }}>
      <div className="ph-grab" />
      <div className="ph-item-head">
        <div className="ph-item-titles">
          <span className="ph-item-title">{itemName}</span>
          <span className="ph-mono ph-item-connected"><span className="ph-live-dot" />CONNECTED · {macName.toUpperCase()}</span>
        </div>
        <button type="button" className="ph-close" aria-label="Close" onClick={onClose}>×</button>
      </div>
      <dl className="ph-rows">
        {rows.map(row => <div className="ph-row" key={row.label}>
          <dt>{row.label}</dt>
          <dd className="ph-mono">{row.value}</dd>
        </div>)}
      </dl>
      <div className="ph-item-actions">
        <span className="ph-mono ph-section">ACTIONS</span>
        <button type="button" className="ph-button ph-button-solid ph-button-wide" onClick={onProperties}>
          Fill item properties
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#fff" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 6l6 6-6 6" /></svg>
        </button>
      </div>
    </div>
  </>;
}
