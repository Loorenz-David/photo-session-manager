import { TEXT } from '../state';

type Props = {
  open: boolean;
  /** What the Preview's header used to show: "SHOT 02 · CAPTURED", the shot's name, "1:1". */
  kicker: string;
  title: string;
  chip: string;
  /** A template shot's photo: it lies over its template (the hold hint) and can be retaken. */
  hasTemplate: boolean;
  /** The Mac has the photo's full-frame preview, so it can be reframed (PH6c); else Reframe waits and says why. */
  canReframe: boolean;
  onClose: () => void;
  onRetake: () => void;
  onReframe: () => void;
};

/** master_plan_live §6 (v1.3): how the template shows through a template shot's photo. */
const HOLD_HINT = 'Hold to view template · slide sideways for overlay';

/**
 * Book 10J LV-D21 (4), master_plan_live §5 PH10: the photo's details and actions, in a bottom sheet
 * behind the Preview's ⋯ button (the Item drawer's sheet look and motion). The actions are a column
 * that takes more buttons: Retake for a template shot's photo, Reframe for every photo (LV-D21 (6)).
 */
export function PhotoDrawer({ open, kicker, title, chip, hasTemplate, canReframe, onClose, onRetake, onReframe }: Props) {
  return <>
    <div className="ph-backdrop ph-backdrop-photo" data-testid="photo-backdrop" style={{ opacity: open ? 1 : 0, pointerEvents: open ? 'auto' : 'none' }} onClick={onClose} />
    <div className="ph-sheet ph-sheet-photo" role="dialog" aria-label="Photo details" aria-hidden={!open} data-testid="photo-drawer" style={{ transform: `translateY(${open ? 0 : 105}%)` }}>
      <div className="ph-grab" />
      <div className="ph-item-head">
        <div className="ph-item-titles">
          <span className="ph-mono ph-kicker">{kicker}</span>
          <span className="ph-item-title">{title}</span>
        </div>
        <button type="button" className="ph-close" aria-label="Close" onClick={onClose}>×</button>
      </div>
      <div className="ph-photo-facts">
        <span className="ph-mono ph-ratio-chip" data-testid="ratio-chip">{chip}</span>
        {hasTemplate && <span className="ph-photo-hint">{HOLD_HINT}</span>}
      </div>
      <div className="ph-item-actions">
        <span className="ph-mono ph-section">ACTIONS</span>
        {hasTemplate && <button type="button" className="ph-button ph-button-solid" onClick={onRetake}>Retake</button>}
        <button type="button" className="ph-button ph-button-outline" disabled={!canReframe} onClick={onReframe}>Reframe</button>
        {!canReframe && <span className="ph-action-why">{TEXT.reframeWaiting}</span>}
      </div>
    </div>
  </>;
}
