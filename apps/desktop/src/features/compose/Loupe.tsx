import type { Point } from './geometry';
import type { ComposeState } from './machine';

export function Loupe({ src, pointer, state }: { src: string; pointer: Point; state: ComposeState }) {
  const view = state.view;
  const imageX = (pointer.x - (view.centre.x - view.width / 2)) / view.scale;
  const imageY = (pointer.y - (view.centre.y - view.height / 2)) / view.scale;
  return (
    <div className="compose-loupe" style={{ left: pointer.x + 16, top: pointer.y + 16 }}>
      <img src={src} alt="100 percent preview" draggable={false} style={{ width: view.turned.w, height: view.turned.h, transform: `translate(${120 - imageX}px, ${120 - imageY}px)` }} />
    </div>
  );
}
