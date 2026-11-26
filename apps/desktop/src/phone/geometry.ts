/**
 * Where things are on the phone's screen (design README §Image viewport sizing; master_plan_live
 * §5 PH6, PH7, HB6). Pure functions: sizes in CSS px, frame points as 0…1 of the full live frame.
 */

export type Size = { width: number; height: number };
export type Point = { x: number; y: number };
/** A point in image-local coordinates, with the image's own width and height. */
export type LocalPoint = Point & { w: number; h: number };
type Rect = { left: number; top: number; right: number; width: number; height: number };

/** Design: a shot wider than this is "long": the viewport turns 90° so the phone is held sideways. */
export function isRotated(ratio: number): boolean {
  return ratio > 1.2;
}

/**
 * The image viewport's own box in the stage, before it is turned: not rotated → as wide as the
 * stage allows (`min(stageW, stageH·r)`); rotated → as wide as the stage is tall
 * (`min(stageH, stageW·r)`), then turned 90° clockwise.
 */
export function viewportBox(stage: Size, ratio: number): Size & { rotated: boolean } {
  const rotated = isRotated(ratio);
  const width = rotated ? Math.min(stage.height, stage.width * ratio) : Math.min(stage.width, stage.height * ratio);
  return { width, height: width / ratio, rotated };
}

/**
 * A touch in image-local coordinates: inside a rotated viewport "up" is up for the person holding
 * the phone sideways (`x = clientY − top`, `y = right − clientX`, and the width is the box's height).
 */
export function localPoint(client: Point, rect: Rect, rotated: boolean): LocalPoint {
  return rotated
    ? { x: client.y - rect.top, y: rect.right - client.x, w: rect.height, h: rect.width }
    : { x: client.x - rect.left, y: client.y - rect.top, w: rect.width, h: rect.height };
}

/** The shot's zoom preset as a scale about the centre: `1 / (1 − zoom/100)` (10J: live shows the area the container keeps). */
export function zoomScale(zoom: number): number {
  return 1 / (1 - Math.min(Math.max(zoom, 0), 90) / 100);
}

/**
 * HB6: the point of the full live frame under a tap. The frame is drawn to cover the viewport
 * (cropped on its longer side), then scaled by the zoom about the centre; both are undone here.
 */
export function tapToFrame(local: Point, viewport: Size, frame: Size, zoom: number): Point {
  const scale = zoomScale(zoom);
  const unzoomed = {
    x: viewport.width / 2 + (local.x - viewport.width / 2) / scale,
    y: viewport.height / 2 + (local.y - viewport.height / 2) / scale,
  };
  const cover = Math.max(viewport.width / frame.width, viewport.height / frame.height);
  const drawn = { width: frame.width * cover, height: frame.height * cover };
  const clamp = (v: number) => Math.min(1, Math.max(0, v));
  return {
    x: clamp((unzoomed.x - (viewport.width - drawn.width) / 2) / drawn.width),
    y: clamp((unzoomed.y - (viewport.height - drawn.height) / 2) / drawn.height),
  };
}

const NAMED: [string, number][] = [['1:1', 1], ['4:5', 4 / 5], ['3:2', 3 / 2], ['2:3', 2 / 3], ['16:9', 16 / 9]];

/** The ratio chip's text: a split shot is two squares (LV-D16); else the shape's name, else the number. */
export function ratioLabel(ratio: number, split: boolean): string {
  if (split) return 'SPLIT 2×1:1';
  return NAMED.find(([, value]) => Math.abs(value - ratio) < 0.01)?.[0] ?? ratio.toFixed(2);
}
