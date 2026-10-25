import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { CameraStatus } from './camera';

/** Calls `refresh` whenever a session or its photos change (`session_changed`, `photo_added`). */
export async function onSessionsChanged(refresh: () => void): Promise<UnlistenFn> {
  const unlisteners = await Promise.all(['session_changed', 'photo_added'].map(name => listen(name, () => refresh())));
  return () => unlisteners.forEach(unlisten => unlisten());
}

/** Delivers every `camera_status_changed` payload (the new status). */
export async function onCameraStatus(handler: (status: CameraStatus) => void): Promise<UnlistenFn> {
  return listen<CameraStatus>('camera_status_changed', event => handler(event.payload));
}
