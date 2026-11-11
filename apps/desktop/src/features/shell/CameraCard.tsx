import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { FingerprintBox } from '@/features/settings/FingerprintBox';
import { cameraCapture, cameraFilesNotReceivedDismiss, cameraReconnect, cameraRetry, cameraTakeBack, cameraTakeBackAvailability, type CameraStatus } from '@/services/camera';

export function takeBackError(error: unknown): string {
  const e = error as { code?: string; data?: Record<string, string>; message?: string };
  if (e.code === 'take_back_blocked') return `Can't take the camera back yet: ${e.data?.reason}`;
  if (e.code === 'take_back_failed') return `Taking the camera back stopped at step ${e.data?.step}: ${e.data?.cause}`;
  return e.message ?? String(error);
}

export function CameraCard({ status, onStatus, notice }: { status: CameraStatus; onStatus: (status: CameraStatus) => void; notice?: string | null }) {
  const [busy, setBusy] = useState(false);
  const [fingerprintOpen, setFingerprintOpen] = useState(false);
  const [blocked, setBlocked] = useState<string | null>('Checking…');
  const action = async (run: () => Promise<unknown>, success?: string) => {
    setBusy(true);
    try { await run(); if (success) toast.success(success); }
    catch (error) { toast.error(takeBackError(error)); }
    finally { setBusy(false); }
  };
  const needs = status.kind === 'needs_action' ? status : null;
  const takeBack = needs?.reason === 'held_by_other_program' && needs.holder === 'Capture One';
  useEffect(() => {
    if (!takeBack) return;
    let active = true;
    const check = () => { void cameraTakeBackAvailability().then(reason => { if (active) setBlocked(reason); }).catch(() => { if (active) setBlocked('Could not check the camera'); }); };
    check();
    const interval = setInterval(check, 1000);
    return () => { active = false; clearInterval(interval); };
  }, [takeBack]);
  const missing = needs?.reason === 'files_not_received';
  // P68: the fingerprint box is reachable from the header when the camera waits for it.
  const fingerprint = needs?.reason === 'fingerprint_unconfirmed' || needs?.reason === 'fingerprint_mismatch';
  return <div className="flex flex-col gap-3">
    {notice && <p role="status">{notice}</p>}
    <p>{needs?.message ?? (status.kind === 'connected' ? `Camera connected by ${status.link === 'wireless' ? 'Wi-Fi' : 'cable'}` : status.kind === 'disconnected' ? 'Camera disconnected' : status.kind === 'connecting' ? 'Connecting…' : 'Camera busy')}</p>
    {busy && takeBack && <p>Taking the camera back… (about 20 s)</p>}
    <div className="flex flex-wrap gap-2">
      <Button disabled={busy || status.kind !== 'connected'} onClick={() => void action(cameraCapture)}>Take photo</Button>
      {takeBack ? <><Button disabled={busy || blocked !== null} onClick={() => void action(async () => {
        const result = await cameraTakeBack();
        toast.success(result.camera_held ? 'Camera back on the cable' : 'Capture One reopened; the camera is still held');
      })}>Take the camera back</Button>{blocked && <p>{blocked}</p>}</> : missing ? <>
        {needs.link === 'wireless' && <Button disabled={busy} onClick={() => void action(cameraReconnect)}>Reconnect</Button>}
        <Button disabled={busy} variant="outline" onClick={() => void action(cameraFilesNotReceivedDismiss)}>Dismiss</Button>
      </> : <Button disabled={busy} onClick={() => void action(async () => onStatus(await cameraRetry()))}>Retry</Button>}
      {fingerprint && <Button variant="outline" onClick={() => setFingerprintOpen(true)}>Confirm the camera&apos;s fingerprint</Button>}
    </div>
    <Dialog open={fingerprintOpen} onOpenChange={setFingerprintOpen}>
      <DialogContent>
        <DialogTitle>Camera fingerprint</DialogTitle>
        <DialogDescription>Check that this is your camera before Session Manager connects to it.</DialogDescription>
        {fingerprintOpen && <FingerprintBox needed changed={needs?.reason === 'fingerprint_mismatch'} onDone={() => setFingerprintOpen(false)} />}
      </DialogContent>
    </Dialog>
  </div>;
}
