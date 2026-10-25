import type { ReactNode } from 'react';
import { motion, useMotionValue, useTransform, type PanInfo } from 'motion/react';

/** Swipe thresholds (NOTES.md §Flows, Review): 120 px horizontal, 110 px vertical. */
export const SWIPE_X_PX = 120;
export const SWIPE_Y_PX = 110;

export type SwipeAction = 'approve' | 'condition' | 'delete';

/** Right = approve, up = condition, left = delete; anything shorter snaps back. */
export function swipeAction(offset: { x: number; y: number }): SwipeAction | null {
  if (-offset.y >= SWIPE_Y_PX && -offset.y >= Math.abs(offset.x)) return 'condition';
  if (offset.x >= SWIPE_X_PX) return 'approve';
  if (offset.x <= -SWIPE_X_PX) return 'delete';
  return null;
}

type SwipeLayerProps = { children: ReactNode; onSwipe: (action: SwipeAction) => void };

/** Drag the photo on the stage to review it; stamps fade in with the drag distance. */
export function SwipeLayer({ children, onSwipe }: SwipeLayerProps) {
  const x = useMotionValue(0);
  const y = useMotionValue(0);
  const approveOpacity = useTransform(x, [0, SWIPE_X_PX], [0, 1]);
  const deleteOpacity = useTransform(x, [-SWIPE_X_PX, 0], [1, 0]);
  const conditionOpacity = useTransform(y, [-SWIPE_Y_PX, 0], [1, 0]);

  function onDragEnd(_event: PointerEvent | MouseEvent | TouchEvent, info: PanInfo) {
    const action = swipeAction(info.offset);
    if (action) onSwipe(action);
  }

  return (
    <motion.div className="swipe-layer" drag dragSnapToOrigin dragElastic={0.6} style={{ x, y }} onDragEnd={onDragEnd}>
      {children}
      <motion.span className="swipe-stamp stamp-approve" style={{ opacity: approveOpacity }} aria-hidden>
        Main
      </motion.span>
      <motion.span className="swipe-stamp stamp-delete" style={{ opacity: deleteOpacity }} aria-hidden>
        Delete
      </motion.span>
      <motion.span className="swipe-stamp stamp-condition" style={{ opacity: conditionOpacity }} aria-hidden>
        Condition
      </motion.span>
    </motion.div>
  );
}
