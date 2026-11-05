import { FolderInput, PanelRightClose, PanelsRightBottom } from 'lucide-react';
import { Hint } from '@/components/ui/tooltip';
import { useAction } from '@/lib/pending';
import type { Photo } from '@/services/photos';
import { displayOrder } from './order';
import { PhotoGrid, type TileHandlers } from './PhotoGrid';

type PhotoPanelProps = TileHandlers & {
  photos: readonly Photo[];
  onFold: () => void;
  onImport: () => Promise<unknown>;
  sessionsOpen: boolean;
  onToggleSessions: () => void;
};

/** The photo panel (NOTES.md §Screens 4): Main images (unreviewed shown after Main), then Condition. */
export function PhotoPanel({ photos, onFold, onImport, sessionsOpen, onToggleSessions, ...handlers }: PhotoPanelProps) {
  const importing = useAction(onImport);
  const order = displayOrder(photos);
  const mainShown = [...order.main, ...order.unreviewed];
  return (
    <aside className="photo-panel" aria-label="Photos">
      <div className="panel-head">
        <span className="panel-title">Photos</span>
        <span className="panel-meta">
          {order.main.length} main · {order.condition.length} condition
        </span>
        <span className="panel-tools">
          <Hint text="Sessions"><button type="button" className="stage-icon-button panel-sessions" onClick={onToggleSessions} aria-label="Sessions" aria-pressed={sessionsOpen}><PanelsRightBottom /></button></Hint>
          <Hint text="Import photos"><button type="button" className="stage-icon-button" onClick={() => void importing.run()} aria-disabled={importing.busy || undefined} aria-busy={importing.busy ? 'true' : undefined} aria-label="Import photos">
            <FolderInput />{importing.busy && <span aria-hidden>…</span>}
          </button></Hint>
          <Hint text="Fold panel (F)"><button type="button" className="stage-icon-button" onClick={onFold} aria-label="Fold panel">
            <PanelRightClose />
          </button></Hint>
        </span>
      </div>
      <div className="panel-scroll">
        <section className="panel-section" aria-label="Main images">
          <h3 className="section-caps">
            Main images <span>{order.main.length}</span>
            {order.unreviewed.length > 0 && <small>{order.unreviewed.length} to review</small>}
          </h3>
          <PhotoGrid group="main" photos={mainShown} variant="grid" {...handlers} />
        </section>
        <section className="panel-section" aria-label="Condition">
          <h3 className="section-caps">
            Condition <span>{order.condition.length}</span>
          </h3>
          <PhotoGrid group="condition" photos={order.condition} variant="grid" {...handlers} />
        </section>
      </div>
    </aside>
  );
}
