import { useState, type CSSProperties } from 'react';
import { defaultRect } from '@/features/compose/geometry';
import type { Frame, Rect } from '@/services/compositions';
import { previewUrl } from '@/services/sessions';
import type { Presets } from '@/services/templates';

/** `mechanisms::template::zoomed`: the rect shrunk about its centre by `zoom` %. */
export function zoomedRect(rect: Rect, zoom: number): Rect {
  const factor = 1 - zoom / 100;
  const w = rect.w * factor;
  const h = rect.h * factor;
  return { x: rect.x + (rect.w - w) / 2, y: rect.y + (rect.h - h) / 2, w, h };
}

/** The frame a container's presets give a photo shaped like its example (`preset_values`; no shape = the whole photo). */
export function presetRect(presets: Presets, frame: Frame): Rect {
  return zoomedRect(defaultRect(presets.split ? 'split' : 'normal', presets.shape ?? 'original', 'named', frame), presets.zoom);
}

const pct = (v: number) => `${v * 100}%`;
/** The whole example placed so that `part` fills its box (the session's output preview, CSS only). */
const frameStyle = (part: Rect): CSSProperties => ({
  width: pct(1 / part.w), height: pct(1 / part.h), left: pct(-part.x / part.w), top: pct(-part.y / part.h),
});

/**
 * The container's example with its presets applied (owner 2026-11-16): the crop drawn on the example (dimmed outside,
 * the split's middle dashed) and the result as the session shows it, one picture or the two square halves of a Split.
 */
export function PresetPreview({ path, presets }: { path: string | null; presets: Presets }) {
  const [frame, setFrame] = useState<Frame>({ w: 3, h: 2 });
  if (!path) return <div className="template-example preset-preview-empty" data-bw="true"><span>No example photo</span></div>;
  const src = previewUrl(path);
  const rect = presetRect(presets, frame);
  const halves: Rect[] = presets.split
    ? [{ ...rect, w: rect.w / 2 }, { ...rect, x: rect.x + rect.w / 2, w: rect.w / 2 }]
    : [rect];
  const resultRatio = (rect.w * frame.w) / (rect.h * frame.h);
  return <div className="preset-preview" data-bw="true">
    <div className="preset-source" style={{ aspectRatio: `${frame.w} / ${frame.h}` }}>
      <img src={src} alt="Example angle" draggable={false}
        onLoad={event => { const img = event.currentTarget; if (img.naturalWidth && img.naturalHeight) setFrame({ w: img.naturalWidth, h: img.naturalHeight }); }} />
      <div className="preset-crop" data-split={presets.split || undefined}
        style={{ left: pct(rect.x), top: pct(rect.y), width: pct(rect.w), height: pct(rect.h) }} />
    </div>
    <div className="preset-result" aria-label={presets.split ? 'Result: Left and Right' : 'Result'}>
      {halves.map((part, index) => <div key={index} className="preset-result-picture"
        style={{ aspectRatio: presets.split ? '1 / 1' : `${resultRatio}` }} data-side={presets.split ? (index === 0 ? 'left' : 'right') : undefined}>
        <div className="preset-result-frame" style={frameStyle(part)}><img src={src} alt="" draggable={false} /></div>
      </div>)}
    </div>
  </div>;
}
