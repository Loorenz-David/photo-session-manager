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

/** The editor opens only on a photo with an adjusted preview and a materialised frame (plan P31 Goal). */
export function canCompose(photo: Photo): boolean {
  return photo.adjusted_preview_path !== null && photo.composition.rect !== null && photo.composition.frame !== null;
}
