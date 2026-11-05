import type { CompositionValues, Frame } from '@/services/compositions';
import { turnedFrame } from './geometry';

const px = (value: number) => Number(value.toFixed(4));
const percent = (value: number) => `${px(value * 100)}%`;

/** CSS geometry of the output rectangle over the turned frame. */
export function composedLayout(values: CompositionValues, frame: Frame) {
  const turned = turnedFrame(frame, values.quarter_turns);
  const { x, y, w, h } = values.rect;
  const ratioValue = (w * turned.w) / (h * turned.h);
  return {
    ratio: `${px(w * turned.w)} / ${px(h * turned.h)}`,
    ratioValue,
    frame: { width: percent(1 / w), height: percent(1 / h), left: percent(-x / w), top: percent(-y / h) },
    rotate: `rotate(${values.straighten ?? 0}deg)`,
    imageTransform: `rotate(${values.quarter_turns * 90}deg) scaleX(${values.flip === 'horizontal' ? -1 : 1})`,
    swapped: values.quarter_turns % 2 !== 0,
  };
}

/** A crop mark follows the backend's half-away-from-zero pixel rule. */
export function leavesFrameOut(values: CompositionValues, frame: Frame): boolean {
  const turned = turnedFrame(frame, values.quarter_turns);
  return (values.straighten ?? 0) !== 0 ||
    Math.round(values.rect.w * turned.w) < turned.w || Math.round(values.rect.h * turned.h) < turned.h;
}
