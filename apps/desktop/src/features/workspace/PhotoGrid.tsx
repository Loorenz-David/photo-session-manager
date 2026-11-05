import { useDroppable } from '@dnd-kit/core';
import { SortableContext, rectSortingStrategy, verticalListSortingStrategy } from '@dnd-kit/sortable';
import type { Photo } from '@/services/photos';
import { outputNumbers } from './outputs';
import { PhotoContextMenu, type PhotoMenuAction } from './PhotoMenu';
import { PhotoTile, photoDragId } from './PhotoTile';

export type Group = 'main' | 'condition';
export const groupDropId = (group: Group) => `group:${group}`;

export type TileHandlers = {
  activeId: number | null;
  selection: ReadonlySet<number>;
  onOpen: (photoId: number) => void;
  onToggleSelected: (photoId: number) => void;
  onMenu: (action: PhotoMenuAction, photo: Photo) => void;
};

type PhotoGridProps = TileHandlers & {
  group: Group;
  photos: Photo[];
  variant: 'grid' | 'strip';
};

/**
 * One sortable group (Main with its unreviewed photos after it, or Condition). The group itself is a drop target,
 * so a photo can be dropped into an empty group. Tiles are numbered per output (MC-29): a split photo fills two places.
 */
export function PhotoGrid({ group, photos, variant, activeId, selection, onOpen, onToggleSelected, onMenu }: PhotoGridProps) {
  const { setNodeRef, isOver } = useDroppable({ id: groupDropId(group) });
  const numbers = outputNumbers(photos);
  return (
    <SortableContext
      id={group}
      items={photos.map(photo => photoDragId(photo.id))}
      strategy={variant === 'grid' ? rectSortingStrategy : verticalListSortingStrategy}
    >
      <div ref={setNodeRef} className={`photo-group group-${variant}`} data-group={group} data-over={isOver || undefined}>
        {photos.map((photo, index) => (
          <PhotoContextMenu key={photo.id} photo={photo} onAction={onMenu} canApply={selection.size > 0}>
            <div className="tile-wrap">
              <PhotoTile
                photo={photo}
                number={numbers.get(photo.id)?.first ?? index + 1}
                last={numbers.get(photo.id)?.last ?? index + 1}
                active={photo.id === activeId}
                selected={selection.has(photo.id)}
                variant={variant}
                onOpen={onOpen}
                onToggleSelected={onToggleSelected}
              />
            </div>
          </PhotoContextMenu>
        ))}
      </div>
    </SortableContext>
  );
}
