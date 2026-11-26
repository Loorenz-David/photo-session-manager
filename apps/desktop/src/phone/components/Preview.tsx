import { type Dispatch, type PointerEvent as ReactPointerEvent, useRef, useState } from 'react';
import { exampleUrl } from '../api';
import { isRotated, type LocalPoint } from '../geometry';
import { buzz, HAPTIC, overlayValue, SLIDE_PX } from '../gestures';
import { type Action, previewHeader, type PreviewSubject } from '../state';
import { OverlayBar } from './OverlayBar';
import { PhotoImage } from './PhotoImage';
import { ImageFrame, pointerLocal } from './Stage';

type Pointer = ReactPointerEvent<HTMLDivElement>;
type Props = {
  token: string;
  subject: PreviewSubject;
  /** The image's aspect ratio. */
  ratio: number;
  /** The template overlay, shared with the main screen. */
  overlay: number;
  dispatch: Dispatch<Action>;
  backLabel: string;
  onBack: () => void;
  /** Retake (a template shot's photo) or Delete (an extra or a condition image). */
  onAct: () => void;
  onMove: () => void;
};

/**
 * Design §4 and §Preview gestures: a captured image, full screen. A template shot's photo lies
 * over its template: hold shows the template through it, a sideways slide sets how much.
 */
export function Preview({ token, subject, ratio, overlay, dispatch, backLabel, onBack, onAct, onMove }: Props) {
  const header = previewHeader(subject, ratio);
  const hasTemplate = subject.kind === 'shot';
  const rotated = isRotated(ratio);
  const touch = useRef<{ rect: DOMRect; sliding: boolean; startOverlay: number; origin: LocalPoint } | null>(null);
  const [holding, setHolding] = useState(false);
  const [adjust, setAdjust] = useState<{ x: number; y: number; w: number } | null>(null);

  const down = (event: Pointer) => {
    // Extras and condition images have no template: nothing to hold for.
    if (!hasTemplate) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    const rect = event.currentTarget.getBoundingClientRect();
    touch.current = { rect, sliding: false, startOverlay: overlay, origin: pointerLocal(event, rect, rotated) };
    buzz(HAPTIC.hold);
    setHolding(true);
  };
  const move = (event: Pointer) => {
    const now = touch.current;
    if (!now) return;
    const at = pointerLocal(event, now.rect, rotated);
    const dx = at.x - now.origin.x;
    if (!now.sliding && Math.abs(dx) > SLIDE_PX && Math.abs(dx) > Math.abs(at.y - now.origin.y)) now.sliding = true;
    if (!now.sliding) return;
    const value = overlayValue(now.startOverlay, dx, at.w);
    if (Math.floor(value / 10) !== Math.floor(overlay / 10)) buzz(HAPTIC.overlayTick);
    dispatch({ type: 'overlay', overlay: value });
    setAdjust({ x: at.x, y: at.y, w: at.w });
  };
  const release = () => {
    touch.current = null;
    setHolding(false);
    setAdjust(null);
  };

  return <div className="ph-preview" data-testid="preview">
    <div className="ph-safe-top" />
    <div className="ph-preview-header">
      <div className="ph-preview-titles">
        <span className="ph-mono ph-kicker">{header.kicker}</span>
        <span className="ph-preview-title">{header.title}</span>
      </div>
      <span className="ph-mono ph-ratio-chip" data-testid="ratio-chip">{header.chip}</span>
    </div>
    <ImageFrame ratio={ratio} testId="preview-viewport" onPointerDown={down} onPointerMove={move} onPointerUp={release} onPointerCancel={release}>
      {hasTemplate && subject.shot.example && <img className="ph-cover ph-grey" alt="" draggable={false} src={exampleUrl(token, subject.shot.container_id)} />}
      <PhotoImage token={token} image={subject.image} className="ph-cover ph-preview-photo" testId="preview-photo" style={{ opacity: holding ? overlay / 100 : 1 }}>Preparing preview…</PhotoImage>
      <div className="ph-top-row">
        <div className={`ph-pill${holding ? '' : ' ph-pill-dark'}`} data-testid="preview-pill">{holding ? 'TEMPLATE' : header.mode}</div>
      </div>
      {adjust && <OverlayBar at={adjust} percent={overlay} label="PHOTO" />}
      {hasTemplate && <div className="ph-hint-row ph-preview-hint-row">
        <div className="ph-hint" style={{ opacity: holding ? 0 : 1 }}>Hold to view template · slide sideways for overlay</div>
      </div>}
      {hasTemplate && subject.shot.split && <div className="ph-split-line" style={{ left: '50%' }} />}
    </ImageFrame>
    <div className="ph-preview-actions">
      <button type="button" className="ph-button ph-button-outline ph-button-move" onClick={onMove}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M4 8h14l-4-4M20 16H6l4 4" /></svg>
        Move image
      </button>
      <div className="ph-rule" />
      <div className="ph-preview-pair">
        <button type="button" className="ph-button ph-button-outline" onClick={onBack}>{backLabel}</button>
        <button type="button" className="ph-button ph-button-solid" onClick={onAct}>{hasTemplate ? 'Retake' : 'Delete'}</button>
      </div>
    </div>
  </div>;
}
