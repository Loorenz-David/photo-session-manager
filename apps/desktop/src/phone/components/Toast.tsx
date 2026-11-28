import { useEffect, useState } from 'react';

/** Design §7: a toast hides by itself after 1.4 s. */
const TOAST_MS = 1400;

/** Design §7: the #111 pill under the top edge. Lock and unlock never use it (they have their own control). */
export function Toast({ toast }: { toast: { text: string; n: number } | null }) {
  const [shown, setShown] = useState(false);
  useEffect(() => {
    if (!toast) return;
    setShown(true);
    const timer = setTimeout(() => setShown(false), TOAST_MS);
    return () => clearTimeout(timer);
  }, [toast]);
  return <div className="ph-toast-row">
    <div className="ph-toast" role="status" data-testid="toast" style={{ opacity: shown ? 1 : 0, transform: `translateY(${shown ? 0 : -8}px)` }}>{toast?.text ?? ''}</div>
  </div>;
}
