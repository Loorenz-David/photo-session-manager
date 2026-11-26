type Props = {
  /** The finger, in image-local px, and the image's width. */
  at: { x: number; y: number; w: number };
  percent: number;
  /** "TEMPLATE" on the main screen, "PHOTO" in the preview. */
  label: 'TEMPLATE' | 'PHOTO';
};

const clamp = (value: number, low: number, high: number) => Math.max(low, Math.min(high, value));

/** Design §Overlay bar: 180 px wide, centred on the finger (100 px from the edges), 96 px above it (56 px from the top). */
export function OverlayBar({ at, percent, label }: Props) {
  return <div className="ph-overlay-bar" data-testid="overlay-bar" style={{ left: clamp(at.x, 100, Math.max(100, at.w - 100)), top: Math.max(56, at.y - 96) }}>
    <span className="ph-overlay-pill">{label} {percent}%</span>
    <div className="ph-overlay-track"><div className="ph-overlay-fill" style={{ width: `${percent}%` }} /></div>
  </div>;
}
