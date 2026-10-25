import { useEffect, useState } from 'react';
import { cameraRetry, cameraStatus, type CameraStatus } from '@/services/camera';
import { onCameraStatus } from '@/services/events';

export const CAMERA_HELP =
  'Capture One does not see a camera. Check that the camera is on and connected by USB in PC Remote (USB mode), ' +
  'and that Imaging Edge is closed, since it holds the camera. Click to retry.';

/** Header camera status (ch04 S-A): green when Capture One reports a camera, red otherwise; click retries. */
export function CameraDot() {
  const [status, setStatus] = useState<CameraStatus | null>(null);
  const [retrying, setRetrying] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);

  useEffect(() => {
    let alive = true;
    let unlisten: (() => void) | undefined;
    cameraStatus()
      .then(next => alive && setStatus(next))
      .catch(() => alive && setStatus({ kind: 'disconnected' }));
    onCameraStatus(next => alive && setStatus(next))
      .then(off => (alive ? (unlisten = off) : off()))
      .catch(() => undefined);
    return () => {
      alive = false;
      unlisten?.();
    };
  }, []);

  async function retry() {
    if (retrying) return;
    setRetrying(true);
    try {
      setStatus(await cameraRetry());
    } catch {
      setStatus({ kind: 'disconnected' });
    } finally {
      setRetrying(false);
    }
  }

  const connected = status?.kind === 'connected';
  const unknown = status === null;
  const title = connected
    ? `Camera connected: ${status.name}`
    : unknown
      ? 'Checking the camera…'
      : 'Camera disconnected. Click to retry.';

  return (
    <div className="camera-dot" onMouseEnter={() => setHelpOpen(true)} onMouseLeave={() => setHelpOpen(false)}>
      <button
        type="button"
        className="camera-button"
        data-state={connected ? 'connected' : unknown ? 'unknown' : 'disconnected'}
        title={title}
        aria-label={title}
        onClick={() => void retry()}
        onFocus={() => setHelpOpen(true)}
        onBlur={() => setHelpOpen(false)}
      >
        <span className="camera-light" data-testid="camera-light" aria-hidden />
        <span className="camera-text">{retrying ? 'Retrying…' : connected ? status.name : unknown ? '' : 'Camera disconnected'}</span>
      </button>
      {!connected && !unknown && helpOpen && (
        <div role="tooltip" className="camera-help">
          {CAMERA_HELP}
        </div>
      )}
    </div>
  );
}
