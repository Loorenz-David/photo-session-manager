import { type PointerEvent as ReactPointerEvent, useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { composedLayout } from '@/features/compose/composed';
import { anchorIn, type Box, transformCss, viewTransform } from '@/features/compose/geometry';
import { overlayLines } from '@/features/compose/overlays';
import { refit, type RefitChange } from '@/features/compose/refit';
import { type OverlayKind, STRAIGHTEN_GRID_PX } from '@/features/compose/useComposePrefs';
import type { CompositionValues } from '@/services/compositions';
import { type PhoneComposition, type PhoneImage, previewUrl } from '../api';
import { buzz, HAPTIC } from '../gestures';
import { type Fingers, NO_FINGERS, track, twoFinger } from '../pinch';
import { panWithin, resetValues, rotateWithin, straightenFromTwist, TWIST_DEAD_ZONE, zoomWithin } from '../reframe';
import { composedOf, GUIDE_KINDS, GUIDE_LABELS, readGuides, REFRAME_GUIDES_KEY, writeGuides } from '../state';

type Pointer = ReactPointerEvent<HTMLDivElement>;

/** What Done sends: the framing as edited, or (after Reset with nothing changed since) the Mac's own reset. */
export type ReframeResult = { kind: 'save'; values: CompositionValues } | { kind: 'reset' };

type Props = {
  token: string;
  /** The photo being reframed; it has a composition (the caller closes the editor when it loses it). */
  image: PhoneImage & { composition: PhoneComposition };
  /** The Mac's `default_shape`: what Reset frames with. */
  defaultShape: string;
  /** Done's request is out: Done waits for its answer. */
  busy?: boolean;
  onCancel: () => void;
  /** Done with a framing that differs from the one the editor opened with; absent, Done is off. */
  onDone?: (result: ReframeResult) => void;
};

const SHAPES: readonly { value: string; label: string }[] = [
  { value: 'original', label: 'Original' }, { value: '1:1', label: '1:1' }, { value: '4:5', label: '4:5' }, { value: '3:2', label: '3:2' },
  { value: '2:3', label: '2:3' }, { value: '16:9', label: '16:9' }, { value: 'free', label: 'Free' },
];
const TURNS: readonly { label: string; change: RefitChange }[] = [
  { label: 'Rotate 90° left', change: { kind: 'turn', direction: 'left' } },
  { label: 'Rotate 90° right', change: { kind: 'turn', direction: 'right' } },
  { label: 'Flip horizontal', change: { kind: 'flip' } },
];

/** One gesture on the stage, from the first finger down until none is tracked; `base` = the values its fingers started from. */
type Gesture = { rect: DOMRect; fingers: Fingers; base: CompositionValues };

const same = (a: CompositionValues, b: CompositionValues) =>
  a.kind === b.kind && a.shape === b.shape && a.orientation === b.orientation && a.quarter_turns === b.quarter_turns && a.flip === b.flip
  && (a.straighten ?? 0) === (b.straighten ?? 0) && a.rect.x === b.rect.x && a.rect.y === b.rect.y && a.rect.w === b.rect.w && a.rect.h === b.rect.h;

/**
 * Book 10J LV-D21 (6), master_plan_live §5 PH12: reframe a photo on the phone, the Mac's composition
 * (10D MC-27) with phone gestures. The frame stays centred and the photo moves under it: one finger
 * moves it; two fingers move it, zoom it (spread = closer) and straighten it (twist, after 3°). Every
 * move is bounded while it happens (`reframe.ts`), so the frame never shows empty corners and
 * nothing springs back. A fine grid shows while straightening; the composition guides lie inside
 * the frame. The controls apply the Mac's own `refit`. Nothing reaches the Mac before Done.
 * The editor stays upright whatever the photo's shape (plan point P9); it has no trimming handles
 * (P8: a Free shape keeps its proportions here).
 */
export function ReframeEditor({ token, image, defaultShape, busy = false, onCancel, onDone }: Props) {
  const frame = image.composition.frame;
  // What the editor opened with: the stored framing, re-fitted to the frame when the Mac asks for a review (as the Mac's editor).
  const [opened] = useState<CompositionValues>(() => {
    const stored = composedOf(image)!.values;
    return image.composition.state === 'needs_review' ? refit(stored, { kind: 'reframe' }, frame) : stored;
  });
  const [values, setValues] = useState(opened);
  const live = useRef(values);
  /** Reset was the last change: Done asks the Mac for its own reset. */
  const [wasReset, setWasReset] = useState(false);
  const [menu, setMenu] = useState<'shape' | 'turn' | 'guides' | null>(null);
  const [guides, setGuides] = useState<OverlayKind>(() => readGuides(REFRAME_GUIDES_KEY, 'thirds'));
  const [twisting, setTwisting] = useState(false);
  const [sliding, setSliding] = useState(false);
  const slideBase = useRef<CompositionValues | null>(null);
  const gesture = useRef<Gesture | null>(null);

  const stage = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 0, height: 0 });
  useLayoutEffect(() => {
    const element = stage.current;
    if (!element) return undefined;
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
  const stageBox: Box | null = size.width > 0 && size.height > 0 ? { left: 0, top: 0, width: size.width, height: size.height } : null;

  /** The editor shows `next` (always a valid framing). */
  const show = (next: CompositionValues) => {
    if (same(next, live.current)) return;
    if (Math.trunc(next.straighten ?? 0) !== Math.trunc(live.current.straighten ?? 0)) buzz(HAPTIC.overlayTick);
    live.current = next;
    setValues(next);
    setWasReset(false);
  };
  /** A control: the Mac's re-fit of what the editor shows. */
  const apply = (change: RefitChange) => {
    buzz(HAPTIC.select);
    show(refit(live.current, change, frame));
  };

  /** Every way a gesture ends: the last values shown stay (they were valid at every move). */
  const endGesture = useCallback(() => {
    gesture.current = null;
    setTwisting(false);
  }, []);
  const endSlide = useCallback(() => {
    slideBase.current = null;
    setSliding(false);
  }, []);
  // Design §Reliability: no gesture outlives the page's attention.
  useEffect(() => {
    const end = () => {
      endGesture();
      endSlide();
    };
    const hidden = () => {
      if (document.visibilityState === 'hidden') end();
    };
    window.addEventListener('blur', end);
    document.addEventListener('visibilitychange', hidden);
    return () => {
      window.removeEventListener('blur', end);
      document.removeEventListener('visibilitychange', hidden);
    };
  }, [endGesture, endSlide]);

  const pointOf = (event: Pointer, rect: DOMRect) => ({ x: event.clientX - rect.left, y: event.clientY - rect.top });

  const down = (event: Pointer) => {
    if (busy) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    setMenu(null);
    // The stage's rectangle, once per gesture.
    const now = gesture.current ?? { rect: event.currentTarget.getBoundingClientRect(), fingers: NO_FINGERS, base: live.current };
    const before = now.fingers;
    now.fingers = track(before, { type: 'down', pointerId: event.pointerId, ...pointOf(event, now.rect) });
    // One finger starts, or a second joins it: both go on from what the editor shows now.
    if (now.fingers !== before) now.base = live.current;
    gesture.current = now;
  };

  const move = (event: Pointer) => {
    const now = gesture.current;
    if (!now || !stageBox) return;
    const before = now.fingers;
    now.fingers = track(before, { type: 'move', pointerId: event.pointerId, ...pointOf(event, now.rect) });
    if (now.fingers === before) return;
    const view = viewTransform(now.base, frame, stageBox);
    if (now.fingers.kind === 'one') {
      // The photo follows the finger: the rect moves the other way, in fractions of the turned frame.
      const { start, at } = now.fingers;
      return show(panWithin(now.base, frame, { x: -(at.x - start.x) / view.width, y: -(at.y - start.y) / view.height }));
    }
    if (now.fingers.kind !== 'two') return;
    // Two fingers, each step bounded, from the values they started on: straighten, then zoom about
    // the point that was between them, then move with it.
    const fingers = twoFinger(now.fingers.start, now.fingers.at);
    const turned = rotateWithin(now.base, frame, (now.base.straighten ?? 0) + straightenFromTwist(fingers.rotation));
    const zoomed = zoomWithin(turned, frame, anchorIn(view.frameBox, fingers.anchor), fingers.scale);
    const after = viewTransform(zoomed, frame, stageBox);
    setTwisting(Math.abs(fingers.rotation) > TWIST_DEAD_ZONE);
    show(panWithin(zoomed, frame, { x: -(fingers.mid.x - fingers.anchor.x) / after.width, y: -(fingers.mid.y - fingers.anchor.y) / after.height }));
  };

  const up = (event: Pointer) => {
    const now = gesture.current;
    if (!now) return;
    const before = now.fingers;
    now.fingers = track(before, { type: 'up', pointerId: event.pointerId, ...pointOf(event, now.rect) });
    if (now.fingers === before) return;
    if (now.fingers.kind === 'idle') return endGesture();
    // One of two fingers left: the other moves the photo on from here.
    now.base = live.current;
    setTwisting(false);
  };

  /** The phone took a finger away (a call, a system gesture): the gesture ends where it is. */
  const cancelled = (event: Pointer) => {
    const now = gesture.current;
    if (!now || track(now.fingers, { type: 'cancel', pointerId: event.pointerId, x: 0, y: 0 }) === now.fingers) return;
    endGesture();
  };

  const pickGuides = (kind: OverlayKind) => {
    buzz(HAPTIC.select);
    setGuides(kind);
    writeGuides(REFRAME_GUIDES_KEY, kind);
    setMenu(null);
  };
  const toggle = (which: 'shape' | 'turn' | 'guides') => {
    buzz(HAPTIC.menu);
    setMenu(now => (now === which ? null : which));
  };
  const reset = () => {
    buzz(HAPTIC.select);
    setMenu(null);
    show(resetValues(frame, defaultShape));
    setWasReset(true);
  };
  const done = () => {
    if (busy || !onDone) return;
    // Nothing changed: nothing is sent (plan point P7).
    if (same(values, opened)) return onCancel();
    onDone(wasReset ? { kind: 'reset' } : { kind: 'save', values });
  };

  const view = stageBox && viewTransform(values, frame, stageBox);
  const layout = composedLayout(values, frame);
  const split = values.kind === 'split';
  const degrees = values.straighten ?? 0;
  const src = previewUrl(token, image) ?? undefined;
  const menuItems = menu === 'shape' ? SHAPES.map(shape => ({ label: shape.label, on: values.shape === shape.value, pick: () => { setMenu(null); apply({ kind: 'shape', shape: shape.value }); } }))
    : menu === 'turn' ? TURNS.map(turn => ({ label: turn.label, on: false, pick: () => { setMenu(null); apply(turn.change); } }))
      : menu === 'guides' ? GUIDE_KINDS.map(kind => ({ label: GUIDE_LABELS[kind], on: guides === kind, pick: () => pickGuides(kind) }))
        : [];

  return <div className="ph-reframe" role="dialog" aria-label="Reframe" data-testid="reframe-editor">
    <div className="ph-safe-top" />
    <div className="ph-reframe-bar">
      <button type="button" className="ph-reframe-text" onClick={onCancel}>Cancel</button>
      <span className="ph-mono ph-kicker">REFRAME</span>
      <button type="button" className="ph-reframe-text ph-reframe-done" disabled={busy || !onDone} onClick={done}>Done</button>
    </div>
    <div className="ph-reframe-stage" ref={stage} data-testid="reframe-stage" onContextMenu={event => event.preventDefault()}
      onPointerDown={down} onPointerMove={move} onPointerUp={up} onPointerCancel={cancelled}>
      {view && <>
        <div className="ph-reframe-picture" data-testid="reframe-picture" style={{ width: view.width, height: view.height, transform: transformCss(view) }}>
          <img alt="" draggable={false} src={src} data-swapped={layout.swapped || undefined} style={{ transform: layout.imageTransform }} />
        </div>
        <div className="ph-reframe-frame" data-testid="reframe-frame" data-kind={values.kind}
          style={{ left: view.frameBox.left, top: view.frameBox.top, width: view.frameBox.width, height: view.frameBox.height }}>
          {split && <span className="ph-composed-split" data-testid="reframe-middle" aria-hidden />}
        </div>
        <svg className="ph-guides" data-testid="reframe-guides" data-guides={guides} viewBox={`0 0 ${size.width} ${size.height}`} width={size.width} height={size.height} aria-hidden>
          {overlayLines(guides, view.frameBox, split).map((line, index) => <line key={index} {...line} />)}
        </svg>
        {(twisting || sliding) && <div className="ph-reframe-grid" data-testid="reframe-grid" style={{ backgroundSize: `${STRAIGHTEN_GRID_PX}px ${STRAIGHTEN_GRID_PX}px` }} />}
      </>}
    </div>
    <div className="ph-reframe-tools">
      <label className="ph-reframe-straighten">
        <input type="range" aria-label="Straighten" min={-45} max={45} step={0.1} value={degrees} disabled={busy}
          onPointerDown={() => {
            slideBase.current = live.current;
            setSliding(true);
          }}
          onPointerUp={endSlide} onPointerCancel={endSlide} onBlur={endSlide}
          // While the control is held the angle is taken from the framing the hold started on, so going back gives the frame back.
          onChange={event => show(refit(slideBase.current ?? live.current, { kind: 'straighten', degrees: Number(event.target.value) }, frame))} />
        <output className="ph-mono" data-testid="reframe-degrees">{degrees.toFixed(1)}°</output>
      </label>
      {menu && <div className="ph-reframe-menu" role="menu" data-testid="reframe-menu">
        {menuItems.map(item => <button key={item.label} type="button" role="menuitem" className="ph-reframe-item" aria-current={item.on || undefined} onClick={item.pick}>{item.label}</button>)}
      </div>}
      <div className="ph-reframe-row">
        <div className="ph-reframe-kind" role="group" aria-label="Kind">
          <button type="button" aria-pressed={!split} onClick={() => apply({ kind: 'kind', to: 'normal' })}>Normal</button>
          <button type="button" aria-pressed={split} onClick={() => apply({ kind: 'kind', to: 'split' })}>Split</button>
        </div>
        <button type="button" className="ph-reframe-tool" aria-haspopup="menu" aria-expanded={menu === 'shape'} onClick={() => toggle('shape')}>Shape</button>
        <button type="button" className="ph-reframe-tool" onClick={() => apply({ kind: 'orientation', orientation: values.orientation === 'turned' ? 'named' : 'turned' })}>Turn</button>
      </div>
      <div className="ph-reframe-row">
        <button type="button" className="ph-reframe-tool" aria-label="Rotate and flip" aria-haspopup="menu" aria-expanded={menu === 'turn'} onClick={() => toggle('turn')}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden><path d="M21 12a9 9 0 1 1-3-6.7" /><path d="M21 4v5h-5" /></svg>
        </button>
        <button type="button" className="ph-reframe-tool" aria-haspopup="menu" aria-expanded={menu === 'guides'} onClick={() => toggle('guides')}>Guides</button>
        <button type="button" className="ph-reframe-tool" onClick={reset}>Reset</button>
      </div>
    </div>
  </div>;
}
