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
export function Stage({ photos, active, mode, onToggleMode, onStep, onAction, onMenu, onItemCard }: StageProps) {
  const toReview = displayOrder(photos).unreviewed.length;

  if (!active) {
    return (
      <section className="stage stage-empty" aria-label="Stage">
        <h1>Ready for the first photo</h1>
        <p>Shoot with the camera, or drop files here to import them.</p>
      </section>
    );
  }

  const where = placement(photos, active);
  return (
    <section className="stage" aria-label="Stage">
      <div className="stage-bar">
        <span className="stage-place">
          <strong>{where.label}</strong> {where.index} of {where.total}
          {toReview > 0 && <em>{toReview} to review</em>}
        </span>
        <span className="stage-tools">
          <PhotoMenuButton photo={active} onAction={onMenu} />
          <button type="button" className="stage-icon-button" onClick={onItemCard} aria-label="Item details" title="Item details (I)">
            <Info />
          </button>
        </span>
      </div>
      <div className="stage-body">
        <button type="button" className="stage-nav stage-prev" onClick={() => onStep(-1)} aria-label="Previous photo">
          <ChevronLeft />
        </button>
        <SwipeLayer key={active.id} onSwipe={onAction}>
          <StageImage key={active.id} photo={active} mode={mode} />
        </SwipeLayer>
        <button type="button" className="stage-nav stage-next" onClick={() => onStep(1)} aria-label="Next photo">
          <ChevronRight />
        </button>
      </div>
      <div className="stage-foot">
        <TogglePill mode={mode} processing={active.adjusted_preview_path === null} onToggle={onToggleMode} />
        <ReviewButtons
          photo={active}
          onDelete={() => onAction('delete')}
          onMiddle={() => onAction(active.classification === 'condition' ? 'approve' : 'condition')}
          onApprove={() => onAction('approve')}
        />
      </div>
    </section>
  );
}
