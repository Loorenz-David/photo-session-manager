/**
 * The design's gesture rules as pure functions (design README §Interactions; the prototype's logic
 * class is the reference for every threshold).
 */

/** Slide past this many px (image-local) before a touch becomes a lock slide or an overlay slide. */
export const SLIDE_PX = 14;
/** Slide up this far to lock live. */
export const LOCK_PX = 110;
/** A full image width of sideways slide changes the overlay by this many percent. */
export const OVERLAY_GAIN = 130;
/** A tap while locked focuses after this long (so a double tap can cancel it). */
export const TAP_FOCUS_MS = 260;
const DOUBLE_TAP_MS = 300;
const DOUBLE_TAP_PX = 50;

const clamp = (value: number, low: number, high: number) => Math.max(low, Math.min(high, value));

export type TouchMode = 'overlay' | 'lock' | 'none';

/**
 * What a touch on the image becomes, decided once (the first threshold crossed) and kept until
 * release; `null` = still a plain hold. Sideways → the template overlay (not in Free shot: there is
 * no template); up → the lock slide (nothing when already locked).
 */
export function touchMode(dx: number, dy: number, now: { locked: boolean; free: boolean }): TouchMode | null {
  if (!now.free && Math.abs(dx) > SLIDE_PX && Math.abs(dx) > Math.abs(dy)) return 'overlay';
  if (dy < -SLIDE_PX && Math.abs(dy) > Math.abs(dx)) return now.locked ? 'none' : 'lock';
  return null;
}

/** How far the lock slide is, 0…1 (locked at 1). */
export function lockProgress(dy: number): number {
  return clamp(-dy / LOCK_PX, 0, 1);
}

/** The overlay after sliding `dx` px across an image `width` px wide, from `start` percent. */
export function overlayValue(start: number, dx: number, width: number): number {
  return Math.round(clamp(start + (dx / width) * OVERLAY_GAIN, 0, 100));
}

export type Tap = { t: number; x: number; y: number };

/** Two taps close in time and place (the second one unlocks). */
export function isDoubleTap(first: Tap, second: Tap): boolean {
  return second.t - first.t < DOUBLE_TAP_MS && Math.hypot(second.x - first.x, second.y - first.y) < DOUBLE_TAP_PX;
}

/** Design §Haptics (ms). Only where the browser has them (Android Chrome); iPhone keeps the visual cues. */
export const HAPTIC = {
  capture: [80, 40, 120],
  shutterPress: 20,
  lock: [25, 40, 70],
  unlock: [20, 30, 20],
  move: [30, 40, 60],
  retake: [30, 40, 30],
  focus: 25,
  select: 15,
  magnify: 12,
  menu: 12,
  hold: 10,
  clearFocus: 10,
  hoverTick: 6,
  overlayTick: 5,
  dragOpen: 18,
} as const;

export function buzz(pattern: number | readonly number[]): void {
  try {
    navigator.vibrate?.(pattern as number | number[]);
  } catch {
    // A browser that refuses to vibrate changes nothing else.
  }
}
