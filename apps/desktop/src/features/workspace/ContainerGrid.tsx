import { useDroppable } from '@dnd-kit/core';
import { SortableContext, rectSortingStrategy } from '@dnd-kit/sortable';
import type { Photo } from '@/services/photos';
import type { TemplateView } from '@/services/templates';
import { outputNumbers } from './outputs';
import { groupDropId, type TileHandlers } from './PhotoGrid';
import { PhotoContextMenu } from './PhotoMenu';
import { PhotoTile, photoDragId } from './PhotoTile';

type Props = TileHandlers & { photos: Photo[]; view: TemplateView };

/** Final container order, then Main photos without a container and unreviewed photos. */
export function ContainerGrid({ photos, view, activeId, selection, onOpen, onToggleSelected, onMenu }: Props) {
  const { setNodeRef, isOver } = useDroppable({ id: groupDropId('main') });
  const main = photos.filter(photo => photo.classification === 'main');
  const unreviewed = photos.filter(photo => photo.classification === 'unreviewed');
  const slots = new Map(view.photos.map(photo => [photo.photo_id, photo.slot]));
  const ordered = [
    ...view.containers.flatMap(container => main.filter(photo => slots.get(photo.id) === container.final_no)),
    ...main.filter(photo => slots.get(photo.id) === undefined || slots.get(photo.id) === null),
    ...unreviewed,
  ];
  const numbers = outputNumbers(ordered);
  const tile = (photo: Photo, index: number) => <PhotoContextMenu key={photo.id} photo={photo} onAction={onMenu} canApply={selection.size > 0}>
    <div className="tile-wrap">
      <PhotoTile photo={photo} number={numbers.get(photo.id)?.first ?? index + 1} last={numbers.get(photo.id)?.last ?? index + 1} active={photo.id === activeId} selected={selection.has(photo.id)} variant="grid" onOpen={onOpen} onToggleSelected={onToggleSelected} />
    </div>
  </PhotoContextMenu>;
  let index = 0;
  return <SortableContext id="main" items={ordered.map(photo => photoDragId(photo.id))} strategy={rectSortingStrategy}>
    <div ref={setNodeRef} className="photo-group group-grid container-grid" data-group="main" data-over={isOver || undefined}>
      {view.containers.map(container => {
        const filled = main.filter(photo => slots.get(photo.id) === container.final_no);
        return filled.length ? filled.map(photo => tile(photo, index++)) : <div key={`empty-${container.final_no}`} className="container-placeholder" data-container={container.final_no} data-empty="true" aria-label={`Empty container ${container.final_no}`}>{container.final_no}</div>;
      })}
      {main.filter(photo => slots.get(photo.id) === undefined || slots.get(photo.id) === null).map(photo => tile(photo, index++))}
      {unreviewed.map(photo => tile(photo, index++))}
    </div>
  </SortableContext>;
}
