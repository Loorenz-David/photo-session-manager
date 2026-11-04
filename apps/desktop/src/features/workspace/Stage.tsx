import type { MouseEvent, ReactNode } from 'react';
import { ChevronLeft, ChevronRight, Info } from 'lucide-react';
import type { Photo } from '@/services/photos';
import { displayOrder } from './order';
import { PhotoMenuButton, type PhotoMenuAction } from './PhotoMenu';
import { ReviewButtons } from './ReviewButtons';
import { StageImage } from './StageImage';
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

/** Where the active photo sits: "Main 5 of 7" (unreviewed photos count after Main) or "Condition 1 of 2". */
function placement(photos: readonly Photo[], photo: Photo): { label: string; index: number; total: number } {
  const order = displayOrder(photos);
  if (photo.classification === 'condition') {
    return { label: 'Condition', index: order.condition.indexOf(photo) + 1, total: order.condition.length };
  }
  const shown = [...order.main, ...order.unreviewed];
  return { label: 'Main', index: shown.indexOf(photo) + 1, total: shown.length };
}

/** The photo stage (COMPONENTS.md §Workspace): stacked images, prev/next, the pill and the round review buttons. */
export function Stage({ photos, active, mode, onToggleMode, onStep, onAction, onMenu, onItemCard, onCompose, compose }: StageProps) {
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
          <strong>{where.label}</strong> {where.index} of {where.total}
          {toReview > 0 && <em>{toReview} to review</em>}
        </span>
        <span className="stage-tools">
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
