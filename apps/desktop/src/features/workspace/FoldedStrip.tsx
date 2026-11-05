import { PanelRightOpen, PanelsRightBottom } from 'lucide-react';
import { Hint } from '@/components/ui/tooltip';
import type { Photo } from '@/services/photos';
import { displayOrder } from './order';
import { PhotoGrid, type TileHandlers } from './PhotoGrid';

type FoldedStripProps = TileHandlers & { photos: readonly Photo[]; onUnfold: () => void; sessionsOpen: boolean; onToggleSessions: () => void };

/** The panel folded to a 64 px strip (F): thumbnails in display order with a "COND" divider before Condition. */
export function FoldedStrip({ photos, onUnfold, sessionsOpen, onToggleSessions, ...handlers }: FoldedStripProps) {
  const order = displayOrder(photos);
  return (
    <aside className="folded-strip" aria-label="Photo strip">
      <Hint text={{ name: 'Unfold panel', key: 'F', description: 'Opens the full photo panel beside the editor' }}><button type="button" className="stage-icon-button" onClick={onUnfold} aria-label="Unfold panel">
        <PanelRightOpen />
      </button></Hint>
      <Hint text="Sessions"><button type="button" className="stage-icon-button panel-sessions" onClick={onToggleSessions} aria-label="Sessions" aria-pressed={sessionsOpen}><PanelsRightBottom /></button></Hint>
      <PhotoGrid group="main" photos={[...order.main, ...order.unreviewed]} variant="strip" {...handlers} />
      <span className="strip-divider" role="separator">
        COND
      </span>
      <PhotoGrid group="condition" photos={order.condition} variant="strip" {...handlers} />
    </aside>
  );
}
