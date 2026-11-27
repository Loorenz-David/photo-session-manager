import { type CSSProperties, type Dispatch, type PointerEvent as ReactPointerEvent, type ReactNode, useCallback, useEffect, useRef, useState } from 'react';
import { exampleUrl, previewUrl } from '../api';
import { type SwipeAction, swipeAction, type SwipeDirection } from '../arrange';
import { isRotated, type LocalPoint } from '../geometry';
import { buzz, HAPTIC, isDoubleTap, overlayValue, SLIDE_PX, SWIPE, type Tap, ZOOM } from '../gestures';
import { type Box, clampPan, type Fingers, NO_FINGERS, NO_ZOOM, pinchView, settleZoom, track, type Zoom } from '../pinch';
import { type Action, composedOf, previewHeader, type PreviewSubject } from '../state';
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
 * sideways sets the overlay, and no swipe happens until release). PH11: on a zoomed picture the
 * finger is `panning` instead (no swipe, no hold); the last finger of a pinch that left the picture
 * at 1× is only `waiting` to lift (no swipe starts from it).
 */
type Touch = {
  pointerId: number;
  origin: LocalPoint;
  startOverlay: number;
  phase: 'pressing' | 'swiping' | 'holding' | 'panning' | 'waiting';
  /** Panning: the zoom when this finger took the picture, and whether it has moved it since. */
  base: Zoom;
  moved: boolean;
  /** The finger a pinch left behind: its release is never a tap. */
  afterPinch: boolean;
  /** Swiping: along the picture's width, upwards, or downwards (which does nothing). */
  axis: 'x' | 'up' | 'down';
  /** Holding: the sideways slide that sets the overlay has started. */
  sliding: boolean;
  timer: ReturnType<typeof setTimeout> | null;
};
/**
 * One gesture, from the first finger down until none is tracked (PH11). The viewport's rectangle is
 * measured once, here: every point of the gesture is in the picture's own coordinates of that
 * moment, whatever the zoom layer does meanwhile.
 */
type Gesture = { rect: DOMRect; box: Box; fingers: Fingers; pinchBase: Zoom | null };
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
const round = (value: number, digits: number) => Math.round(value * 10 ** digits) / 10 ** digits || 0;
/** The zoom layer's transform (about the viewport's centre). */
const zoomCss = (zoom: Zoom) => `translate(${round(zoom.x, 2)}px, ${round(zoom.y, 2)}px) scale(${round(zoom.scale, 4)})`;
const shift = (swipe: Swipe, by: number) => (swipe.axis === 'up' ? `translateY(${-by}px)` : `translateX(${Math.sign(swipe.d) * by}px) rotate(${(Math.sign(swipe.d) * by) / 24}deg)`);

/**
 * Design §Preview gestures, with book 10J LV-D20 and LV-D21 (4) (master_plan_live §5 PH10): a
 * captured image, full screen, and nothing else at first glance — the photo, its strip, ⋯ (bottom
 * left: the photo's details and actions) and × (bottom right). Swipe the photo left to deny, right
 * for Main, up for Condition, and the next photo shows. A template shot's photo lies over its
 * template: hold shows the template through it, a sideways slide sets how much; the "TEMPLATE"
 * pill and the overlay readout show only during that gesture. The root is named by what the photo
 * is ("SHOT 02 · CAPTURED").
 *
 * LV-D21 (5), PH11: two fingers zoom the picture about the point between them (1× to 6×; it may
 * stretch a little past both while they are down and settles on release); zoomed, one finger moves
 * it, never past its edge, and swipes and the hold are off; a double tap goes to 3× about the tap, or
 * back to 1×. A second finger during a swipe or a hold ends it with nothing sent and starts a
 * pinch. The zoom goes back to 1× when another photo shows.
 */
export function Preview({ token, subject, ratio, overlay, dispatch, enabled, onClose, onDetails, onSwipe, strip }: Props) {
  const header = previewHeader(subject, ratio);
  const hasTemplate = subject.kind === 'shot';
  const rotated = isRotated(ratio);
  const touch = useRef<Touch | null>(null);
  const gesture = useRef<Gesture | null>(null);
  const lastTap = useRef<Tap | null>(null);
  const [holding, setHolding] = useState(false);
  const [adjust, setAdjust] = useState<{ x: number; y: number; w: number } | null>(null);
  const [swipe, setSwipe] = useState<Swipe | null>(null);
  const [flyOut, setFlyOut] = useState<FlyOut | null>(null);

  // The zoom belongs to the photo it was made on: another photo is always at 1×.
  const photoId = subject.image.photo_id;
  const [zoomed, setZoomed] = useState({ photoId, zoom: NO_ZOOM, settling: false });
  const zoomRef = useRef(zoomed);
  const zoomNow = useCallback(() => (zoomRef.current.photoId === photoId ? zoomRef.current.zoom : NO_ZOOM), [photoId]);
  /** `settling`: the picture eases to where it rests (a release, a double tap); while fingers move it, it follows at once. */
  const showZoom = useCallback((zoom: Zoom, settling: boolean) => {
    zoomRef.current = { photoId, zoom, settling };
    setZoomed(zoomRef.current);
  }, [photoId]);
  const shown = zoomed.photoId === photoId ? zoomed : { zoom: NO_ZOOM, settling: false };
  useEffect(() => {
    if (!zoomed.settling) return undefined;
    const timer = setTimeout(() => setZoomed(now => (now.settling ? { ...now, settling: false } : now)), ZOOM.settleMs);
    return () => clearTimeout(timer);
  }, [zoomed]);

  /** The one-finger gesture ends without acting: the picture springs back, the template hides. */
  const endTouch = useCallback(() => {
    if (touch.current?.timer) clearTimeout(touch.current.timer);
    touch.current = null;
    setHolding(false);
    setAdjust(null);
    setSwipe(null);
  }, []);
  /** Every way a gesture ends without acting; a zoom under way settles where it is. */
  const reset = useCallback(() => {
    const ended = gesture.current;
    gesture.current = null;
    endTouch();
    if (ended) showZoom(settleZoom(zoomNow(), ended.box), true);
  }, [endTouch, showZoom, zoomNow]);

  // Design §Reliability: a touch never outlives the page's attention, and never carries over to another photo.
  useEffect(() => reset, [photoId, reset]);
  useEffect(() => {
    lastTap.current = null;
    showZoom(NO_ZOOM, false);
  }, [photoId, showZoom]);
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

  /** The second of two close taps (the first is remembered). */
  const doubleTapped = (at: LocalPoint): boolean => {
    const tap = { t: Date.now(), x: at.x, y: at.y };
    const double = lastTap.current !== null && isDoubleTap(lastTap.current, tap);
    lastTap.current = double ? null : tap;
    return double;
  };

  /** One finger takes the picture: it pans a zoomed one; at 1× it is PH9.1's touch. */
  const begin = (pointerId: number, at: LocalPoint) => {
    const zoom = zoomNow();
    const now: Touch = {
      pointerId, origin: at, startOverlay: overlay, phase: zoom.scale > 1 ? 'panning' : 'pressing', base: zoom, moved: false, afterPinch: false,
      axis: 'x', sliding: false, timer: null,
    };
    // Extras and condition images have no template: nothing to hold for. A zoomed picture is not held either.
    if (now.phase === 'pressing' && hasTemplate) {
      now.timer = setTimeout(() => {
        if (touch.current !== now || now.phase !== 'pressing') return;
        now.phase = 'holding';
        buzz(HAPTIC.hold);
        setHolding(true);
      }, SWIPE.holdMs);
    }
    touch.current = now;
  };

  const down = (event: Pointer) => {
    if (!enabled) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    let now = gesture.current;
    if (!now) {
      // The viewport's own rectangle, once per gesture (the zoom layer inside it is never measured).
      const rect = event.currentTarget.getBoundingClientRect();
      const size = pointerLocal(event, rect, rotated);
      now = { rect, box: { left: 0, top: 0, width: size.w, height: size.h }, fingers: NO_FINGERS, pinchBase: null };
      gesture.current = now;
    }
    const at = pointerLocal(event, now.rect, rotated);
    const before = now.fingers;
    now.fingers = track(before, { type: 'down', pointerId: event.pointerId, x: at.x, y: at.y });
    if (now.fingers.kind === 'one') {
      endTouch();
      return begin(event.pointerId, at);
    }
    if (before.kind !== 'one') return;
    // A second finger: the swipe, hold or pan under way ends with nothing sent, and a pinch starts (plan point P2).
    endTouch();
    now.pinchBase = zoomNow();
  };

  const move = (event: Pointer) => {
    const active = gesture.current;
    if (!active) return;
    const at = pointerLocal(event, active.rect, rotated);
    active.fingers = track(active.fingers, { type: 'move', pointerId: event.pointerId, x: at.x, y: at.y });
    if (active.fingers.kind === 'two') {
      if (active.pinchBase && active.fingers.ids.includes(event.pointerId)) showZoom(pinchView(active.pinchBase, active.fingers.start, active.fingers.at, active.box), false);
      return;
    }
    const now = touch.current;
    if (!now || now.pointerId !== event.pointerId || now.phase === 'waiting') return;
    const dx = at.x - now.origin.x;
    const dy = at.y - now.origin.y;
    if (now.phase === 'panning') {
      if (Math.hypot(dx, dy) > SWIPE.startPx) now.moved = true;
      return showZoom(clampPan({ scale: now.base.scale, x: now.base.x + dx, y: now.base.y + dy }, active.box), false);
    }
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
    const active = gesture.current;
    if (!active) return;
    const at = pointerLocal(event, active.rect, rotated);
    const before = active.fingers;
    active.fingers = track(before, { type: 'up', pointerId: event.pointerId, x: at.x, y: at.y });
    if (active.fingers === before) return;
    if (active.fingers.kind === 'one') {
      // One of two fingers left: the other moves a zoomed picture on from where it is, or only waits to lift.
      const zoom = zoomNow();
      const rest = active.fingers;
      active.pinchBase = null;
      touch.current = {
        pointerId: rest.id, origin: { ...rest.at, w: active.box.width, h: active.box.height }, startOverlay: overlay, phase: zoom.scale > 1 ? 'panning' : 'waiting',
        base: zoom, moved: false, afterPinch: true, axis: 'x', sliding: false, timer: null,
      };
      return;
    }
    // The last finger left.
    const now = touch.current;
    gesture.current = null;
    if (!now) return;
    if (now.phase === 'panning' || now.phase === 'waiting') {
      endTouch();
      // A double tap on a zoomed picture: back to 1×.
      if (now.phase === 'panning' && !now.afterPinch && !now.moved && doubleTapped(at)) return showZoom(NO_ZOOM, true);
      return showZoom(settleZoom(zoomNow(), active.box), true);
    }
    if (now.phase === 'pressing') {
      endTouch();
      if (!doubleTapped(at)) return;
      // A double tap at 1×: `ZOOM.doubleTap`× about the tapped point (a spread of that size about it).
      const spread = (by: number): [LocalPoint, LocalPoint] => [{ ...at, x: at.x - by }, { ...at, x: at.x + by }];
      return showZoom(clampPan(pinchView(NO_ZOOM, spread(1), spread(ZOOM.doubleTap), active.box), active.box), true);
    }
    const released: Swipe | null = now.phase !== 'swiping' || now.axis === 'down' ? null
      : now.axis === 'x' ? { axis: 'x', d: at.x - now.origin.x } : { axis: 'up', d: Math.max(0, now.origin.y - at.y) };
    endTouch();
    if (!released || Math.abs(released.d) < SWIPE.commitPx) return;
    const action = swipeAction(subject, directionOf(released));
    if (action === 'none') return;
    // The confirmation leaves the photo where it is; every other action shows the next one at once.
    const src = action === 'main_confirm' ? null : previewUrl(token, subject.image);
    if (src) setFlyOut({ n: (flyOut?.n ?? 0) + 1, src, from: shift(released, Math.abs(released.d)), to: shift(released, Math.abs(released.d) + 420) });
    onSwipe(action);
  };

  /** The phone took a finger away (a call, a system gesture): the gesture ends where it is. */
  const cancelled = (event: Pointer) => {
    const active = gesture.current;
    if (!active || track(active.fingers, { type: 'cancel', pointerId: event.pointerId, x: 0, y: 0 }) === active.fingers) return;
    reset();
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
    <ImageFrame ratio={ratio} testId="preview-viewport" onPointerDown={down} onPointerMove={move} onPointerUp={up} onPointerCancel={cancelled}>
      <div className="ph-zoom" data-testid="preview-zoom" data-settling={shown.settling || undefined} style={{ transform: zoomCss(shown.zoom) }}>
        {hasTemplate && subject.shot.example && <img className="ph-cover ph-grey" alt="" draggable={false} src={exampleUrl(token, subject.shot.container_id)} />}
        <PhotoImage key={photoId} token={token} image={subject.image} className="ph-cover ph-preview-photo" testId="preview-photo" style={photoStyle}>Preparing preview…</PhotoImage>
        {hasTemplate && subject.shot.split && !composedOf(subject.image) && <div className="ph-split-line" style={{ left: '50%' }} />}
      </div>
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
