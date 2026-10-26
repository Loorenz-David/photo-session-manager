import { createContext, useContext, type MouseEvent } from 'react';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { previewUrl } from '@/services/sessions';
import type { Photo } from '@/services/photos';
import { orientationTransform } from './StageImage';
import { PhotoUploadBadge } from '@/features/publish/PhotoUploadBadge';

export const photoDragId = (photoId: number) => `photo:${photoId}`;
export const PublishFailedContext = createContext(false);

type ThumbProps = { photo: Photo };

/** Thumbnail: the adjusted preview when it exists, else the camera original with its orientation (MC-20). */
export function Thumb({ photo }: ThumbProps) {
  if (photo.adjusted_preview_path) {
    return <img className="thumb-image" src={previewUrl(photo.adjusted_preview_path)} alt="" draggable={false} />;
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

type PhotoTileProps = {
  photo: Photo;
  /** Display number in its section (01 = cover in Main). */
  number: number;
  active: boolean;
  selected: boolean;
  variant: 'grid' | 'strip';
  onOpen: (photoId: number) => void;
  onToggleSelected: (photoId: number) => void;
};

/** One sortable photo in the grid or the folded strip. ⌘-click adds it to the selection. */
export function PhotoTile({ photo, number, active, selected, variant, onOpen, onToggleSelected }: PhotoTileProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: photoDragId(photo.id) });
  const unreviewed = photo.classification === 'unreviewed';
  const isCover = photo.classification === 'main' && number === 1;
  const failed = useContext(PublishFailedContext);

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
      aria-label={`Photo ${String(number).padStart(2, '0')}${unreviewed ? ', to review' : ''}`}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      onClick={onClick}
      {...attributes}
      {...listeners}
      role="button"
    >
      <Thumb photo={photo} />
      {variant === 'grid' && (
        <>
          <span className="tile-number">{String(number).padStart(2, '0')}</span>
          {isCover && <span className="tile-cover">Cover</span>}
          {!photo.adjusted_preview_path && <span className="tile-processing">Processing</span>}
          {(photo.upload_state || failed) && <PhotoUploadBadge state={photo.upload_state ?? 'none'} />}
        </>
      )}
      {unreviewed && <span className="tile-unreviewed" aria-hidden />}
    </div>
  );
}
