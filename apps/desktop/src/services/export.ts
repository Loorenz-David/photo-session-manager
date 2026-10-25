import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { commands } from './bindings';
import { unwrap } from './sessions';

export type ExportState = 'running' | 'done' | 'failed';
/** `export_progress` payload (master plan §6.7); `total` on `done` is the number of photo files written. */
export type ExportProgress = { session_id: number; done: number; total: number; state: ExportState };

/** The configured integration's kind, or null when none is configured (the header then offers Export, H6). */
export async function integrationKind(): Promise<string | null> {
  return unwrap(await commands.integrationGet());
}

/** Starts the session's local export; progress and the outcome arrive through `onExportProgress`. */
export async function runExport(sessionId: number): Promise<number> {
  return unwrap(await commands.exportRun(sessionId)).run_id;
}

/** Shows `<export folder>/<SKU>/` of the session in Finder. */
export async function openExportFolder(sessionId: number): Promise<void> {
  unwrap(await commands.exportOpenFolder(sessionId));
}

/** Every `export_progress` event. */
export async function onExportProgress(handler: (progress: ExportProgress) => void): Promise<UnlistenFn> {
  return listen<ExportProgress>('export_progress', event => handler(event.payload));
}
