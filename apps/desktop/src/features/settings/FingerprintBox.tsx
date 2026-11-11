import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { cameraFingerprintConfirm, cameraFingerprintPending, cameraFingerprintReject, cameraStatus } from '@/services/camera';
import { onFingerprintPending } from '@/services/events';

/** The camera fingerprint to compare and confirm (P50/P51; P68: shared by Settings → Camera and the header's
 *  "Confirm the camera's fingerprint"). Shown while a fingerprint waits for confirmation, or when `needed`. */
export function FingerprintBox({ needed = false, changed: changedAtOpen = false, onDone }: { needed?: boolean; changed?: boolean; onDone?: () => void }) {
  const [fingerprint, setFingerprint] = useState<string | null>(null);
  const [changed, setChanged] = useState(changedAtOpen);
  const [needsAction, setNeedsAction] = useState(needed);
  useEffect(() => {
    let alive = true;
    let off: (() => void) | undefined;
    Promise.resolve().then(() => onFingerprintPending(value => { if (alive) { setFingerprint(value.fingerprint); setChanged(value.replace); setNeedsAction(true); } })).then(stop => { if (alive) off = stop; else stop(); }).catch(() => undefined);
    Promise.resolve().then(cameraStatus).then(status => {
      if (!alive || status.kind !== 'needs_action') return;
      if (status.reason === 'fingerprint_mismatch' || status.reason === 'fingerprint_unconfirmed') {
        setChanged(status.reason === 'fingerprint_mismatch');
        setNeedsAction(true);
      }
    }).catch(() => undefined);
    // P68: a fingerprint reported before this box opened is shown too.
    Promise.resolve().then(() => cameraFingerprintPending()).then(pending => {
      if (!alive || !pending) return;
      setFingerprint(current => current ?? pending.fingerprint);
      setChanged(pending.replace);
      setNeedsAction(true);
    }).catch(() => undefined);
    return () => { alive = false; off?.(); };
  }, []);
  const run = async (task: () => Promise<void>) => {
    try { await task(); setFingerprint(null); setNeedsAction(false); onDone?.(); } catch (error) {
      const e = error as { code?: string; message?: string };
      toast.error(e.code === 'fingerprint_changed' ? 'The camera shows a different fingerprint now. Compare the new one.' : e.message ?? String(error));
    }
  };
  if (!fingerprint && !needsAction) return null;
  return <div className="rounded border p-3"><h4>{changed ? 'Fingerprint changed' : 'Confirm camera fingerprint'}</h4><p>Compare with the camera: MENU › Network › Network Option › Access Authen. Info</p>
    {fingerprint ? <><code>{fingerprint}</code><div className="flex gap-2">
      <Button type="button" onClick={() => void run(() => cameraFingerprintConfirm(fingerprint))}>Confirm</Button>
      <Button type="button" variant="outline" onClick={() => void run(cameraFingerprintReject)}>Not mine</Button>
    </div></> : <p>The fingerprint will appear when the camera reports it.</p>}
  </div>;
}
