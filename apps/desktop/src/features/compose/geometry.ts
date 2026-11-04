/**
 * The TypeScript mirror of `psm_core::mechanisms::composition` (P28 §The geometry contract), proven on the same
 * `composition_cases.json`, plus the editor's view maths (plan P31 §The view maths).
 *
 * Frames: `Frame { w, h }` is the source frame as Capture One exports it (crop ignored, its own orientation applied).
 * Quarter turns and flip run first (lossless); `turnedFrame` swaps the sides for odd turns. Every rect fraction, every
 * ratio and the pixel rule use the turned frame `W × H`. Positive `straighten` rotates the image clockwise on screen.
 */
import type { CompositionValues, Frame, Rect } from '@/services/compositions';

/** A rect's short side is at least 5 % of the turned frame's short side. */
export const MIN_RECT_FRACTION = 0.05;
/** The frame box keeps this margin to the stage on every side. */
export const FRAME_MARGIN_PX = 24;
/** Geometry reals compare within this (P28 contract: `|a − b| ≤ 1e-6`). */
const GEOMETRY_TOLERANCE = 1e-6;

export type Point = { x: number; y: number };
export type Box = { left: number; top: number; width: number; height: number };
export type PixelRect = { left: number; top: number; width: number; height: number };
/** Capture One's geometry as the adapter reads it (opaque values, H7). */
export type EditorGeometry = { rotation: number; flip: string; keystoneVertical: number; keystoneHorizontal: number; keystoneAmount: number };
export type FrameSource = 'preview' | 'master';
export type Fingerprint = { frame: Frame; source: FrameSource; geometry: EditorGeometry };
export type ReviewReason = 'geometry' | 'frame';

export function turnedFrame(frame: Frame, quarterTurns: number): Frame {
  return quarterTurns % 2 === 1 ? { w: frame.h, h: frame.w } : frame;
}

export function orientationOf(frame: Frame): 'landscape' | 'portrait' | 'square' {
  if (frame.w > frame.h) return 'landscape';
  if (frame.w < frame.h) return 'portrait';
  return 'square';
}

/** `split → 2`; `original → W/H`; `free → null`; named shapes inverted when `orientation = turned`. */
export function shapeRatio(kind: string, shape: string, orientation: string, turned: Frame): number | null {
  if (kind === 'split') return 2;
  let r: number;
  switch (shape) {
    case 'original':
      return turned.w / turned.h;
    case '1:1':
      r = 1;
      break;
    case '4:5':
      r = 4 / 5;
      break;
    case '3:2':
      r = 3 / 2;
      break;
    case '2:3':
      r = 2 / 3;
      break;
    case '16:9':
      r = 16 / 9;
      break;
    default:
      return null;
  }
  return orientation === 'turned' ? 1 / r : r;
}

/** The largest rect of the shape's ratio centred in the turned frame; `free`/`original` → the whole frame. */
export function defaultRect(kind: string, shape: string, orientation: string, turned: Frame): Rect {
  const r = shapeRatio(kind, shape, orientation, turned);
  if (r === null) return { x: 0, y: 0, w: 1, h: 1 };
  const frameRatio = turned.w / turned.h;
  if (r >= frameRatio) {
    const h = frameRatio / r;
    return { x: 0, y: (1 - h) / 2, w: 1, h };
  }
  const w = r / frameRatio;
  return { x: (1 - w) / 2, y: 0, w, h: 1 };
}

/** Every rect corner's source point `s = c + R(−θ)(p − c)` lies inside the frame (ε = 1e-6 · max(W, H)). */
export function insideAfterStraighten(rect: Rect, turned: Frame, straighten: number): boolean {
  const w = turned.w;
  const h = turned.h;
  const cx = w / 2;
  const cy = h / 2;
  const rad = (straighten * Math.PI) / 180;
  const sin = Math.sin(rad);
  const cos = Math.cos(rad);
  const epsilon = 1e-6 * Math.max(w, h);
  for (const px of [rect.x * w, (rect.x + rect.w) * w]) {
    for (const py of [rect.y * h, (rect.y + rect.h) * h]) {
      const dx = px - cx;
      const dy = py - cy;
      const sx = cx + cos * dx + sin * dy;
      const sy = cy - sin * dx + cos * dy;
      if (sx < -epsilon || sx > w + epsilon || sy < -epsilon || sy > h + epsilon) return false;
    }
  }
  return true;
}

/** The first failing reason in the contract's order, or null when the values are valid. */
export function validate(values: CompositionValues, frame: Frame): string | null {
  if (values.quarter_turns > 3) return 'turns_range';
  const straighten = values.straighten ?? 0;
  if (!Number.isFinite(straighten) || Math.abs(straighten) > 45) return 'straighten_range';
  const r = values.rect;
  if ([r.x, r.y, r.w, r.h].some(v => !Number.isFinite(v) || v < 0 || v > 1) || r.w <= 0 || r.h <= 0) return 'fraction_range';
  if (r.x + r.w > 1 + 1e-9 || r.y + r.h > 1 + 1e-9) return 'rect_outside';
  const t = turnedFrame(frame, values.quarter_turns);
  if (Math.min(r.w * t.w, r.h * t.h) < MIN_RECT_FRACTION * Math.min(t.w, t.h)) return 'rect_too_small';
  const ratio = shapeRatio(values.kind, values.shape, values.orientation, t);
  if (ratio !== null) {
    if (values.kind === 'normal' && Math.abs(r.w * t.w - ratio * r.h * t.h) > 1) return 'off_shape';
    if (values.kind === 'split' && Math.abs(r.w * t.w - 2 * r.h * t.h) > 1) return 'split_ratio';
  }
  if (straighten !== 0 && !insideAfterStraighten(r, t, straighten)) return 'rect_outside';
  return null;
}

/** Rust `f64::round`: half away from zero. */
function roundHalfAway(v: number): number {
  return Math.sign(v) * Math.round(Math.abs(v));
}

export function pixelRect(rect: Rect, turned: Frame): PixelRect {
  return {
    left: roundHalfAway(rect.x * turned.w),
    top: roundHalfAway(rect.y * turned.h),
    width: roundHalfAway(rect.w * turned.w),
    height: roundHalfAway(rect.h * turned.h),
  };
}

/** Width rounded down to even; two adjacent squares of `width / 2`. */
export function splitHalves(pixel: PixelRect): [PixelRect, PixelRect] {
  const half = Math.max(Math.trunc(pixel.width / 2), 0);
  return [
    { ...pixel, width: half, height: half },
    { ...pixel, left: pixel.left + half, width: half, height: half },
  ];
}

export function geometryEqual(a: EditorGeometry, b: EditorGeometry): boolean {
  return (
    Math.abs(a.rotation - b.rotation) <= GEOMETRY_TOLERANCE &&
    Math.abs(a.keystoneVertical - b.keystoneVertical) <= GEOMETRY_TOLERANCE &&
    Math.abs(a.keystoneHorizontal - b.keystoneHorizontal) <= GEOMETRY_TOLERANCE &&
    a.flip === b.flip &&
    a.keystoneAmount === b.keystoneAmount
  );
}

/** Geometry equal, orientation equal, and equal pixel size when both observations are masters. */
export function fingerprintCompatible(stored: Fingerprint, observed: Fingerprint): boolean {
  return (
    geometryEqual(stored.geometry, observed.geometry) &&
    orientationOf(stored.frame) === orientationOf(observed.frame) &&
    !(stored.source === 'master' && observed.source === 'master' && (stored.frame.w !== observed.frame.w || stored.frame.h !== observed.frame.h))
  );
}

export function reviewReason(stored: Fingerprint, observed: Fingerprint): ReviewReason {
  return geometryEqual(stored.geometry, observed.geometry) ? 'frame' : 'geometry';
}

// ---- View maths (plan P31 §The view maths): the frame stays, the photo moves under it. ----

/** The largest box of ratio `rectRatio` centred in the stage shrunk by `FRAME_MARGIN_PX` on every side. */
export function frameBox(stage: Box, rectRatio: number): Box {
  const innerWidth = Math.max(0, stage.width - 2 * FRAME_MARGIN_PX);
  const innerHeight = Math.max(0, stage.height - 2 * FRAME_MARGIN_PX);
  let width: number;
  let height: number;
  if (innerHeight === 0 || innerWidth / innerHeight >= rectRatio) {
    height = innerHeight;
    width = height * rectRatio;
  } else {
    width = innerWidth;
    height = width / rectRatio;
  }
  return {
    left: stage.left + FRAME_MARGIN_PX + (innerWidth - width) / 2,
    top: stage.top + FRAME_MARGIN_PX + (innerHeight - height) / 2,
    width,
    height,
  };
}

export type ViewTransform = {
  frameBox: Box;
  /** Screen pixels per source pixel of the turned frame. */
  scale: number;
  /** The preview element's centre in stage coordinates. */
  centre: Point;
  rotateDeg: number;
  turned: Frame;
  /** The preview element's size: `W_t·s × H_t·s`. */
  width: number;
  height: number;
};

/** Where the turned, flipped preview sits so that `values.rect` fills the frame box. */
export function viewTransform(values: CompositionValues, frame: Frame, stage: Box): ViewTransform {
  const turned = turnedFrame(frame, values.quarter_turns);
  const r = values.rect;
  const box = frameBox(stage, (r.w * turned.w) / (r.h * turned.h));
  const scale = box.width / (r.w * turned.w);
  const width = turned.w * scale;
  const height = turned.h * scale;
  return {
    frameBox: box,
    scale,
    centre: { x: box.left + (0.5 - r.x) * width, y: box.top + (0.5 - r.y) * height },
    rotateDeg: values.straighten ?? 0,
    turned,
    width,
    height,
  };
}

const px = (v: number) => `${Math.round(v * 100) / 100}px`;

/** The CSS transform of the preview element (`transform-origin: center`). */
export function transformCss(view: ViewTransform): string {
  return `translate(${px(view.centre.x - view.width / 2)}, ${px(view.centre.y - view.height / 2)}) rotate(${view.rotateDeg}deg)`;
}

/** A stage point as fractions of the frame box (the anchor a zoom keeps still). */
export function anchorIn(box: Box, point: Point): Point {
  return { x: (point.x - box.left) / box.width, y: (point.y - box.top) / box.height };
}

/** The rect moved by `delta` (frame fractions). The machine passes `−(pointer movement) / (W_t·s, H_t·s)` so the photo follows the pointer. */
export function panBy(rect: Rect, delta: Point): Rect {
  return { ...rect, x: rect.x + delta.x, y: rect.y + delta.y };
}

/**
 * The composition zoomed by `factor` (> 1 zooms in: the rect shrinks) keeping the image point under `anchor`
 * (frame-box fractions) still; the factor is clamped so the rect never exceeds the frame nor shrinks below
 * `MIN_RECT_FRACTION` of the turned frame's short side.
 */
export function zoomAbout(rect: Rect, anchor: Point, factor: number, turned: Frame): Rect {
  const minFactor = Math.max(rect.w, rect.h);
  const maxFactor = Math.min(rect.w * turned.w, rect.h * turned.h) / (MIN_RECT_FRACTION * Math.min(turned.w, turned.h));
  let f = Math.min(factor, maxFactor);
  if (f < minFactor) f = minFactor;
  if (f === 1) return rect;
  const w = rect.w / f;
  const h = rect.h / f;
  const ix = rect.x + anchor.x * rect.w;
  const iy = rect.y + anchor.y * rect.h;
  return { x: ix - anchor.x * w, y: iy - anchor.y * h, w, h };
}

/** Keeps the ratio; a rect larger than the frame shrinks about its centre; `x ∈ [0, 1 − w]`, `y ∈ [0, 1 − h]`. */
export function clampRect(rect: Rect): Rect {
  let { x, y, w, h } = rect;
  const over = Math.max(w, h);
  if (over > 1) {
    const cx = x + w / 2;
    const cy = y + h / 2;
    w = Math.min(1, w / over);
    h = Math.min(1, h / over);
    x = cx - w / 2;
    y = cy - h / 2;
  }
  x = Math.min(Math.max(x, 0), Math.max(0, 1 - w));
  y = Math.min(Math.max(y, 0), Math.max(0, 1 - h));
  return { x, y, w, h };
}
