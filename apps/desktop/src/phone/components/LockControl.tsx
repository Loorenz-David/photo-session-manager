/** The lock control as the stage drives it (design §Lock control). */
export type LockState = {
  /** The touch origin, in image-local px. */
  x: number;
  y: number;
  /** Slide progress 0…1. */
  p: number;
  closed: boolean;
  /** `false` for the unlock animation: the circle only. */
  track: boolean;
  /** Fading out. */
  out: boolean;
  /** Counts pulses, so each lock or unlock plays the pulse again. */
  pulse: number;
};

const clamp = (value: number, low: number, high: number) => Math.max(low, Math.min(high, value));

const Padlock = ({ closed }: { closed: boolean }) => <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round">
  <rect x="5" y="11" width="14" height="10" rx="2" />
  <path d={closed ? 'M8 11V8a4 4 0 0 1 8 0v3' : 'M8 11V8a4 4 0 0 1 7.6-1.8'} />
</svg>;

/**
 * Design §Lock control: a track above the finger that fills with the slide, a ring that follows
 * the finger, and the padlock circle at the top, closing with a pulse when the slide completes.
 */
export function LockControl({ lock }: { lock: LockState }) {
  const length = clamp(lock.y - 78, 0, 118);
  const circleTop = lock.track ? Math.max(8 - lock.y, -length - 70) : -Math.min(60, Math.max(0, lock.y - 38));
  const scale = lock.track && !lock.closed ? 0.85 + lock.p * 0.15 : 1;
  return <div className="ph-lock" data-testid="lock-control" style={{ left: lock.x, top: lock.y, opacity: lock.out ? 0 : 1 }}>
    {lock.track && <>
      <div className="ph-lock-track" style={{ top: -length, height: length }}>
        <div className="ph-lock-fill" data-testid="lock-fill" style={{ height: `${Math.round(lock.p * 100)}%` }} />
      </div>
      <div className="ph-lock-ring" style={{ top: -lock.p * length - 22 }} />
    </>}
    <div key={lock.pulse} className={`ph-lock-circle${lock.pulse ? ' ph-lock-pulse' : ''}${lock.closed ? ' ph-lock-closed' : ''}`}
      style={{ left: lock.track && length < 30 ? 30 : -30, top: circleTop, transform: `scale(${scale})` }}>
      <Padlock closed={lock.closed} />
    </div>
  </div>;
}
