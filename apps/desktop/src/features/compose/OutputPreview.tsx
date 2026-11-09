import type { CSSProperties } from 'react';
import type { CompositionValues, Frame } from '@/services/compositions';
import type { Photo } from '@/services/photos';
import { editorPicture } from './editorPicture';
import { turnedFrame } from './geometry';

type OutputPreviewProps = { photo: Photo; values: CompositionValues; frame: Frame };

const px = (v: number) => Number(v.toFixed(4));
const pct = (v: number) => `${v * 100}%`;

/**
 * The composed result at its real proportions (10D §What the photographer sees: "the output previews at their real
 * proportions"): a box of the rect's pixel ratio showing the same adjusted preview clipped to the rect — CSS only, no
 * second network resource.
 */
export function OutputPreview({ photo, values, frame }: OutputPreviewProps) {
  const picture = editorPicture(photo, values);
  const turned = turnedFrame(frame, values.quarter_turns);
  const { rect } = values;
  const box: CSSProperties = { aspectRatio: `${px(rect.w * turned.w)} / ${px(rect.h * turned.h)}` };
  // The turned frame scaled so that the rect fills the box; the straighten turns it about the frame's centre.
  const frameStyle: CSSProperties = {
    width: pct(1 / rect.w),
    height: pct(1 / rect.h),
    left: pct(-rect.x / rect.w),
    top: pct(-rect.y / rect.h),
    transform: `rotate(${values.straighten ?? 0}deg)`,
  };
  return (
    <div className="output-preview" style={box} aria-label="Output preview">
      <div className="output-preview-frame" data-turned={values.quarter_turns % 2 === 1 || undefined} style={frameStyle}>
        {picture && (
          <img src={picture.src} alt="" draggable={false} data-swapped={picture.swapped || undefined} style={{ transform: picture.transform }} />
        )}
      </div>
    </div>
  );
}
