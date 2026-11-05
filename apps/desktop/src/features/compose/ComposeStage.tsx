import type { RefObject } from 'react';
import { valuesOf } from '@/services/compositions';
import type { Photo } from '@/services/photos';
import { previewUrl } from '@/services/sessions';
import { ComposeFrame } from './ComposeFrame';
import type { Box } from './geometry';
import { Handles } from './Handles';
import type { ComposeState } from './machine';

type ComposeStageProps = {
  photo: Photo;
  stageBox: Box | null;
  /** The machine's published state (null until the stage is measured). */
  state: ComposeState | null;
  stageRef: RefObject<HTMLDivElement | null>;
  imageRef: RefObject<HTMLDivElement | null>;
  /** The first-time line (10D: "Crops made in Capture One are not used."). */
  hint: boolean;
  onPointer?: (point: { x: number; y: number }) => void;
};

/** The Re-frame notice (MC-27 §Revisions). */
export function reviewText(reason: string | null): string {
  return reason === 'geometry' ? 'Changed in Capture One: straighten here instead' : 'Capture One changed this photo’s frame — check the framing and press Return';
}

/** During a trim the overlay draws the trimmed rect inside the fixed frame box; otherwise the frame box itself. */
function drawnBox(state: ComposeState): Box {
  const box = state.view.frameBox;
  if (state.phase.kind !== 'trimming') return box;
  const start = state.phase.startRect;
  const live = state.live;
  return {
    left: box.left + (live.x - start.x) * state.view.width,
    top: box.top + (live.y - start.y) * state.view.height,
    width: live.w * state.view.width,
    height: live.h * state.view.height,
  };
}

/**
 * The editor's stage (10D §What the photographer sees): the frame stays centred, the 2560 px adjusted preview moves
 * under it. The preview wrapper's size and transform are written by `useComposeGesture` (once per animation frame),
 * never through React, so a gesture never waits for a render; the inner image carries the turn and flip; the overlay
 * carries the frame, the handles and, while composing a `needs_review` photo, the Re-frame banner.
 */
export function ComposeStage({ photo, stageBox, state, stageRef, imageRef, hint, onPointer }: ComposeStageProps) {
  const values = state?.values ?? valuesOf(photo.composition);
  const turns = values?.quarter_turns ?? 0;
  const flipped = values?.flip === 'horizontal';
  const review = photo.composition.state === 'needs_review';
  return (
    <div ref={stageRef} className="compose-stage" aria-label="Compose" onPointerMove={event => { const box = event.currentTarget.getBoundingClientRect(); onPointer?.({ x: event.clientX - box.left, y: event.clientY - box.top }); }}>
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
      {state && stageBox && (
        <ComposeFrame width={stageBox.width} height={stageBox.height} frameBox={drawnBox(state)} kind={state.values.kind} guides={state.guides}>
          <Handles box={drawnBox(state)} />
        </ComposeFrame>
      )}
      {review && (
        <p className="compose-review" role="status">
          {reviewText(photo.composition.review_reason)}
        </p>
      )}
      {hint && (
        <p className="compose-hint" role="status">
          Crops made in Capture One are not used.
        </p>
      )}
    </div>
  );
}
