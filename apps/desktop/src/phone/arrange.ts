/**
 * Sort and arrange on the phone (book 10J LV-D20; master_plan_live §5 PH9): the pure rules the
 * Preview's swipe and the strips' drag and drop stand on. The Mac decides every placement (PH4a);
 * these only say what to ask it and what to show meanwhile.
 */
import type { PhoneSession } from './api';
import { photoIds, type PreviewSubject } from './state';

export type SwipeDirection = 'left' | 'right' | 'up';
/** What a counted swipe does: a request, the confirmation first (`main_confirm`), or nothing. */
export type SwipeAction = 'retake' | 'delete' | 'main' | 'main_confirm' | 'condition' | 'none';

/**
 * PH9.1: left denies (a template shot's photo by Retake, so its shot is empty and current), right
 * makes Main (a template shot's photo asks first; an extra already is), up sends to Condition (a
 * Condition photo already is).
 */
export function swipeAction(subject: PreviewSubject, direction: SwipeDirection): SwipeAction {
  if (direction === 'left') return subject.kind === 'shot' ? 'retake' : 'delete';
  if (direction === 'right') return subject.kind === 'shot' ? 'main_confirm' : subject.kind === 'extra' ? 'none' : 'main';
  return subject.kind === 'condition' ? 'none' : 'condition';
}

/**
 * Plan point P4: the photo after `photoId` in the strip's order (template shots that have a photo
 * in shooting order, the extras, the Condition images); `null` after the last one, or when the
 * photo is not in the session.
 */
export function nextPhoto(view: PhoneSession, photoId: number): number | null {
  const ids = photoIds(view);
  const at = ids.indexOf(photoId);
  return at < 0 ? null : ids[at + 1] ?? null;
}
