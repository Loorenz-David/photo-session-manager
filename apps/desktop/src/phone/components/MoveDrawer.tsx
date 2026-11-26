import { useEffect, useState } from 'react';
import { exampleUrl, type PhoneSession, previewUrl } from '../api';
import { type MoveTile, moveTiles, type PreviewSubject } from '../state';

type Props = {
  token: string;
  view: PhoneSession;
  /** The image being moved (the open preview's). */
  subject: PreviewSubject;
  open: boolean;
  onClose: () => void;
  onPick: (tile: MoveTile) => void;
};

function Tile({ token, tile, inverted, onPick }: { token: string; tile: MoveTile; inverted: boolean; onPick: (tile: MoveTile) => void }) {
  const source = tile.state === 'THIS IMAGE';
  const end = tile.state === 'ADD AT END';
  return <button type="button" className="ph-move-tile" disabled={source} aria-label={`${tile.tag || tile.name} ${tile.state}`} style={{ opacity: source ? 0.45 : 1 }}
    onClick={() => onPick(tile)}>
    <div className={`ph-move-thumb${end ? ' ph-move-thumb-end' : ''}`} style={{ outline: `2px solid ${source ? '#111' : 'transparent'}`, outlineOffset: '2px' }}>
      {tile.image && <img className="ph-cover ph-move-image" alt="" draggable={false} src={previewUrl(token, tile.image)} />}
      {!tile.image && tile.example !== null && <img className="ph-cover ph-move-image ph-grey" alt="" draggable={false} src={exampleUrl(token, tile.example)} />}
      {end && <svg className="ph-move-plus" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#111" strokeWidth="2" strokeLinecap="round"><path d="M12 5v14M5 12h14" /></svg>}
      {tile.tag && <span className={`ph-tag ph-move-tag${inverted ? ' ph-tag-inverted' : ''}`}>{tile.tag}</span>}
    </div>
    <div className="ph-move-texts">
      <span className="ph-move-name">{tile.name}</span>
      <span className="ph-mono ph-move-state">{tile.state}</span>
    </div>
  </button>;
}

/**
 * Design §5: a bottom sheet listing where the image can go: any Main position (a filled one swaps,
 * card LV8), a new extra at the end, or the Condition images (collapsed until asked, open at once
 * when the image is a condition image).
 */
export function MoveDrawer({ token, view, subject, open, onClose, onPick }: Props) {
  const tiles = moveTiles(view, subject);
  const fromCondition = subject.kind === 'condition';
  const [conditionOpen, setConditionOpen] = useState(fromCondition);
  useEffect(() => {
    if (open) setConditionOpen(fromCondition);
  }, [open, fromCondition]);
  const count = view.condition.length;
  return <>
    <div className="ph-backdrop ph-backdrop-move" style={{ opacity: open ? 1 : 0, pointerEvents: open ? 'auto' : 'none' }} onClick={onClose} />
    <div className="ph-sheet ph-sheet-move" role="dialog" aria-label="Move image" aria-hidden={!open} data-testid="move-drawer" style={{ transform: `translateY(${open ? 0 : 105}%)` }}>
      <div className="ph-grab" />
      <div className="ph-sheet-heading">
        <span className="ph-sheet-title">Move image</span>
        <span className="ph-sheet-sub">Place it in the main sequence or send it to condition images. Picking a position that has an image swaps the two.</span>
      </div>
      <div className="ph-move-group">
        <span className="ph-mono ph-section">MAIN IMAGES</span>
        <div className="ph-move-grid ph-move-grid-main" data-testid="move-main">
          {tiles.main.map(tile => <Tile key={tile.key} token={token} tile={tile} inverted={false} onPick={onPick} />)}
        </div>
      </div>
      <div className="ph-move-group">
        <button type="button" className="ph-condition-row" aria-expanded={conditionOpen} onClick={() => setConditionOpen(!conditionOpen)}>
          <span className="ph-condition-name"><span className="ph-mono ph-condition-badge">C</span>Condition images</span>
          <span className="ph-mono ph-condition-count">
            {count} {count === 1 ? 'IMAGE' : 'IMAGES'}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#111" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"
              style={{ transform: `rotate(${conditionOpen ? 180 : 0}deg)`, transition: 'transform .25s ease' }}><path d="M6 9l6 6 6-6" /></svg>
          </span>
        </button>
        <div className="ph-condition-fold" style={{ gridTemplateRows: conditionOpen ? '1fr' : '0fr' }}>
          <div className="ph-condition-clip" aria-hidden={!conditionOpen}>
            <div className="ph-move-grid ph-move-grid-condition" data-testid="move-condition">
              {tiles.condition.map(tile => <Tile key={tile.key} token={token} tile={tile} inverted onPick={onPick} />)}
            </div>
          </div>
        </div>
      </div>
    </div>
  </>;
}
