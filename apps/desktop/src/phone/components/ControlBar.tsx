import { useState } from 'react';
import { buzz, HAPTIC } from '../gestures';

type Props = {
  /** Long shots turn the viewport; the menu and next buttons turn with it. */
  rotated: boolean;
  next: { enabled: boolean; label: string };
  onMenu: () => void;
  onCapture: () => void;
  onNext: () => void;
};

/** Design §2 control bar: menu · shutter · next. It never moves, whatever the image's shape. */
export function ControlBar({ rotated, next, onMenu, onCapture, onNext }: Props) {
  const [pressing, setPressing] = useState(false);
  const turn = { transform: `rotate(${rotated ? 90 : 0}deg)` };
  return <div className="ph-controls">
    <button type="button" className="ph-round ph-menu" aria-label="Item" style={turn} onClick={onMenu}>
      <span /><span /><span />
    </button>
    <div className="ph-shutter" role="button" aria-label="Capture"
      onPointerDown={() => {
        buzz(HAPTIC.shutterPress);
        setPressing(true);
      }}
      onPointerUp={() => {
        if (!pressing) return;
        setPressing(false);
        onCapture();
      }}
      // Design: sliding off the button cancels.
      onPointerLeave={() => setPressing(false)} onPointerCancel={() => setPressing(false)} onContextMenu={event => event.preventDefault()}>
      <div className="ph-shutter-disc" data-testid="shutter-disc" style={{ transform: `scale(${pressing ? 0.86 : 1})` }} />
    </div>
    <div className="ph-next" style={turn}>
      <button type="button" className="ph-round" aria-label={next.label} data-testid="next" style={{ opacity: next.enabled ? 1 : 0.3 }}
        onClick={() => {
          if (next.enabled) onNext();
        }}>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12h14M13 6l6 6-6 6" /></svg>
      </button>
      <span className="ph-next-label">{next.label}</span>
    </div>
  </div>;
}
