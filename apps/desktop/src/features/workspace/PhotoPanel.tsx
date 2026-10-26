import { FolderInput, PanelRightClose } from 'lucide-react';
import { useAction } from '@/lib/pending';
import type { Photo } from '@/services/photos';
import { displayOrder } from './order';
import { PhotoGrid, type TileHandlers } from './PhotoGrid';

type PhotoPanelProps = TileHandlers & {
  photos: readonly Photo[];
  onFold: () => void;
  onImport: () => Promise<unknown>;
};

/** The photo panel (NOTES.md §Screens 4): Main images (unreviewed shown after Main), then Condition. */
export function PhotoPanel({ photos, onFold, onImport, ...handlers }: PhotoPanelProps) {
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
          <button type="button" className="stage-icon-button" onClick={() => void importing.run()} disabled={importing.busy} aria-busy={importing.busy ? 'true' : undefined} aria-label="Import photos" title="Import photos">
            <FolderInput />{importing.busy && <span aria-hidden>…</span>}
          </button>
          <button type="button" className="stage-icon-button" onClick={onFold} aria-label="Fold panel" title="Fold panel (F)">
            <PanelRightClose />
          </button>
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
