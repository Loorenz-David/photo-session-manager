import { useDroppable } from '@dnd-kit/core';
import { SortableContext, rectSortingStrategy } from '@dnd-kit/sortable';
import type { Photo } from '@/services/photos';
import type { TemplateView } from '@/services/templates';
import { outputNumbers } from './outputs';
import { groupDropId, type TileHandlers } from './PhotoGrid';
import { PhotoContextMenu } from './PhotoMenu';
import { PhotoTile, photoDragId } from './PhotoTile';
import { containerDropId } from './templateDrop';

type Props = TileHandlers & { photos: Photo[]; view: TemplateView };

/** A faded numbered empty container; a photo dropped on it goes into that container (10G §6.1). */
function Placeholder({ number }: { number: number }) {
  const { setNodeRef, isOver } = useDroppable({ id: containerDropId(number) });
  return (
    <div ref={setNodeRef} className="container-placeholder" data-container={number} data-empty="true" data-over={isOver || undefined} aria-label={`Empty container ${number}`}>
      {number}
    </div>
  );
}

type Cell = { kind: 'photo'; photo: Photo } | { kind: 'empty'; number: number };

/**
 * The Main area of a templated session (10G §5): the containers in final order, each with its Main photos (today's
 * tiles) or, when none, a faded numbered placeholder; then the Main photos without a container and the unreviewed
 * ones. Placeholders exist only here: they are not photos, not selectable, not counted and never exported.
 */
export function ContainerGrid({ photos, view, activeId, selection, onOpen, onToggleSelected, onMenu }: Props) {
  const { setNodeRef, isOver } = useDroppable({ id: groupDropId('main') });
  const slots = new Map(view.photos.map(entry => [entry.photo_id, entry.slot]));
  const main = photos.filter(photo => photo.classification === 'main');
  const slotOf = (photo: Photo) => slots.get(photo.id) ?? null;
  const cells: Cell[] = view.containers.flatMap((container): Cell[] => {
    const filled = main.filter(photo => slotOf(photo) === container.final_no);
    return filled.length ? filled.map(photo => ({ kind: 'photo', photo })) : [{ kind: 'empty', number: container.final_no }];
  });
  const containerNumbers = new Set(view.containers.map(container => container.final_no));
  const rest = photos.filter(photo => photo.classification !== 'main' || !containerNumbers.has(slotOf(photo) ?? -1));
  cells.push(...rest.map(photo => ({ kind: 'photo' as const, photo })));
  const ordered = cells.flatMap(cell => (cell.kind === 'photo' ? [cell.photo] : []));
  const numbers = outputNumbers(ordered);
  return (
    <SortableContext id="main" items={ordered.map(photo => photoDragId(photo.id))} strategy={rectSortingStrategy}>
      <div ref={setNodeRef} className="photo-group group-grid container-grid" data-group="main" data-over={isOver || undefined}>
        {cells.map((cell, index) => cell.kind === 'empty' ? (
          <Placeholder key={`empty-${cell.number}`} number={cell.number} />
        ) : (
          <PhotoContextMenu key={cell.photo.id} photo={cell.photo} onAction={onMenu} canApply={selection.size > 0}>
            <div className="tile-wrap">
              <PhotoTile
                photo={cell.photo}
                number={numbers.get(cell.photo.id)?.first ?? index + 1}
                last={numbers.get(cell.photo.id)?.last ?? index + 1}
                active={cell.photo.id === activeId}
                selected={selection.has(cell.photo.id)}
                variant="grid"
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
