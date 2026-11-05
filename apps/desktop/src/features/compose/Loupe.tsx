import type { Point } from './geometry';
import type { ComposeState } from './machine';

export function Loupe({ src, pointer, state }: { src: string; pointer: Point; state: ComposeState }) {
  const view = state.view;
  // Undo the stage's straighten first: its preview element is rotated around the
  // centre of the unturned view, while the loupe's window follows the screen pointer.
  const angle = (view.rotateDeg * Math.PI) / 180;
  const dx = pointer.x - view.centre.x;
  const dy = pointer.y - view.centre.y;
  const turnedX = (view.centre.x + Math.cos(angle) * dx + Math.sin(angle) * dy - (view.centre.x - view.width / 2)) / view.scale;
  const turnedY = (view.centre.y - Math.sin(angle) * dx + Math.cos(angle) * dy - (view.centre.y - view.height / 2)) / view.scale;
  const turned = view.turned;
  const turns = state.values.quarter_turns % 4;
  return (
    <div className="compose-loupe" style={{ left: pointer.x + 16, top: pointer.y + 16 }}>
      <div className="compose-loupe-view" style={{ width: turned.w, height: turned.h, left: 120 - turnedX, top: 120 - turnedY, transformOrigin: `${turnedX}px ${turnedY}px`, transform: `rotate(${view.rotateDeg}deg)` }}>
        <img src={src} alt="100 percent preview" draggable={false} style={{ width: state.frame.w, height: state.frame.h, translate: '-50% -50%', transform: `rotate(${turns * 90}deg) scaleX(${state.values.flip === 'horizontal' ? -1 : 1})` }} />
      </div>
    </div>
  );
}
