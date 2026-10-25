import { PanelRightOpen } from 'lucide-react';
import type { Photo } from '@/services/photos';
import { displayOrder } from './order';
import { PhotoGrid, type TileHandlers } from './PhotoGrid';

type FoldedStripProps = TileHandlers & { photos: readonly Photo[]; onUnfold: () => void };

/** The panel folded to a 64 px strip (F): thumbnails in display order with a "COND" divider before Condition. */
export function FoldedStrip({ photos, onUnfold, ...handlers }: FoldedStripProps) {
  const order = displayOrder(photos);
  return (
    <aside className="folded-strip" aria-label="Photo strip">
      <button type="button" className="stage-icon-button" onClick={onUnfold} aria-label="Unfold panel" title="Unfold panel (F)">
        <PanelRightOpen />
      </button>
      <PhotoGrid group="main" photos={[...order.main, ...order.unreviewed]} variant="strip" {...handlers} />
      <span className="strip-divider" role="separator">
        COND
      </span>
      <PhotoGrid group="condition" photos={order.condition} variant="strip" {...handlers} />
    </aside>
  );
}
