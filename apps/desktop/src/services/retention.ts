import { commands, type ArchivedPictureDto_Serialize, type ItemArchiveDto_Serialize } from './bindings';
import { unwrap } from './sessions';

/** An item's retention state (10F §3, Q2, Q3): the keep-files pin, whether its files were cleared, its photos from Shopify. */
export type ItemArchive = ItemArchiveDto_Serialize;
export type ArchivedPicture = ArchivedPictureDto_Serialize;

/** What a cleared item refuses with (10F Q3). */
export const FILES_CLEARED = "This item's files were cleared after publishing";

/** The session's item retention state; a cleared item's photos are downloaded from Shopify once and cached. */
export async function getItemArchive(sessionId: number): Promise<ItemArchive | null> {
  return unwrap(await commands.itemArchiveGet(sessionId));
}

/** "Keep files on this Mac": the item is never cleared by retention or by the storage budget. */
export async function setKeepFiles(sessionId: number, keep: boolean): Promise<ItemArchive | null> {
  return unwrap(await commands.itemKeepFilesSet(sessionId, keep));
}

/** Days a published, clean item keeps its local files (Settings → Storage). */
export async function getRetentionDays(): Promise<number> {
  return unwrap(await commands.storageRetentionGet());
}

export async function setRetentionDays(days: number): Promise<number> {
  return unwrap(await commands.storageRetentionSet(days));
}
