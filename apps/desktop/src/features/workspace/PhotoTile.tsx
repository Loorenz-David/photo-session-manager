import { createContext, useContext, type CSSProperties, type MouseEvent } from 'react';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { turnedFrame } from '@/features/compose/geometry';
import { usePendingCompositions } from '@/features/compose/useCompositionSaves';
import { valuesOf } from '@/services/compositions';
import { previewUrl } from '@/services/sessions';
import type { Photo } from '@/services/photos';
import { orientationTransform } from './StageImage';
import { outputLabel } from './outputs';
import { PhotoUploadBadge } from '@/features/publish/PhotoUploadBadge';

export const photoDragId = (photoId: number) => `photo:${photoId}`;
export const PublishFailedContext = createContext(false);

type ThumbProps = { photo: Photo; transform?: string };

/** Thumbnail: the 640 px thumbnail (MC-28) or the adjusted preview when it exists, else the camera original with its orientation (MC-20). */
export function Thumb({ photo, transform }: ThumbProps) {
  const developed = photo.thumbnail_path ?? photo.adjusted_preview_path;
  if (developed) {
    return <img className="thumb-image" src={previewUrl(developed)} alt="" draggable={false} style={transform ? { transform } : undefined} />;
  }
  if (photo.original_preview_path) {
    return (
      <img
        className="thumb-image"
        src={previewUrl(photo.original_preview_path)}
        alt=""
        draggable={false}
        style={{ transform: orientationTransform(photo.orientation) }}
      />
    );
  }
  return <span className="thumb-empty" aria-hidden />;
}

const percent = (fraction: number) => `${Number((fraction * 100).toFixed(4))}%`;

/**
 * The thumbnail in its turned frame with the composition drawn over it in CSS (MC-28 §Thumbnail); a split photo also
 * shows a hairline at the middle of its rect (10D: "In the grid a split photo shows a middle hairline").
 */
function TileComposition({ photo }: { photo: Photo }) {
  const values = valuesOf(photo.composition);
  const frame = photo.composition.frame;
  if (!values || !frame) return <Thumb photo={photo} />;
  const turned = turnedFrame(frame, values.quarter_turns);
  const style = { aspectRatio: `${turned.w} / ${turned.h}`, '--tile-ratio': String(turned.w / turned.h) } as CSSProperties;
  const { rect } = values;
  return (
    <div className="tile-composition" data-turned={values.quarter_turns % 2 === 1 || undefined} style={style}>
      <Thumb photo={photo} transform={`rotate(${values.quarter_turns * 90}deg) scaleX(${values.flip === 'horizontal' ? -1 : 1})`} />
      <span className="tile-frame" style={{ left: percent(rect.x), top: percent(rect.y), width: percent(rect.w), height: percent(rect.h) }} aria-hidden />
      {values.kind === 'split' && <span className="tile-split" style={{ left: percent(rect.x + rect.w / 2), top: percent(rect.y), height: percent(rect.h) }} aria-hidden />}
    </div>
  );
}

type PhotoTileProps = {
  photo: Photo;
  /** Display number in its section (01 = cover in Main): the first place the photo fills. */
  number: number;
  /** The last place it fills (`number + 1` for a split photo; defaults to `number`). */
  last?: number;
  active: boolean;
  selected: boolean;
  variant: 'grid' | 'strip';
  onOpen: (photoId: number) => void;
  onToggleSelected: (photoId: number) => void;
};

/** The same visible tile contents in the grid and in its drag overlay; the number reads the places the photo fills (`03` or `03–04`). */
export function PhotoTileFace({ photo, number, last = number, variant }: Pick<PhotoTileProps, 'photo' | 'number' | 'last' | 'variant'>) {
  const pending = usePendingCompositions().has(photo.id);
  const unreviewed = photo.classification === 'unreviewed';
  const isCover = photo.classification === 'main' && number === 1;
  const failed = useContext(PublishFailedContext);
  const composed = photo.composition.rect !== null && photo.composition.frame !== null;
  return <>
    {composed ? <TileComposition photo={photo} /> : <Thumb photo={photo} />}
    {variant === 'grid' && <>
      <span className="tile-number">{outputLabel(number, last)}</span>
      {isCover && <span className="tile-cover">Cover</span>}
      {!photo.adjusted_preview_path && <span className="tile-processing">Processing</span>}
      {(photo.upload_state || failed) && <PhotoUploadBadge state={photo.upload_state ?? 'none'} />}
      {photo.composition.state === 'needs_review' && <span className="tile-review">Re-frame</span>}
    </>}
    {unreviewed && <span className="tile-unreviewed" aria-hidden />}
    {pending && <span className="tile-unsaved" aria-label="Unsaved composition" />}
  </>;
}

/** One sortable photo in the grid or the folded strip. ⌘-click adds it to the selection. */
export function PhotoTile({ photo, number, last = number, active, selected, variant, onOpen, onToggleSelected }: PhotoTileProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: photoDragId(photo.id) });
  const unreviewed = photo.classification === 'unreviewed';

  function onClick(event: MouseEvent) {
    if (event.metaKey || event.shiftKey) onToggleSelected(photo.id);
    else onOpen(photo.id);
  }

  return (
    <div
      ref={setNodeRef}
      className={`photo-tile tile-${variant}`}
      data-photo-id={photo.id}
      data-active={active || undefined}
      data-selected={selected || undefined}
      data-dragging={isDragging || undefined}
      aria-current={active ? 'true' : undefined}
      aria-label={`Photo ${outputLabel(number, last)}${unreviewed ? ', to review' : ''}`}
      style={{ transform: CSS.Transform.toString(transform), transition, opacity: isDragging ? 0 : 1 }}
      onClick={onClick}
      {...attributes}
      {...listeners}
      role="button"
    >
      <PhotoTileFace photo={photo} number={number} last={last} variant={variant} />
    </div>
  );
}
