import { FolderInput, Info, PanelRightClose, PanelsRightBottom } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Hint } from '@/components/ui/tooltip';
import { useAction } from '@/lib/pending';
import type { Photo } from '@/services/photos';
import { templateBack, templateShiftBack, type TemplateView } from '@/services/templates';
import { displayOrder } from './order';
import { ContainerGrid } from './ContainerGrid';
import { PhotoGrid, type TileHandlers } from './PhotoGrid';
import { TemplateDiffersContext } from './PhotoTile';
import { TemplateInfoDialog } from './TemplateInfoDialog';
import { TemplateWarning } from './TemplateWarning';

type PhotoPanelProps = TileHandlers & {
  photos: readonly Photo[];
  templateView?: TemplateView | null;
  /** The session shown (defaults to the photos' session). */
  sessionId?: number;
  /** Back to template through the workspace's action runner (defaults to the plain command). */
  onTemplateBack?: () => Promise<unknown>;
  /** Shift the rest back through the workspace's action runner (defaults to the plain command). */
  onTemplateShiftBack?: () => Promise<unknown>;
  onFold: () => void;
  onImport: () => Promise<unknown>;
  sessionsOpen: boolean;
  onToggleSessions: () => void;
};

/** The photo panel (NOTES.md §Screens 4): Main images (unreviewed shown after Main), then Condition. */
export function PhotoPanel({ photos, templateView, sessionId, onTemplateBack, onTemplateShiftBack, onFold, onImport, sessionsOpen, onToggleSessions, ...handlers }: PhotoPanelProps) {
  const [infoOpen, setInfoOpen] = useState(false);
  const session = sessionId ?? photos[0]?.session_id;
  const back = onTemplateBack ?? (async () => { if (session !== undefined) await templateBack(session); });
  const shiftBack = onTemplateShiftBack ?? (async () => { if (session !== undefined) await templateShiftBack(session); });
  const differing = useMemo(
    () => new Set(templateView?.templated ? templateView.photos.filter(entry => entry.differs).map(entry => entry.photo_id) : []),
    [templateView],
  );
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
          <Hint text={{ name: 'Fold panel', key: 'F', description: 'Folds the panel to the strip' }}><button type="button" className="stage-icon-button" onClick={onFold} aria-label="Fold panel">
            <PanelRightClose />
          </button></Hint>
        </span>
      </div>
      <TemplateDiffersContext.Provider value={differing}>
      <div className="panel-scroll">
        {templateView?.templated && <TemplateWarning count={templateView.differs} onBack={back} canShiftBack={templateView.can_shift_back} onShiftBack={shiftBack} />}
        <section className="panel-section" aria-label="Main images">
          <h3 className="section-caps">
            Main images <span>{order.main.length}</span>
            {order.unreviewed.length > 0 && <small>{order.unreviewed.length} to review</small>}
            {templateView?.templated && <button type="button" className="template-info-button" aria-label="Show template" onClick={() => setInfoOpen(true)}><Info aria-hidden />Template</button>}
          </h3>
          {templateView?.templated ? <ContainerGrid photos={mainShown} view={templateView} {...handlers} /> : <PhotoGrid group="main" photos={mainShown} variant="grid" {...handlers} />}
        </section>
        <section className="panel-section" aria-label="Condition">
          <h3 className="section-caps">
            Condition <span>{order.condition.length}</span>
          </h3>
          <PhotoGrid group="condition" photos={order.condition} variant="grid" {...handlers} />
        </section>
      </div>
      </TemplateDiffersContext.Provider>
      {templateView?.templated && <TemplateInfoDialog view={templateView} open={infoOpen} onOpenChange={setInfoOpen} />}
    </aside>
  );
}
