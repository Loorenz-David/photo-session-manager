import { useEffect, useState } from 'react';

/** Design §1: the overlay fades for .4 s once connected, then leaves the page. */
const FADE_MS = 400;

type Props = { connected: boolean; macName: string | null; itemName: string | null };

/** Design §1: full-screen white while `hello` and the session are not both answered; blocks every gesture. */
export function ConnectingOverlay({ connected, macName, itemName }: Props) {
  const [gone, setGone] = useState(false);
  useEffect(() => {
    if (!connected) {
      setGone(false);
      return;
    }
    const timer = setTimeout(() => setGone(true), FADE_MS);
    return () => clearTimeout(timer);
  }, [connected]);
  if (gone) return null;
  const line = [macName, itemName].filter(Boolean).join(' · ').toUpperCase();
  return <div className="ph-connecting" data-testid="connecting" style={{ opacity: connected ? 0 : 1, pointerEvents: connected ? 'none' : 'auto' }}>
    <span className="ph-connecting-title">Connecting to session</span>
    <span className="ph-mono ph-connecting-line">{line}</span>
    <div className="ph-loader" role="progressbar" aria-label="Connecting"><div className="ph-loader-bar" /></div>
  </div>;
}
