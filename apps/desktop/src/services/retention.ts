import { commands } from './bindings';
import type { RemoteView } from './remote';
import { unwrap } from './sessions';

/** What a cleared item refuses with (10F Q3). */
export const FILES_CLEARED = "This item's files were cleared after publishing";

/** "Keep files on this Mac": the item is never cleared by retention or by the storage budget. Answers the session's remote view (H4). */
export async function setKeepFiles(sessionId: number, keep: boolean): Promise<RemoteView | null> {
  return unwrap(await commands.itemKeepFilesSet(sessionId, keep));
}

/** Days a published, clean item keeps its local files (Settings → Storage). */
export async function getRetentionDays(): Promise<number> {
  return unwrap(await commands.storageRetentionGet());
}

export async function setRetentionDays(days: number): Promise<number> {
  return unwrap(await commands.storageRetentionSet(days));
}
