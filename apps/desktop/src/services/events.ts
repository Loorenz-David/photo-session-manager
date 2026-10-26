import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import type { CameraStatus } from './camera';

/** Calls `refresh` whenever a session or its photos change (`session_changed`, `photo_added`). */
export async function onSessionsChanged(refresh: () => void): Promise<UnlistenFn> {
  const unlisteners = await Promise.all(['session_changed', 'photo_added', 'publish_progress'].map(name => listen(name, () => refresh())));
  return () => unlisteners.forEach(unlisten => unlisten());
}

/** Delivers every `camera_status_changed` payload (the new status). */
export async function onCameraStatus(handler: (status: CameraStatus) => void): Promise<UnlistenFn> {
  return listen<CameraStatus>('camera_status_changed', event => handler(event.payload));
}

export type PhotoEvent = { photo_id: number; session_id: number };
export type WorkspaceEvent =
  | { kind: 'photo_added'; photo_id: number; session_id: number }
  | { kind: 'photo_changed'; photo_id: number; session_id: number }
  | { kind: 'session_changed'; session_id: number }
  | { kind: 'publish_progress'; session_id: number };

/** Workspace changes and publish progress refresh the current session view (master plan §6.7). */
export async function onWorkspaceEvents(handler: (event: WorkspaceEvent) => void): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    listen<PhotoEvent>('photo_added', event => handler({ kind: 'photo_added', ...event.payload })),
    listen<PhotoEvent>('photo_changed', event => handler({ kind: 'photo_changed', ...event.payload })),
    listen<{ session_id: number }>('session_changed', event => handler({ kind: 'session_changed', ...event.payload })),
    listen<{ session_id: number }>('publish_progress', event => handler({ kind: 'publish_progress', ...event.payload })),
  ]);
  return () => unlisteners.forEach(unlisten => unlisten());
}

/** Every `photo_added` payload (used for the arrival toast while the user is elsewhere). */
export async function onPhotoAdded(handler: (event: PhotoEvent) => void): Promise<UnlistenFn> {
  return listen<PhotoEvent>('photo_added', event => handler(event.payload));
}

export type FileDropEvent = { type: 'enter' | 'over' | 'leave' } | { type: 'drop'; paths: string[] };

/** Files dragged from Finder onto the window (Tauri webview drag-drop). */
export async function onFileDrop(handler: (event: FileDropEvent) => void): Promise<UnlistenFn> {
  return getCurrentWebview().onDragDropEvent(event => {
    const payload = event.payload;
    if (payload.type === 'drop') handler({ type: 'drop', paths: payload.paths });
    else handler({ type: payload.type });
  });
}
