import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { cameraCapture, cameraFilesNotReceivedDismiss, cameraReconnect, cameraRetry, cameraTakeBack, cameraTakeBackAvailability, type CameraStatus } from '@/services/camera';

export function takeBackError(error: unknown): string {
  const e = error as { code?: string; data?: Record<string, string>; message?: string };
  if (e.code === 'take_back_blocked') return `Can't take the camera back yet: ${e.data?.reason}`;
  if (e.code === 'take_back_failed') return `Taking the camera back stopped at step ${e.data?.step}: ${e.data?.cause}`;
  return e.message ?? String(error);
}

export function CameraCard({ status, onStatus }: { status: CameraStatus; onStatus: (status: CameraStatus) => void }) {
  const [busy, setBusy] = useState(false);
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
  return <div className="flex flex-col gap-3">
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
    </div>
  </div>;
}
