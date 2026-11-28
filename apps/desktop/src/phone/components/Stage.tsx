import { type Dispatch, type PointerEvent as ReactPointerEvent, type ReactNode, type RefObject, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { overlayLines } from '@/features/compose/overlays';
import type { OverlayKind } from '@/features/compose/useComposePrefs';
import { isRotated, type LocalPoint, localPoint, type Point, type Size, tapToFrame, viewportBox, zoomScale } from '../geometry';
import { buzz, HAPTIC, isDoubleTap, lockProgress, overlayValue, type Tap, TAP_FOCUS_MS, touchMode, type TouchMode } from '../gestures';
import { type Action, cameraNotice, currentShot, exposureText, hintText, inFreeShot, liveShown, modeLabel, type PhoneState, stageRatio, wantsLive } from '../state';
import { LiveCanvas, type LiveCanvasHandle } from './LiveCanvas';
import { LockControl, type LockState } from './LockControl';
import { OverlayBar } from './OverlayBar';
import { TemplateLayer } from './TemplateLayer';

type Pointer = ReactPointerEvent<HTMLDivElement>;
type FrameProps = {
  /** The image's aspect ratio (w/h); above 1.2 the viewport turns 90° clockwise. */
  ratio: number;
  testId: string;
  /** The viewport's layers; a function is given the viewport's own size in px (before it is turned). */
  children: ReactNode | ((viewport: Size) => ReactNode);
  onPointerDown?: (event: Pointer) => void;
  onPointerMove?: (event: Pointer) => void;
  onPointerUp?: (event: Pointer) => void;
  onPointerCancel?: (event: Pointer) => void;
};

/**
 * Design §Image viewport sizing: the stage (all the room between the top and the dock) centres an
 * image viewport sized from the shot's ratio. Only this adapts; the dock and control bar never move.
 */
export function ImageFrame({ ratio, testId, children, ...handlers }: FrameProps) {
  const stage = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<Size>({ width: 0, height: 0 });
  useLayoutEffect(() => {
    const element = stage.current;
    if (!element) return;
    const measure = () => setSize(previous =>
      previous.width === element.clientWidth && previous.height === element.clientHeight ? previous : { width: element.clientWidth, height: element.clientHeight });
    measure();
    window.addEventListener('resize', measure);
    const observer = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(measure);
    observer?.observe(element);
    return () => {
      window.removeEventListener('resize', measure);
      observer?.disconnect();
    };
  }, []);
  const box = viewportBox(size, ratio);
  return <div className="ph-stage" ref={stage}>
    <div className="ph-viewport" data-testid={testId} style={{ width: box.width, height: box.height, transform: `rotate(${box.rotated ? 90 : 0}deg)` }}
      onContextMenu={event => event.preventDefault()} {...handlers}>
      {typeof children === 'function' ? children(box) : children}
    </div>
  </div>;
}

/** Where a pointer is in the image's own coordinates (a rotated viewport turns them). */
export function pointerLocal(event: Pointer, rect: DOMRect, rotated: boolean): LocalPoint {
  return localPoint({ x: event.clientX, y: event.clientY }, rect, rotated);
}

/** Design §2 layer 11: full white at once, fading over .45 s from 70 ms after the capture. */
function Flash({ count }: { count: number }) {
  const [on, setOn] = useState(false);
  useEffect(() => {
    if (count === 0) return;
    setOn(true);
    const timer = setTimeout(() => setOn(false), 70);
    return () => clearTimeout(timer);
  }, [count]);
  return <div className="ph-flash" data-testid="flash" style={{ opacity: on ? 1 : 0, transition: on ? 'none' : 'opacity .45s ease-out' }} />;
}

const LockIcon = () => <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.6" strokeLinecap="round">
  <rect x="5" y="11" width="14" height="10" rx="2" /><path d="M8 11V8a4 4 0 0 1 8 0v3" />
</svg>;

/**
 * LV-D21 (7): the composition guides over the live view (10D Overlay; the Mac's `overlayLines` on
 * the viewport's own box, each half of a split shot). Drawn by the UI only, over the live picture
 * and the template alike; it takes no touch (H3; a tap still focuses through it).
 */
function LiveGuides({ kind, split, viewport }: { kind: OverlayKind; split: boolean; viewport: Size }) {
  if (kind === 'off' || viewport.width <= 0 || viewport.height <= 0) return null;
  const lines = overlayLines(kind, { left: 0, top: 0, width: viewport.width, height: viewport.height }, split);
  return <svg className="ph-guides ph-live-guides" data-testid="live-guides" data-guides={kind} viewBox={`0 0 ${viewport.width} ${viewport.height}`}
    width={viewport.width} height={viewport.height} style={{ pointerEvents: 'none' }} aria-hidden>
    {lines.map((line, index) => <line key={index} {...line} />)}
  </svg>;
}

type Props = {
  token: string;
  state: PhoneState;
  dispatch: Dispatch<Action>;
  /** The composition guides chosen for the live view (the Item drawer's Guides row). */
  guides: OverlayKind;
  /** Gestures are taken: connected, and no preview or drawer is open above the stage. */
  enabled: boolean;
  canvas: RefObject<LiveCanvasHandle | null>;
  isCurrentGen: (gen: number) => boolean;
  /** A tap while locked: the marker (0…1 of the image) and the point of the full live frame. */
  onFocus: (mark: Point, framePoint: Point) => void;
  onClearFocus: () => void;
};

/**
 * Design §2 and §Main viewport gestures: the image viewport with its layers, and the one-finger
 * gestures on it (hold for live, slide up to lock, slide sideways for the overlay, tap to focus
 * and double-tap to unlock while locked). All in image-local coordinates.
 */
export function Stage({ token, state, dispatch, guides, enabled, canvas, isCurrentGen, onFocus, onClearFocus }: Props) {
  const shot = currentShot(state.view);
  const free = inFreeShot(state.view);
  const ratio = stageRatio(state);
  const rotated = isRotated(ratio);
  const showing = liveShown(state);
  const live = wantsLive(state);
  const scale = zoomScale(shot?.zoom ?? 0);
  const notice = cameraNotice(state);
  const exposure = exposureText(state.camera);

  const touch = useRef<{ rect: DOMRect; rotated: boolean; mode: TouchMode | null; startOverlay: number; origin: LocalPoint } | null>(null);
  const [lock, setLock] = useState<LockState | null>(null);
  const [adjust, setAdjust] = useState<{ x: number; y: number; w: number } | null>(null);
  const lastTap = useRef<Tap | null>(null);
  const timers = useRef<{ focus?: ReturnType<typeof setTimeout>; fade?: ReturnType<typeof setTimeout>; gone?: ReturnType<typeof setTimeout> }>({});
  useEffect(() => () => Object.values(timers.current).forEach(clearTimeout), []);

  const keepLock = () => {
    clearTimeout(timers.current.fade);
    clearTimeout(timers.current.gone);
  };
  /** The lock control stays `delay` ms, fades (.25 s), then leaves. */
  const fadeLock = (delay: number) => {
    keepLock();
    timers.current.fade = setTimeout(() => {
      setLock(now => now && { ...now, out: true });
      timers.current.gone = setTimeout(() => setLock(null), 260);
    }, delay);
  };

  const down = (event: Pointer) => {
    if (!enabled) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    const rect = event.currentTarget.getBoundingClientRect();
    touch.current = { rect, rotated, mode: null, startOverlay: state.overlay, origin: pointerLocal(event, rect, rotated) };
    if (!state.locked) {
      buzz(HAPTIC.hold);
      dispatch({ type: 'hold', holding: true });
    }
  };

  const move = (event: Pointer) => {
    const now = touch.current;
    if (!now) return;
    const at = pointerLocal(event, now.rect, now.rotated);
    const dx = at.x - now.origin.x;
    const dy = at.y - now.origin.y;
    if (!now.mode) {
      // Decided once per touch: the first threshold crossed.
      now.mode = touchMode(dx, dy, { locked: state.locked, free });
      if (now.mode === 'lock') {
        keepLock();
        setLock({ x: now.origin.x, y: now.origin.y, p: 0, closed: false, track: true, out: false, pulse: 0 });
      }
    }
    if (now.mode === 'overlay') {
      const value = overlayValue(now.startOverlay, dx, at.w);
      if (Math.floor(value / 10) !== Math.floor(state.overlay / 10)) buzz(HAPTIC.overlayTick);
      dispatch({ type: 'overlay', overlay: value });
      setAdjust({ x: at.x, y: at.y, w: at.w });
    }
    if (now.mode === 'lock') {
      const p = lockProgress(dy);
      if (p >= 1 && !state.locked) {
        buzz(HAPTIC.lock);
        dispatch({ type: 'lock', locked: true });
        setLock(control => control && { ...control, p, closed: true, pulse: control.pulse + 1 });
      } else setLock(control => control && { ...control, p });
    }
  };

  const up = (event: Pointer) => {
    const now = touch.current;
    touch.current = null;
    if (!now) return;
    dispatch({ type: 'hold', holding: false });
    setAdjust(null);
    if (now.mode === 'lock') return fadeLock(state.locked ? 450 : 0);
    if (now.mode || !state.locked) return;
    // A tap while locked: the second of two close taps unlocks, a single one focuses.
    const at = pointerLocal(event, now.rect, now.rotated);
    const tap = { t: Date.now(), x: at.x, y: at.y };
    clearTimeout(timers.current.focus);
    if (lastTap.current && isDoubleTap(lastTap.current, tap)) {
      lastTap.current = null;
      buzz(HAPTIC.unlock);
      dispatch({ type: 'lock', locked: false });
      keepLock();
      setLock({ x: at.x, y: at.y + 30, p: 0, closed: false, track: false, out: false, pulse: 1 });
      return fadeLock(450);
    }
    lastTap.current = tap;
    const frame = state.frame ?? { width: 3, height: 2 };
    const zoom = shot?.zoom ?? 0;
    timers.current.focus = setTimeout(
      () => onFocus({ x: at.x / at.w, y: at.y / at.h }, tapToFrame(at, { width: at.w, height: at.h }, frame, zoom)),
      TAP_FOCUS_MS,
    );
  };

  const cancel = () => {
    touch.current = null;
    dispatch({ type: 'hold', holding: false });
    setAdjust(null);
    fadeLock(0);
  };

  const dark = live;
  const focus = state.focus;
  return <ImageFrame ratio={ratio} testId="viewport" onPointerDown={down} onPointerMove={move} onPointerUp={up} onPointerCancel={cancel}>{viewport => <>
    <div className="ph-layer">
      <LiveCanvas ref={canvas} isCurrent={isCurrentGen} shown={showing} scale={scale}
        onDrawn={(gen, width, height) => dispatch({ type: 'drawn', gen, width, height })} />
      {showing && exposure && <span className="ph-image-label ph-image-label-right">{exposure}</span>}
    </div>
    <TemplateLayer token={token} shot={shot} opacity={free ? 0 : showing ? state.overlay / 100 : 1} scale={scale} />
    {shot?.split && <>
      <div className="ph-split-line" data-testid="split-line" style={{ left: '50%' }} />
      <span className="ph-split-pill" style={{ left: '25%' }}>Left · 1:1</span>
      <span className="ph-split-pill" style={{ left: '75%' }}>Right · 1:1</span>
    </>}
    <LiveGuides kind={guides} split={shot?.split ?? false} viewport={viewport} />
    {focus && live && <div className="ph-reticle" data-testid="reticle" style={{ left: `${focus.x * 100}%`, top: `${focus.y * 100}%` }}><div className="ph-reticle-dot" /></div>}
    <div className="ph-top-row">
      <div className={`ph-pill${dark ? ' ph-pill-dark' : ''}`} data-testid="mode-pill">
        {state.locked && <LockIcon />}
        {modeLabel(state)}
      </div>
      {focus?.confirmed && <div className="ph-pill ph-af-chip" data-testid="af-chip">
        AF LOCKED
        <button type="button" className="ph-af-clear" aria-label="Clear focus" onPointerDown={event => {
          event.stopPropagation();
          onClearFocus();
        }}>×</button>
      </div>}
    </div>
    {adjust && <OverlayBar at={adjust} percent={state.overlay} label="TEMPLATE" />}
    {lock && <LockControl lock={lock} />}
    <div className="ph-hint-row">
      <div className="ph-hint" data-testid="hint" style={{ opacity: notice ? 1 : state.holding || adjust || lock || !enabled ? 0 : 1 }}>{notice ?? hintText(state)}</div>
    </div>
    <Flash count={state.flash} />
  </>}</ImageFrame>;
}
