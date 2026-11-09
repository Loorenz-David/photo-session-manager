import type { EditorPicture } from './editorPicture';
import type { Point } from './geometry';
import type { ComposeState } from './machine';

export function Loupe({ picture, pointer, state }: { picture: EditorPicture; pointer: Point; state: ComposeState }) {
  const view = state.view;
  // Undo the stage's straighten first: its preview element is rotated around the
  // centre of the unturned view, while the loupe's window follows the screen pointer.
  const angle = (view.rotateDeg * Math.PI) / 180;
  const dx = pointer.x - view.centre.x;
  const dy = pointer.y - view.centre.y;
  const turnedX = (view.centre.x + Math.cos(angle) * dx + Math.sin(angle) * dy - (view.centre.x - view.width / 2)) / view.scale;
  const turnedY = (view.centre.y - Math.sin(angle) * dx + Math.cos(angle) * dy - (view.centre.y - view.height / 2)) / view.scale;
  const turned = view.turned;
  // The image element before its turn: the frame, or its sides exchanged for a camera preview on its side.
  const quarter = picture.swapped !== (state.values.quarter_turns % 2 === 1);
  const unturned = quarter ? { w: state.frame.h, h: state.frame.w } : state.frame;
  return (
    <div className="compose-loupe" style={{ left: pointer.x + 16, top: pointer.y + 16 }}>
      <div className="compose-loupe-view" style={{ width: turned.w, height: turned.h, left: 120 - turnedX, top: 120 - turnedY, transformOrigin: `${turnedX}px ${turnedY}px`, transform: `rotate(${view.rotateDeg}deg)` }}>
        <img src={picture.src} alt="100 percent preview" draggable={false} style={{ width: unturned.w, height: unturned.h, translate: '-50% -50%', transform: picture.transform }} />
      </div>
    </div>
  );
}
