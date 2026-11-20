import { useEffect, useState } from 'react';
import { cameraFtpAddress, cameraStatus, type CameraStatus } from '@/services/camera';
import { onCameraStatus } from '@/services/events';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { CameraCard } from './CameraCard';

/** How often the header re-reads whether this Mac's address changed (P68). */
export const ADDRESS_CHECK_MS = 5000;

export const CAMERA_HELP = 'Camera disconnected';

/** What the camera is busy with, in words (the header label and Settings → Camera's check share them). */
export function busyWords(reason: string): string {
  return ({ exposure: 'Taking a photo', transfer: 'Receiving a photo', switching: 'Switching connection' } as Record<string, string>)[reason] ?? 'Camera busy';
}

/**
 * A28: the camera's state as Settings → Camera's "Test camera connection" says it: connected (with the camera's name and
 * the link), connecting, busy, the action it needs (the backend's sentence), or not found.
 */
export function cameraCheckLine(status: CameraStatus): string {
  const link = (value: string | null | undefined) => (value === 'wireless' ? 'on Wi-Fi' : 'on the cable');
  switch (status.kind) {
    case 'connected': return `Connected: ${status.name} ${link(status.link)}`;
    case 'connecting': return `Connecting ${link(status.link)}…`;
    case 'busy': return busyWords(status.reason);
    case 'needs_action': return status.message;
    default: return 'No camera found. Check that the camera is on and the cable is plugged in, or that Wi-Fi is set up.';
  }
}

export function CameraDot() {
  const [status, setStatus] = useState<CameraStatus | null>(null);
  // P68: "This Mac's address changed to … — set Host Name on the camera" (the backend's sentence).
  const [addressWarning, setAddressWarning] = useState<string | null>(null);
  useEffect(() => {
    let alive = true;
    const read = () => { Promise.resolve().then(() => cameraFtpAddress()).then(value => { if (alive) setAddressWarning(value.warning); }).catch(() => undefined); };
    read();
    const timer = window.setInterval(read, ADDRESS_CHECK_MS);
    return () => { alive = false; window.clearInterval(timer); };
  }, []);
  useEffect(() => {
    let alive = true;
    let unlisten: (() => void) | undefined;
    cameraStatus().then(next => { if (alive) setStatus(next); }).catch(() => { if (alive) setStatus({kind: 'disconnected'}); });
    onCameraStatus(next => { if (alive) setStatus(next); }).then(off => { if (alive) unlisten = off; else off(); }).catch(() => undefined);
    return () => { alive = false; unlisten?.(); };
  }, []);
  const short = (text: string) => text.length > 48 ? `${text.slice(0, 48)}…` : text;
  const label = addressWarning && status?.kind !== 'busy' ? short(addressWarning) : !status ? 'Checking the camera…' : status.kind === 'connected' ? `${status.name} · ${status.link === 'wireless' ? 'Wi-Fi' : 'Cable'}`
    : status.kind === 'connecting' ? `Connecting · ${status.link === 'wireless' ? 'Wi-Fi' : 'Cable'}`
    : status.kind === 'busy' ? busyWords(status.reason)
    : status.kind === 'needs_action' ? short(status.message)
    : 'Camera disconnected';
  const title = addressWarning && status?.kind !== 'busy' ? addressWarning : status?.kind === 'connected' ? `Camera connected by ${status.link === 'wireless' ? 'Wi-Fi' : 'cable'}` : label;
  return <Popover modal={false}><PopoverTrigger asChild><button type="button" className="camera-button" data-state={status?.kind ?? 'unknown'} title={title} aria-label={label}>
    <span className="camera-light" data-testid="camera-light" aria-hidden /><span className="camera-text">{label}</span>
  </button></PopoverTrigger><PopoverContent align="end" className="w-80">{status && <CameraCard status={status} onStatus={setStatus} notice={addressWarning} />}</PopoverContent></Popover>;
}
