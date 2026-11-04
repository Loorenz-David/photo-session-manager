import type { RefObject } from 'react';
import { valuesOf } from '@/services/compositions';
import type { Photo } from '@/services/photos';
import { previewUrl } from '@/services/sessions';
import { ComposeFrame } from './ComposeFrame';
import { viewTransform, type Box } from './geometry';

type ComposeStageProps = {
  photo: Photo;
  stageBox: Box | null;
  stageRef: RefObject<HTMLDivElement | null>;
  imageRef: RefObject<HTMLDivElement | null>;
  /** The first-time line (10D: "Crops made in Capture One are not used."). */
  hint: boolean;
};

/**
 * The editor's stage (10D §What the photographer sees): the frame stays centred, the 2560 px adjusted preview moves
 * under it. The preview wrapper's size and transform are written by `useComposeGesture` (once per animation frame),
 * never through React, so a gesture never waits for a render; the inner image carries the turn and flip.
 */
export function ComposeStage({ photo, stageBox, stageRef, imageRef, hint }: ComposeStageProps) {
  const values = valuesOf(photo.composition);
  const frame = photo.composition.frame;
  const view = values && frame && stageBox ? viewTransform(values, frame, { left: 0, top: 0, width: stageBox.width, height: stageBox.height }) : null;
  const turns = values?.quarter_turns ?? 0;
  const flipped = values?.flip === 'horizontal';
  return (
    <div ref={stageRef} className="compose-stage" aria-label="Compose">
      <div ref={imageRef} className="compose-image" data-turned={turns % 2 === 1 || undefined}>
        {photo.adjusted_preview_path && (
          <img
            key={photo.id}
            src={previewUrl(photo.adjusted_preview_path)}
            alt=""
            draggable={false}
            style={{ transform: `rotate(${turns * 90}deg) scaleX(${flipped ? -1 : 1})` }}
          />
        )}
      </div>
      {view && stageBox && <ComposeFrame width={stageBox.width} height={stageBox.height} frameBox={view.frameBox} />}
      {hint && (
        <p className="compose-hint" role="status">
          Crops made in Capture One are not used.
        </p>
      )}
    </div>
  );
}
