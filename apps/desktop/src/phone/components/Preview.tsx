import { type CSSProperties, type Dispatch, type PointerEvent as ReactPointerEvent, type ReactNode, useCallback, useEffect, useRef, useState } from 'react';
import { exampleUrl, previewUrl } from '../api';
import { type SwipeAction, swipeAction, type SwipeDirection } from '../arrange';
import { isRotated, type LocalPoint } from '../geometry';
import { buzz, HAPTIC, overlayValue, SLIDE_PX, SWIPE } from '../gestures';
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
  /** The picture takes touches (the photo drawer is not open over it). */
  enabled: boolean;
  /** × : back to the live screen. */
  onClose: () => void;
  /** ⋯ : the photo's details and actions (the photo drawer). */
  onDetails: () => void;
  /** A swipe that counted (never `none`): the app asks the Mac and shows the next photo. */
  onSwipe: (action: Exclude<SwipeAction, 'none'>) => void;
  /** The photo strip, between the picture and the two buttons (LV-D20 (2)). */
  strip: ReactNode;
};

/**
 * One finger on the picture (PH9.1): `pressing` until it either moves (`swiping`, the axis decided
 * once) or, on a template shot's photo, rests for `SWIPE.holdMs` (`holding`: the template shows,
 * sideways sets the overlay, and no swipe happens until release).
 */
type Touch = {
  pointerId: number;
  rect: DOMRect;
  origin: LocalPoint;
  startOverlay: number;
  phase: 'pressing' | 'swiping' | 'holding';
  /** Swiping: along the picture's width, upwards, or downwards (which does nothing). */
  axis: 'x' | 'up' | 'down';
  /** Holding: the sideways slide that sets the overlay has started. */
  sliding: boolean;
  timer: ReturnType<typeof setTimeout> | null;
};
/** How far the picture is swiped: `d` px along the picture's width (signed) or upwards (≥ 0). */
type Swipe = { axis: 'x' | 'up'; d: number };
/** The swiped picture leaving, drawn over the next one for a moment (never delays it). */
type FlyOut = { n: number; src: string; from: string; to: string };

const FLY_OUT_MS = 220;
/** A swipe that will do nothing follows the finger only this much. */
const RESIST = 0.35;

const directionOf = (swipe: Swipe): SwipeDirection => (swipe.axis === 'up' ? 'up' : swipe.d < 0 ? 'left' : 'right');
const hintOf = (action: SwipeAction, direction: SwipeDirection): string => {
  if (action === 'none') return direction === 'right' ? 'ALREADY MAIN' : 'ALREADY CONDITION';
  if (action === 'retake' || action === 'delete') return 'DENY';
  return action === 'condition' ? 'CONDITION' : 'MAIN';
};
const shift = (swipe: Swipe, by: number) => (swipe.axis === 'up' ? `translateY(${-by}px)` : `translateX(${Math.sign(swipe.d) * by}px) rotate(${(Math.sign(swipe.d) * by) / 24}deg)`);

/**
 * Design §Preview gestures, with book 10J LV-D20 and LV-D21 (4) (master_plan_live §5 PH10): a
 * captured image, full screen, and nothing else at first glance — the photo, its strip, ⋯ (bottom
 * left: the photo's details and actions) and × (bottom right). Swipe the photo left to deny, right
 * for Main, up for Condition, and the next photo shows. A template shot's photo lies over its
 * template: hold shows the template through it, a sideways slide sets how much; the "TEMPLATE"
 * pill and the overlay readout show only during that gesture. The root is named by what the photo
 * is ("SHOT 02 · CAPTURED").
 */
export function Preview({ token, subject, ratio, overlay, dispatch, enabled, onClose, onDetails, onSwipe, strip }: Props) {
  const header = previewHeader(subject, ratio);
  const hasTemplate = subject.kind === 'shot';
  const rotated = isRotated(ratio);
  const touch = useRef<Touch | null>(null);
  const [holding, setHolding] = useState(false);
  const [adjust, setAdjust] = useState<{ x: number; y: number; w: number } | null>(null);
  const [swipe, setSwipe] = useState<Swipe | null>(null);
  const [flyOut, setFlyOut] = useState<FlyOut | null>(null);

  /** Every way a touch ends without acting: the picture springs back, the template hides. */
  const reset = useCallback(() => {
    if (touch.current?.timer) clearTimeout(touch.current.timer);
    touch.current = null;
    setHolding(false);
    setAdjust(null);
    setSwipe(null);
  }, []);

  // Design §Reliability: a touch never outlives the page's attention, and never carries over to another photo.
  const photoId = subject.image.photo_id;
  useEffect(() => reset, [photoId, reset]);
  useEffect(() => {
    const hidden = () => {
      if (document.visibilityState === 'hidden') reset();
    };
    window.addEventListener('blur', reset);
    document.addEventListener('visibilitychange', hidden);
    return () => {
      window.removeEventListener('blur', reset);
      document.removeEventListener('visibilitychange', hidden);
    };
  }, [reset]);
  useEffect(() => {
    if (!flyOut) return undefined;
    const timer = setTimeout(() => setFlyOut(null), FLY_OUT_MS);
    return () => clearTimeout(timer);
  }, [flyOut]);
  // Design §Reliability: the drawer blocks the picture's gestures; one under way ends with nothing done.
  useEffect(() => {
    if (!enabled) reset();
  }, [enabled, reset]);

  const down = (event: Pointer) => {
    // A second finger ends the gesture; nothing starts until the first one has left.
    if (touch.current) return reset();
    if (!enabled) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    const rect = event.currentTarget.getBoundingClientRect();
    const now: Touch = {
      pointerId: event.pointerId, rect, origin: pointerLocal(event, rect, rotated), startOverlay: overlay, phase: 'pressing', axis: 'x', sliding: false, timer: null,
    };
    // Extras and condition images have no template: nothing to hold for.
    if (hasTemplate) {
      now.timer = setTimeout(() => {
        if (touch.current !== now || now.phase !== 'pressing') return;
        now.phase = 'holding';
        buzz(HAPTIC.hold);
        setHolding(true);
      }, SWIPE.holdMs);
    }
    touch.current = now;
  };

  const move = (event: Pointer) => {
    const now = touch.current;
    if (!now || now.pointerId !== event.pointerId) return;
    const at = pointerLocal(event, now.rect, rotated);
    const dx = at.x - now.origin.x;
    const dy = at.y - now.origin.y;
    if (now.phase === 'pressing' && Math.hypot(dx, dy) > SWIPE.startPx) {
      if (now.timer) clearTimeout(now.timer);
      now.phase = 'swiping';
      now.axis = Math.abs(dx) > Math.abs(dy) ? 'x' : dy < 0 ? 'up' : 'down';
    }
    if (now.phase === 'swiping' && now.axis !== 'down') setSwipe(now.axis === 'x' ? { axis: 'x', d: dx } : { axis: 'up', d: Math.max(0, -dy) });
    if (now.phase !== 'holding') return;
    if (!now.sliding && Math.abs(dx) > SLIDE_PX && Math.abs(dx) > Math.abs(dy)) now.sliding = true;
    if (!now.sliding) return;
    const value = overlayValue(now.startOverlay, dx, at.w);
    if (Math.floor(value / 10) !== Math.floor(overlay / 10)) buzz(HAPTIC.overlayTick);
    dispatch({ type: 'overlay', overlay: value });
    setAdjust({ x: at.x, y: at.y, w: at.w });
  };

  const up = (event: Pointer) => {
    const now = touch.current;
    if (!now || now.pointerId !== event.pointerId) return;
    const at = pointerLocal(event, now.rect, rotated);
    const released: Swipe | null = now.phase !== 'swiping' || now.axis === 'down' ? null
      : now.axis === 'x' ? { axis: 'x', d: at.x - now.origin.x } : { axis: 'up', d: Math.max(0, now.origin.y - at.y) };
    reset();
    if (!released || Math.abs(released.d) < SWIPE.commitPx) return;
    const action = swipeAction(subject, directionOf(released));
    if (action === 'none') return;
    // The confirmation leaves the photo where it is; every other action shows the next one at once.
    const src = action === 'main_confirm' ? null : previewUrl(token, subject.image);
    if (src) setFlyOut({ n: (flyOut?.n ?? 0) + 1, src, from: shift(released, Math.abs(released.d)), to: shift(released, Math.abs(released.d) + 420) });
    onSwipe(action);
  };

  const direction = swipe && directionOf(swipe);
  const action = direction && swipeAction(subject, direction);
  const distance = swipe ? Math.abs(swipe.d) : 0;
  const armed = action !== null && action !== 'none' && distance >= SWIPE.commitPx;
  const photoStyle = {
    opacity: holding ? overlay / 100 : 1,
    transform: swipe && action ? shift(swipe, action === 'none' ? distance * RESIST : distance) : undefined,
    transition: swipe ? 'opacity 0.14s ease-out' : undefined,
  };

  return <div className="ph-preview" data-testid="preview" aria-label={header.kicker}>
    <div className="ph-safe-top" />
    <ImageFrame ratio={ratio} testId="preview-viewport" onPointerDown={down} onPointerMove={move} onPointerUp={up} onPointerCancel={reset}>
      {hasTemplate && subject.shot.example && <img className="ph-cover ph-grey" alt="" draggable={false} src={exampleUrl(token, subject.shot.container_id)} />}
      <PhotoImage key={photoId} token={token} image={subject.image} className="ph-cover ph-preview-photo" testId="preview-photo" style={photoStyle}>Preparing preview…</PhotoImage>
      {flyOut && <img key={flyOut.n} className="ph-cover ph-swipe-out" alt="" draggable={false} src={flyOut.src}
        style={{ '--ph-swipe-from': flyOut.from, '--ph-swipe-to': flyOut.to } as CSSProperties} />}
      {holding && <div className="ph-top-row">
        <div className="ph-pill" data-testid="preview-pill">TEMPLATE</div>
      </div>}
      {adjust && <OverlayBar at={adjust} percent={overlay} label="PHOTO" />}
      {swipe && action && direction && distance >= SWIPE.hintPx && <div className="ph-swipe-hint-row">
        <div className="ph-mono ph-swipe-hint" data-testid="swipe-hint" data-tone={action === 'none' ? 'none' : direction === 'left' ? 'deny' : 'go'} data-armed={armed || undefined}
          style={{ opacity: Math.min(1, distance / SWIPE.commitPx) }}>{hintOf(action, direction)}</div>
      </div>}
      {hasTemplate && subject.shot.split && <div className="ph-split-line" style={{ left: '50%' }} />}
    </ImageFrame>
    {strip}
    <div className="ph-preview-bar">
      <button type="button" className="ph-round" aria-label="Photo details" onClick={onDetails}>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden><circle cx="5" cy="12" r="1.8" /><circle cx="12" cy="12" r="1.8" /><circle cx="19" cy="12" r="1.8" /></svg>
      </button>
      <button type="button" className="ph-round" aria-label="Close" onClick={onClose}>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" aria-hidden><path d="M6 6l12 12M18 6L6 18" /></svg>
      </button>
    </div>
  </div>;
}
