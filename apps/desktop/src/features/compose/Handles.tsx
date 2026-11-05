import type { Box } from './geometry';
import { HANDLES, type Handle } from './refit';

/** Handle hit size in screen pixels (plan P32 Delegated choices: 12 px). */
export const HANDLE_PX = 12;

type HandlesProps = { box: Box };

/** Where `handle` sits on `box` (its centre). */
function anchor(box: Box, handle: Handle): { x: number; y: number } {
  const x = handle.includes('w') ? box.left : handle.includes('e') ? box.left + box.width : box.left + box.width / 2;
  const y = handle.includes('n') ? box.top : handle.includes('s') ? box.top + box.height : box.top + box.height / 2;
  return { x, y };
}

/**
 * The eight trim handles on the frame (10D: "corner and edge handles trim the frame"). Their pointer down is read by
 * `useComposeGesture` on the stage (`.compose-handle[data-handle]`) and becomes a `TrimStart`.
 */
export function Handles({ box }: HandlesProps) {
  return (
    <g className="compose-handles">
      {HANDLES.map(handle => {
        const at = anchor(box, handle);
        return <rect key={handle} className="compose-handle" data-handle={handle} x={at.x - HANDLE_PX / 2} y={at.y - HANDLE_PX / 2} width={HANDLE_PX} height={HANDLE_PX} rx={2} />;
      })}
    </g>
  );
}
