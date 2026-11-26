import { forwardRef, useImperativeHandle, useRef } from 'react';
import type { Frame } from '../socket';

export type LiveCanvasHandle = {
  /** Draws an accepted frame (PH7). Decoding is asynchronous: a newer frame replaces one still waiting. */
  draw: (frame: Frame) => void;
};

type Props = {
  /** Whether `gen` is still the current demand generation (a frame decoded too late is not drawn). */
  isCurrent: (gen: number) => boolean;
  /** A frame reached the canvas (reported when its generation or size is new). */
  onDrawn: (gen: number, width: number, height: number) => void;
  /** The live layer shows only once a frame of the current generation is drawn. */
  shown: boolean;
  /** The current shot's zoom preset: the picture is scaled about its centre like the template. */
  scale: number;
};

/**
 * The live picture: the camera's JPEG drawn as it arrived (H3: no processing), on a canvas that
 * stays mounted; modes switch by opacity only (design §Reliability).
 */
export const LiveCanvas = forwardRef<LiveCanvasHandle, Props>(function LiveCanvas({ isCurrent, onDrawn, shown, scale }, ref) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const waiting = useRef<Frame | null>(null);
  const busy = useRef(false);
  const reported = useRef('');
  const latest = useRef({ isCurrent, onDrawn });
  latest.current = { isCurrent, onDrawn };

  useImperativeHandle(ref, () => ({
    draw(frame) {
      waiting.current = frame;
      if (busy.current) return;
      busy.current = true;
      void (async () => {
        while (waiting.current) {
          const next = waiting.current;
          waiting.current = null;
          try {
            const bitmap = await createImageBitmap(new Blob([next.jpeg as BlobPart], { type: 'image/jpeg' }));
            const target = canvas.current;
            if (target && latest.current.isCurrent(next.gen)) {
              if (target.width !== bitmap.width) target.width = bitmap.width;
              if (target.height !== bitmap.height) target.height = bitmap.height;
              target.getContext('2d')?.drawImage(bitmap, 0, 0);
              const key = `${next.gen}:${bitmap.width}x${bitmap.height}`;
              if (key !== reported.current) {
                reported.current = key;
                latest.current.onDrawn(next.gen, bitmap.width, bitmap.height);
              }
            }
            bitmap.close();
          } catch {
            // A frame the browser cannot decode is skipped; the next one replaces it.
          }
        }
        busy.current = false;
      })();
    },
  }), []);

  return <canvas ref={canvas} className="ph-live" data-testid="live-layer" width={1024} height={680}
    style={{ opacity: shown ? 1 : 0, transform: `scale(${scale})` }} />;
});
