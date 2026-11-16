import { useDroppable } from '@dnd-kit/core';
import { Info } from 'lucide-react';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { SortableContext, rectSortingStrategy } from '@dnd-kit/sortable';
import { TemplateExample } from '@/features/settings/TemplateExample';
import type { Photo } from '@/services/photos';
import type { TemplateView } from '@/services/templates';
import { outputNumbers } from './outputs';
import { groupDropId, type TileHandlers } from './PhotoGrid';
import { PhotoContextMenu } from './PhotoMenu';
import { PhotoTile, photoDragId } from './PhotoTile';
import { containerDropId } from './templateDrop';

type Props = TileHandlers & { photos: Photo[]; view: TemplateView };

type ContainerView = TemplateView['containers'][number];

/**
 * A faded numbered empty container; a photo dropped on it goes into that container (10G §6.1). It shows its example
 * (black and white) and its place in the shooting order; the container the next shot goes to is outlined ("Next").
 */
function Placeholder({ container, next }: { container: ContainerView; next: boolean }) {
  const number = container.final_no;
  const { setNodeRef, isOver } = useDroppable({ id: containerDropId(number) });
  return (
    <div
      ref={setNodeRef}
      className="container-placeholder"
      data-container={number}
      data-empty="true"
      data-next={next || undefined}
      data-over={isOver || undefined}
      aria-label={`Empty container ${number}${container.name ? ` (${container.name})` : ''}, shot ${container.shoot_no}${next ? ', next to shoot' : ''}`}
    >
      {container.example_path && <TemplateExample path={container.example_path} />}
      <span className="container-number">{number}</span>
      {container.name && <span className="container-name">{container.name}</span>}
      <span className="container-shot">{next ? `Next · Shot ${container.shoot_no}` : `Shot ${container.shoot_no}`}</span>
      {container.description && <Popover>
        <PopoverTrigger asChild>
          <button type="button" className="container-info" aria-label={`How to take shot ${container.shoot_no}`}><Info aria-hidden /></button>
        </PopoverTrigger>
        <PopoverContent align="end" className="container-info-text">
          <strong>{container.name ?? `Container ${number}`}</strong>
          <p>{container.description}</p>
        </PopoverContent>
      </Popover>}
    </div>
  );
}

type Cell = { kind: 'photo'; photo: Photo; slot: number | null } | { kind: 'empty'; container: ContainerView };

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
    return filled.length ? filled.map(photo => ({ kind: 'photo', photo, slot: container.final_no })) : [{ kind: 'empty', container }];
  });
  const containerNumbers = new Set(view.containers.map(container => container.final_no));
  const rest = photos.filter(photo => photo.classification !== 'main' || !containerNumbers.has(slotOf(photo) ?? -1));
  cells.push(...rest.map(photo => ({ kind: 'photo' as const, photo, slot: null })));
  // The container the next shot goes to (10G §5 guidance); none once every container has had its shot.
  const nextFinal = view.containers.find(container => container.shoot_no === view.next_shot)?.final_no ?? null;
  const ordered = cells.flatMap(cell => (cell.kind === 'photo' ? [cell.photo] : []));
  const numbers = outputNumbers(ordered);
  return (
    <SortableContext id="main" items={ordered.map(photo => photoDragId(photo.id))} strategy={rectSortingStrategy}>
      <div ref={setNodeRef} className="photo-group group-grid container-grid" data-group="main" data-over={isOver || undefined}>
        {cells.map((cell, index) => cell.kind === 'empty' ? (
          <Placeholder key={`empty-${cell.container.final_no}`} container={cell.container} next={cell.container.final_no === nextFinal} />
        ) : (
          <PhotoContextMenu key={cell.photo.id} photo={cell.photo} onAction={onMenu} canApply={selection.size > 0}>
            <div className="tile-wrap" data-next={(cell.slot !== null && cell.slot === nextFinal) || undefined}>
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
