import { commands, type CompositionDto_Serialize, type CompositionValuesDto, type FrameDto } from './bindings';
import type { Photo } from './photos';
import { unwrap } from './sessions';

/** A photo's composition as the backend returns it (10D MC-27; `rect`/`frame` are null until the first render). */
export type Composition = CompositionDto_Serialize;
export type Frame = FrameDto;
/** Fractions of the turned, flipped, unstraightened frame (MC-27 §Values). */
export type Rect = { x: number; y: number; w: number; h: number };
/** What a save sends: the DTO with a materialised rect. */
export type CompositionValues = Omit<CompositionValuesDto, 'rect'> & { rect: Rect };

/** Saves the composition; the backend validates (MC-27 §Validation) and answers with the stored row (`revision + 1`). */
export async function setComposition(photoId: number, values: CompositionValues): Promise<Composition> {
  return unwrap(await commands.compositionSet(photoId, values));
}

/** Back to the default composition of the current `default_shape`. */
export async function resetComposition(photoId: number): Promise<Composition> {
  return unwrap(await commands.compositionReset(photoId));
}

export type CompositionApplyItem = { photo_id: number; values: CompositionValues };
export async function applyComposition(items: CompositionApplyItem[]) {
  return unwrap(await commands.compositionApply(items));
}

/** The composition's values when the backend has materialised its rect, else null. */
export function valuesOf(composition: Composition): CompositionValues | null {
  const r = composition.rect;
  if (!r || r.x === null || r.y === null || r.w === null || r.h === null) return null;
  return {
    kind: composition.kind,
    shape: composition.shape,
    orientation: composition.orientation,
    quarter_turns: composition.quarter_turns,
    flip: composition.flip,
    straighten: composition.straighten,
    rect: { x: r.x, y: r.y, w: r.w, h: r.h },
  };
}

/**
 * The editor opens on a photo with a materialised frame and a picture to show (plan P31 Goal). Capture One's adjusted
 * preview always qualifies. The camera's own preview qualifies only while the composition's frame came from it
 * (`frame_source === 'camera'`, P58, book changelog 39): a frame from an export's master with no adjusted preview yet
 * says nothing about the camera preview matching, so Compose waits.
 */
export function canCompose(photo: Photo): boolean {
  if (photo.composition.rect === null || photo.composition.frame === null) return false;
  return photo.adjusted_preview_path !== null
    || (photo.original_preview_path !== null && photo.composition.frame_source === 'camera');
}

/** The preview the editor composes on: Capture One's adjusted one, else the camera's (P58). */
export function composePreview(photo: Photo): { path: string; camera: boolean } | null {
  if (photo.adjusted_preview_path) return { path: photo.adjusted_preview_path, camera: false };
  if (canCompose(photo) && photo.original_preview_path) return { path: photo.original_preview_path, camera: true };
  return null;
}

/** The editor's line while it shows the camera preview (P58). */
export const CAMERA_PREVIEW_NOTE = "Camera preview — Capture One's version is on its way";

export const COMPOSE_WAITING = "Waiting for Capture One's full-frame preview";
