import type { CSSProperties } from 'react';
import type { CompositionValues, Frame, Rect } from '@/services/compositions';
import type { Photo } from '@/services/photos';
import { previewUrl } from '@/services/sessions';
import { turnedFrame } from './geometry';

type SplitPreviewsProps = { photo: Photo; values: CompositionValues; frame: Frame };

const pct = (v: number) => `${v * 100}%`;
const SIDES = [
  { side: 'left', label: 'Left' },
  { side: 'right', label: 'Right' },
] as const;

/**
 * The two square outputs of a Split (10D: "one 2:1 composition whose exact middle makes two square files, Left and
 * Right"; "the output previews at their real proportions"): side by side, each the same adjusted preview clipped to its
 * half of the rect — CSS only, no second network resource. The clip fractions of each half (`data-x0`, `data-x1`) and
 * its side (`data-half`) sit on the clipped frame element.
 */
export function SplitPreviews({ photo, values, frame }: SplitPreviewsProps) {
  const turned = turnedFrame(frame, values.quarter_turns);
  const { rect } = values;
  return (
    <div className="split-previews" aria-label="Output previews">
      {SIDES.map(({ side, label }) => {
        const half: Rect = { x: side === 'left' ? rect.x : rect.x + rect.w / 2, y: rect.y, w: rect.w / 2, h: rect.h };
        // The turned frame scaled so that the half fills the square box; the straighten turns it about the frame's centre.
        const frameStyle: CSSProperties = {
          width: pct(1 / half.w),
          height: pct(1 / half.h),
          left: pct(-half.x / half.w),
          top: pct(-half.y / half.h),
          transform: `rotate(${values.straighten ?? 0}deg)`,
        };
        return (
          <div key={side} className="split-preview-column">
            <div className="split-preview" data-side={side} style={{ aspectRatio: '1 / 1' }} aria-label={label} title={`${label}: ${Math.round(half.w * turned.w)} px`}>
              <div className="output-preview-frame" data-half={side} data-x0={half.x} data-x1={half.x + half.w} data-turned={values.quarter_turns % 2 === 1 || undefined} style={frameStyle}>
                {photo.adjusted_preview_path && (
                  <img src={previewUrl(photo.adjusted_preview_path)} alt="" draggable={false} style={{ transform: `rotate(${values.quarter_turns * 90}deg) scaleX(${values.flip === 'horizontal' ? -1 : 1})` }} />
                )}
              </div>
            </div>
            <span className="split-preview-label">{label}</span>
          </div>
        );
      })}
    </div>
  );
}
