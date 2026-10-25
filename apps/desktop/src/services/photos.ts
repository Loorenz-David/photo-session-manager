import { open } from '@tauri-apps/plugin-dialog';
import {
  commands,
  type PhotoClassification,
  type PhotoResponse_Serialize,
  type ReviewResponse_Serialize,
  type SessionViewResponse_Serialize,
} from './bindings';
import { unwrap } from './sessions';

export type Photo = PhotoResponse_Serialize;
export type SessionView = SessionViewResponse_Serialize;
export type ReviewResult = ReviewResponse_Serialize;
export type Classification = PhotoClassification;

/** The session with its item and its visible photos (deleted ones are hidden). */
export async function getSessionView(sessionId: number): Promise<SessionView> {
  return unwrap(await commands.sessionGet(sessionId));
}

/** Classify into a group (MC-5 "classify into g": appended at the end of the group). */
export async function classifyPhoto(photoId: number, classification: Classification): Promise<ReviewResult> {
  return unwrap(await commands.photosClassify(photoId, classification));
}

/** Reorder within the photo's group to the 1-based `position` (clamped by the backend). */
export async function reorderPhoto(photoId: number, position: number): Promise<ReviewResult> {
  return unwrap(await commands.photosReorder(photoId, position));
}

/** Deletes (hides) the photo; it becomes the session's single undo slot. The RAW is never touched. */
export async function deletePhoto(photoId: number): Promise<ReviewResult> {
  return unwrap(await commands.photosDelete(photoId));
}

/** Undoes the session's last delete; `active_photo_id` is the restored photo. */
export async function undoDelete(sessionId: number): Promise<ReviewResult> {
  return unwrap(await commands.photosUndo(sessionId));
}

export async function movePhotos(photoIds: number[], targetSessionId: number): Promise<ReviewResult> {
  return unwrap(await commands.photosMove(photoIds, targetSessionId));
}

export async function copyPhotos(photoIds: number[], targetSessionId: number): Promise<ReviewResult> {
  return unwrap(await commands.photosCopy(photoIds, targetSessionId));
}

/** Imports files the same way captures arrive (`source = import`). */
export async function importPhotos(paths: string[]): Promise<void> {
  unwrap(await commands.photosImport(paths));
}

/** Assigns every Unassigned photo to the item's session (created when the item has none). */
export async function assignUnassigned(itemId: number): Promise<ReviewResult> {
  return unwrap(await commands.photosAssignUnassigned(itemId));
}

/** Opens the macOS file picker for photos to import; resolves to the chosen paths (empty when cancelled). */
export async function pickImportFiles(): Promise<string[]> {
  const chosen = await open({ multiple: true, directory: false, title: 'Import photos' });
  if (!chosen) return [];
  return Array.isArray(chosen) ? chosen : [chosen];
}
