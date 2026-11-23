import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import type { CameraStatus } from './camera';

/** Calls `refresh` whenever a session or its photos change (`session_changed`, `photo_added`). */
export async function onSessionsChanged(refresh: () => void): Promise<UnlistenFn> {
  const unlisteners = await Promise.all(['session_changed', 'photo_added', 'publish_progress', 'research_changed'].map(name => listen(name, () => refresh())));
  return () => unlisteners.forEach(unlisten => unlisten());
}

/** `research_changed {session_id}`: a research run of that session changed (a step, its state, its writes). */
export async function onResearchChanged(handler: (sessionId: number) => void): Promise<UnlistenFn> {
  return listen<{ session_id: number }>('research_changed', event => handler(event.payload.session_id));
}

/** Delivers every `camera_status_changed` payload (the new status). */
export async function onCameraStatus(handler: (status: CameraStatus) => void): Promise<UnlistenFn> {
  return listen<CameraStatus>('camera_status_changed', event => handler(event.payload));
}

export async function onFingerprintPending(handler: (event: {fingerprint: string; replace: boolean}) => void): Promise<UnlistenFn> {
  return listen<{fingerprint: string; replace: boolean}>('camera_fingerprint_pending', event => handler(event.payload));
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

/** C1: `conversations_changed {toasts, more}`: the conversations copy, a read mark or the Pricing connection changed. */
export type ConversationsChanged = { toasts: { conversation_id: string; text: string }[]; more: number };
export async function onConversationsChanged(handler: (event: ConversationsChanged) => void): Promise<UnlistenFn> {
  return listen<ConversationsChanged>('conversations_changed', event => handler(event.payload));
}

/**
 * L1: `labels_changed {product_ids, access}`: the label state of those products changed (the markers are read again
 * without asking Pricing), or the labels access changed (`access: true`: read again, asking Pricing).
 */
export type LabelsChanged = { product_ids: string[]; access: boolean };
export async function onLabelsChanged(handler: (event: LabelsChanged) => void): Promise<UnlistenFn> {
  return listen<LabelsChanged>('labels_changed', event => handler(event.payload));
}

/** H2, H4: `picture_ready`: a row's cover, a shop search result's picture or a remote item's picture was kept on this Mac at `path`. */
export type PictureReady =
  | { kind: 'cover'; session_id: number; path: string }
  | { kind: 'search'; external_id: string; path: string }
  | { kind: 'remote'; session_id: number; role: string; position: number; path: string };
export async function onPictureReady(handler: (event: PictureReady) => void): Promise<UnlistenFn> {
  return listen<PictureReady>('picture_ready', event => handler(event.payload));
}

/** H3: `bring_back_progress`: a bring back's progress (`done` of `total` while running), then done or failed with the reason. */
export type BringBack = {
  session_id: number;
  done: number;
  total: number;
  state: 'running' | 'done' | 'failed';
  error: string | null;
};
export async function onBringBack(handler: (event: BringBack) => void): Promise<UnlistenFn> {
  return listen<BringBack>('bring_back_progress', event => handler(event.payload));
}
