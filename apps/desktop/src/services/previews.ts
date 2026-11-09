import { commands } from './bindings';

/**
 * Tells the backend which photo is on the stage or in the editor (`null`: none); its pending preview is the next one
 * Capture One renders (P58, book changelog 39). Best effort: the queue order is a speed-up, never a failure.
 */
export function setPreviewFocus(photoId: number | null): void {
  void commands.photosPreviewFocus(photoId).catch(() => undefined);
}
