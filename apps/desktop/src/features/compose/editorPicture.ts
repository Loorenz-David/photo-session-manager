import { composePreview, type CompositionValues } from '@/services/compositions';
import type { Photo } from '@/services/photos';
import { previewUrl } from '@/services/sessions';
import { orientationTransform } from '@/features/workspace/StageImage';

export type EditorPicture = {
  src: string;
  /** Composition turn and flip, then (camera preview only) the RAW's orientation (MC-20). */
  transform: string;
  /** The image element is laid out with the frame's sides exchanged (odd quarter turns in total). */
  swapped: boolean;
  /** Showing the camera's preview while Capture One's version is on its way (P58). */
  camera: boolean;
};

/**
 * The picture the editor draws under the frame (P58, book changelog 39): Capture One's adjusted preview, else the
 * camera's own full-size preview turned by its stored orientation (display only, H3) when its frame matched.
 */
export function editorPicture(photo: Photo, values: Pick<CompositionValues, 'quarter_turns' | 'flip'>): EditorPicture | null {
  const preview = composePreview(photo);
  if (!preview) return null;
  const turn = `rotate(${values.quarter_turns * 90}deg) scaleX(${values.flip === 'horizontal' ? -1 : 1})`;
  const orientation = preview.camera ? orientationTransform(photo.orientation) : undefined;
  const quarter = preview.camera && photo.orientation !== null && photo.orientation >= 5 && photo.orientation <= 8;
  return {
    src: previewUrl(preview.path),
    transform: orientation ? `${turn} ${orientation}` : turn,
    swapped: (values.quarter_turns % 2 === 1) !== quarter,
    camera: preview.camera,
  };
}
