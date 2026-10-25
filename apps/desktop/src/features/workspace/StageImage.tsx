import { useEffect, useState, type CSSProperties } from 'react';
import { previewUrl } from '@/services/sessions';
import type { Photo } from '@/services/photos';
import type { ViewMode } from './useWorkspaceState';

/**
 * CSS transform for EXIF orientation 1–8 (MC-20: the embedded camera JPEG carries no orientation of its own,
 * so the UI applies the RAW's IFD0 orientation to the original preview only).
 */
export function orientationTransform(orientation: number | null): string | undefined {
  switch (orientation) {
    case 2:
      return 'scaleX(-1)';
    case 3:
      return 'rotate(180deg)';
    case 4:
      return 'scaleY(-1)';
    case 5:
      return 'rotate(90deg) scaleX(-1)';
    case 6:
      return 'rotate(90deg)';
    case 7:
      return 'rotate(270deg) scaleX(-1)';
    case 8:
      return 'rotate(270deg)';
    default:
      return undefined;
  }
}

function quarterTurn(orientation: number | null): boolean {
  return orientation !== null && orientation >= 5 && orientation <= 8;
}

type StageImageProps = { photo: Photo; mode: ViewMode };

/** Original ⇄ Adjusted crossfade (ch06: 700 ms), also used when a new render replaces the previous one. */
export const CROSSFADE_MS = 700;
const crossfade: CSSProperties = { transitionProperty: 'opacity', transitionDuration: `${CROSSFADE_MS}ms` };

/**
 * The stacked original and adjusted images (ch06 §Original / Adjusted). The adjusted layer fades over 700 ms;
 * a new render (a new path per generation) is laid over the previous one and fades in the same way.
 */
export function StageImage({ photo, mode }: StageImageProps) {
  const adjustedUrl = photo.adjusted_preview_path ? previewUrl(photo.adjusted_preview_path) : null;
  const originalUrl = photo.original_preview_path ? previewUrl(photo.original_preview_path) : null;
  // Adjusted renders shown for this photo, oldest first; the newest fades in over the one before it.
  const [layers, setLayers] = useState<string[]>(adjustedUrl ? [adjustedUrl] : []);
  const [loaded, setLoaded] = useState<ReadonlySet<string>>(new Set());

  useEffect(() => {
    if (!adjustedUrl) return;
    setLayers(current => (current[current.length - 1] === adjustedUrl ? current : [...current.slice(-1), adjustedUrl]));
  }, [adjustedUrl]);

  const newest = layers[layers.length - 1];
  // The original stays up until an adjusted render has actually loaded, so nothing flashes empty.
  const showAdjusted = mode === 'adjusted' && layers.some(url => loaded.has(url));
  const originalStyle: CSSProperties = { ...crossfade, transform: orientationTransform(photo.orientation) };

  return (
    <div className="stage-frame">
      {originalUrl ? (
        <img
          className="stage-layer stage-original crossfade"
          data-quarter-turn={quarterTurn(photo.orientation) || undefined}
          data-visible={!showAdjusted}
          src={originalUrl}
          alt="Camera original"
          style={originalStyle}
          draggable={false}
        />
      ) : (
        !showAdjusted && <div className="stage-processing">Processing</div>
      )}
      {layers.map(url => (
        <img
          key={url}
          className="stage-layer stage-adjusted crossfade"
          data-visible={mode === 'adjusted' && loaded.has(url)}
          data-newest={url === newest || undefined}
          src={url}
          alt="Adjusted"
          style={crossfade}
          draggable={false}
          onLoad={() => {
            setLoaded(current => new Set(current).add(url));
            // Once the newest render is in, older layers can go after the crossfade.
            window.setTimeout(() => setLayers(current => current.slice(current.indexOf(url))), CROSSFADE_MS);
          }}
        />
      ))}
    </div>
  );
}
