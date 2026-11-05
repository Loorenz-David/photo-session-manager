import type { MouseEvent, ReactNode } from 'react';
import { ChevronLeft, ChevronRight, Crop, Info } from 'lucide-react';
import { Hint } from '@/components/ui/tooltip';
import { canCompose, COMPOSE_WAITING } from '@/services/compositions';
import type { Photo } from '@/services/photos';
import { displayOrder } from './order';
import { outputNumbers, outputTotal } from './outputs';
import { PhotoMenuButton, type PhotoMenuAction } from './PhotoMenu';
import { ReviewButtons } from './ReviewButtons';
import { StageImage } from './StageImage';
import { usePendingCompositions } from '@/features/compose/useCompositionSaves';
import { SwipeLayer, type SwipeAction } from './SwipeLayer';
import { TogglePill } from './TogglePill';
import type { ViewMode } from './useWorkspaceState';

type StageProps = {
  photos: readonly Photo[];
  active: Photo | null;
  mode: ViewMode;
  onToggleMode: () => void;
  onStep: (delta: number) => void;
  onAction: (action: SwipeAction) => void;
  onMenu: (action: PhotoMenuAction, photo: Photo) => void;
  onItemCard: () => void;
  /** A double-click on the stage photo opens Compose (10D). */
  onCompose?: () => void;
  /** Compose mode (P31): the editor's stage replaces the swipe layer, the pill and the review buttons; Done finishes. */
  compose?: { onDone: () => void; children: ReactNode };
};

/**
 * Where the active photo sits, counted in outputs (MC-29): "Main 5 of 7" (unreviewed photos count after Main),
 * "Condition 1 of 2", or "Main 3–4 of 8" for a split photo (the two places it fills).
 */
function placement(photos: readonly Photo[], photo: Photo): { label: string; range: string; total: number } {
  const order = displayOrder(photos);
  const group = photo.classification === 'condition' ? order.condition : [...order.main, ...order.unreviewed];
  const own = outputNumbers(group).get(photo.id) ?? { first: 0, last: 0 };
  return { label: photo.classification === 'condition' ? 'Condition' : 'Main', range: own.first === own.last ? String(own.first) : `${own.first}–${own.last}`, total: outputTotal(group) };
}

/** The photo stage (COMPONENTS.md §Workspace): stacked images, prev/next, the pill and the round review buttons. */
export function Stage({ photos, active, mode, onToggleMode, onStep, onAction, onMenu, onItemCard, onCompose, compose }: StageProps) {
  const pending = usePendingCompositions();
  const toReview = displayOrder(photos).unreviewed.length;

  if (!active) {
    return (
      <section className="stage stage-empty" aria-label="Stage">
        <h1>Ready for the first photo</h1>
        <p>Shoot with the camera, or drop files here to import them.</p>
      </section>
    );
  }

  function onDoubleClick(event: MouseEvent) {
    if (!compose && onCompose && (event.target as Element).closest('.stage-frame')) onCompose();
  }

  const where = placement(photos, active);
  return (
    <section className="stage" aria-label="Stage" data-composing={compose ? 'true' : undefined}>
      <div className="stage-bar">
        <span className="stage-place">
          <strong>{where.label}</strong> {where.range} of {where.total}
          {toReview > 0 && <em>{toReview} to review</em>}
          {compose && pending.has(active.id) && <span className="tile-unsaved" aria-label="Unsaved composition" />}
        </span>
        <span className="stage-tools">
          {!compose && <Hint text={canCompose(active) ? 'Compose (R)' : COMPOSE_WAITING}>
            <button type="button" className="stage-compose" aria-label="Compose" aria-disabled={!canCompose(active) || undefined} onClick={onCompose}>
              <Crop /> Compose
            </button>
          </Hint>}
          {compose && (
            <button type="button" className="compose-done" onClick={compose.onDone} title="Done (Return)">
              Done
            </button>
          )}
          <PhotoMenuButton photo={active} onAction={onMenu} />
          <button type="button" className="stage-icon-button" onClick={onItemCard} aria-label="Item details" title="Item details (I)">
            <Info />
          </button>
        </span>
      </div>
      <div className="stage-body" onDoubleClick={onDoubleClick}>
        <button type="button" className="stage-nav stage-prev" onClick={() => onStep(-1)} aria-label="Previous photo">
          <ChevronLeft />
        </button>
        {compose ? (
          compose.children
        ) : (
          <SwipeLayer key={active.id} onSwipe={onAction}>
            <StageImage key={active.id} photo={active} mode={mode} />
          </SwipeLayer>
        )}
        <button type="button" className="stage-nav stage-next" onClick={() => onStep(1)} aria-label="Next photo">
          <ChevronRight />
        </button>
      </div>
      {!compose && (
        <div className="stage-foot">
          <TogglePill mode={mode} processing={active.adjusted_preview_path === null} onToggle={onToggleMode} />
          <ReviewButtons
            photo={active}
            onDelete={() => onAction('delete')}
            onMiddle={() => onAction(active.classification === 'condition' ? 'approve' : 'condition')}
            onApprove={() => onAction('approve')}
          />
        </div>
      )}
    </section>
  );
}
