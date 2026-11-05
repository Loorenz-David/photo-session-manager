/**
 * The Compose editor's interaction state machine (plan P31 §The interaction contract, extended by P32). Pure: no DOM,
 * no React.
 *
 * State: `{ phase, values (last committed), entry (when the editor came to this photo), live (what is on screen),
 * photoId, frame, view, needsConfirm }` with `phase ∈ idle | dragging { pointerId, startRect, origin } | pinching
 * { startRect, anchor, viaGestureEvents } | wheeling { mode: pan | zoom, startRect, lastAt } | trimming { handle,
 * pointerId, startRect, origin } | adjusting { startValues }` (P32: a handle trim; a live control such as the straighten
 * slider between its first `input` and its `change`).
 *
 * Effects: `View { rect }` (the binding writes the transform once per animation frame), `Save { photoId, values }`,
 * `Exit`, `Switch { photoId }`, `PreventDefault`.
 *
 * Rules (each has a row in `machine.test.ts`):
 *  1. A drag starts only on `PointerDown` with `isPrimary && button === 0` while `idle` (or while `wheeling`: rule 10
 *     first). Pointer events of any other `pointerId` while `dragging` have no effect.
 *  2. `PointerMove` of the dragging pointer → `live = clampRect(startRect panned by −(p − origin) / (W_t·s, H_t·s))`
 *     → `View`. The photo follows the pointer (`x −= dx / (W_t·s)`, `y −= dy / (H_t·s)`).
 *  3. `PointerUp` of the dragging pointer → `idle`; if `live ≠ values` then `values = live` and `Save`.
 *  4. `PointerCancel`, `LostCapture` (of the dragging pointer), `Blur`, `Hidden` while a gesture is active → exactly
 *     as rule 3 (the visible state is committed; an interrupted gesture never loses what the photographer saw).
 *  5. `KeyEsc` while `dragging`/`pinching`/`wheeling`/`trimming` → `live = startRect`, `View`, `idle`, no `Save`;
 *     while `adjusting` → `values = startValues`, `View`, `idle`, no `Save`.
 *  6. `KeyEsc` while `idle` → `live = values = entry`, `View`, and `Save` only if it differs from the last saved values.
 *  7. `Wheel` while `idle`: `ctrlKey || metaKey` → `wheeling { zoom }`, `live = clampRect(zoomAbout(live, point,
 *     exp(−dy · ZOOM_PER_PX)))`; otherwise → `wheeling { pan }`, the photo follows the fingers (natural scrolling:
 *     `x += dx / (W_t·s)`, `y += dy / (H_t·s)`). Every `Wheel` yields `PreventDefault`. Subsequent `Wheel`s of the
 *     same mode extend the gesture (`lastAt = at`); a `Wheel` of the other mode first commits (rule 9) and starts
 *     the new gesture.
 *  8. `Wheel` or `GestureChange` while `dragging` → `PreventDefault` only (one gesture at a time).
 *  9. `Tick { at }` while `wheeling` and `at − lastAt ≥ WHEEL_SETTLE_MS` → commit as rule 3 → `idle`.
 * 10. `PointerDown` while `wheeling` → commit the wheel gesture (rule 3) then start the drag (rule 1).
 * 11. `GestureStart` while `idle` or `wheeling` → `pinching { startRect, anchor: point, viaGestureEvents: true }` (a
 *     wheel gesture in flight is committed first); `GestureChange { scale }` → `live = clampRect(zoomAbout(startRect,
 *     anchor, scale))`, `View`; `GestureEnd` → commit (rule 3). While `pinching` with `viaGestureEvents`, `Wheel` →
 *     `PreventDefault` only (WebKit sends both; the gesture scale wins).
 * 12. `SwitchPhoto` → if a gesture is active commit it first (one `Save` for the old photo), then `values = entry =
 *     live = the new photo's composition values`, `phase idle`, `Switch`.
 * 13. `KeyReturn` → commit an active gesture (rule 3), then `Exit`. P32: when the photo still `needsConfirm` (it came
 *     to the editor in `needs_review`) and nothing was saved, one `Save` of the current values confirms it first.
 * 14. `Nudge`/`SetValues` while a pointer or pinch gesture is active → ignored; while `idle` or `wheeling` → commit the
 *     wheel first, then apply. P32: `Nudge { dx, dy }` pans by `(dx, dy)` screen pixels like a drag (`x −= dx / (W_t·s)`),
 *     clamped (rule 15), one `Save`. `SetValues { values, cause }`: `cause 'live'` → `adjusting { startValues }`,
 *     `values = live = new`, `View`, no `Save` (a slider between `input` and `change`); `cause 'set'` (default) →
 *     `idle`, apply, `View`, `Save` when the values differ from the gesture's start; `cause 'confirm'` → the same with
 *     the `Save` forced.
 * 15. Clamps (`clampRect`, `zoomAbout`): the rect keeps its ratio inside the frame and never shrinks below
 *     `MIN_RECT_FRACTION`; with `straighten ≠ 0` a move, zoom or trim whose result fails `insideAfterStraighten` is
 *     refused (the previous `live` stays).
 * 16. `Saved { photoId, composition }` of another photo → no change; of this photo while `idle` with values ≠ `values`
 *     (a refused save returned the stored composition) → `values = live = stored`, `View`.
 * P32 trims: `TrimStart { handle, pointerId, point }` while `idle` (or `wheeling`/`adjusting`: commit first) →
 *     `trimming { handle, pointerId, startRect: live, origin: point }`; `TrimMove { pointerId, point }` (or `PointerMove`
 *     of that pointer) → `live = trim(startRect, handle, p − origin)` under rule 15 — the view does **not** change (the
 *     overlay draws the trimmed rect inside the fixed frame box); `TrimEnd` / `PointerUp` / rule-4 interruptions →
 *     `idle`, `values.rect = live`, the view re-fits to the new rect (`View`) and one `Save` when it changed.
 * Entry (P32 Re-frame): a photo in `needs_review` enters with `entry = values = refit(stored, reframe, frame)` and
 *     `needsConfirm = true`; any `Save` clears it.
 * Extension (recorded in the P31 Review log): `Resize { stage }` recomputes the view for a new stage box → `View`.
 */
import { type Composition, type CompositionValues, type Frame, type Rect, valuesOf } from '@/services/compositions';
import { anchorIn, clampRect, insideAfterStraighten, panBy, shapeRatio, viewTransform, zoomAbout, type Box, type Point, type ViewTransform } from './geometry';
import { refit, trim, type Handle } from './refit';

/** Wheel zoom factor per pixel of `deltaY`: `exp(−deltaY · ZOOM_PER_PX)`. */
export const ZOOM_PER_PX = 0.005;
/** A wheel gesture ends after this quiet time. */
export const WHEEL_SETTLE_MS = 120;
const EPSILON = 1e-9;

export type Phase =
  | { kind: 'idle' }
  | { kind: 'dragging'; pointerId: number; startRect: Rect; origin: Point }
  | { kind: 'pinching'; startRect: Rect; anchor: Point; viaGestureEvents: boolean }
  | { kind: 'wheeling'; mode: 'pan' | 'zoom'; startRect: Rect; lastAt: number }
  | { kind: 'trimming'; handle: Handle; pointerId: number; startRect: Rect; origin: Point }
  | { kind: 'adjusting'; startValues: CompositionValues };

export type ViewGeometry = ViewTransform & { stage: Box };

export type ComposeState = {
  phase: Phase;
  values: CompositionValues;
  entry: CompositionValues;
  live: Rect;
  photoId: number;
  frame: Frame;
  view: ViewGeometry;
  /** The photo came to the editor in `needs_review` and nothing has been saved since (Return must save to confirm). */
  needsConfirm: boolean;
};

export type SetValuesCause = 'set' | 'live' | 'confirm';

export type ComposeEvent =
  | { type: 'PointerDown'; pointerId: number; isPrimary: boolean; button: number; point: Point }
  | { type: 'PointerMove'; pointerId: number; point: Point }
  | { type: 'PointerUp'; pointerId: number }
  | { type: 'PointerCancel'; pointerId: number }
  | { type: 'LostCapture'; pointerId: number }
  | { type: 'Wheel'; dx: number; dy: number; ctrlKey: boolean; metaKey: boolean; point: Point; at: number }
  | { type: 'GestureStart'; point: Point }
  | { type: 'GestureChange'; scale: number; point?: Point }
  | { type: 'GestureEnd' }
  | { type: 'Tick'; at: number }
  | { type: 'KeyEsc' }
  | { type: 'KeyReturn' }
  | { type: 'SwitchPhoto'; photoId: number; composition: Composition }
  | { type: 'Blur' }
  | { type: 'Hidden' }
  | { type: 'Nudge'; dx: number; dy: number }
  | { type: 'SetValues'; values: CompositionValues; cause?: SetValuesCause }
  | { type: 'TrimStart'; handle: Handle; pointerId: number; point: Point }
  | { type: 'TrimMove'; pointerId: number; point: Point }
  | { type: 'TrimEnd'; pointerId: number }
  | { type: 'Saved'; photoId: number; composition: Composition }
  | { type: 'Resize'; stage: Box };

export type Effect =
  | { type: 'View'; rect: Rect }
  | { type: 'Save'; photoId: number; values: CompositionValues }
  | { type: 'Exit' }
  | { type: 'Switch'; photoId: number }
  | { type: 'PreventDefault' };

export type Step = { state: ComposeState; effects: Effect[] };

const IDLE: Phase = { kind: 'idle' };
const PREVENT: Effect = { type: 'PreventDefault' };

export function sameRect(a: Rect, b: Rect): boolean {
  return Math.abs(a.x - b.x) <= EPSILON && Math.abs(a.y - b.y) <= EPSILON && Math.abs(a.w - b.w) <= EPSILON && Math.abs(a.h - b.h) <= EPSILON;
}

export function sameValues(a: CompositionValues, b: CompositionValues): boolean {
  return (
    a.kind === b.kind &&
    a.shape === b.shape &&
    a.orientation === b.orientation &&
    a.quarter_turns === b.quarter_turns &&
    a.flip === b.flip &&
    Math.abs((a.straighten ?? 0) - (b.straighten ?? 0)) <= EPSILON &&
    sameRect(a.rect, b.rect)
  );
}

function viewFor(values: CompositionValues, rect: Rect, frame: Frame, stage: Box): ViewGeometry {
  return { ...viewTransform({ ...values, rect }, frame, stage), stage };
}

/** The values the editor opens with: the stored ones, re-fitted to the current frame when the photo needs review (MC-27 §Revisions). */
function entryValues(composition: Composition): { values: CompositionValues; frame: Frame; needsConfirm: boolean } | null {
  const stored = valuesOf(composition);
  const frame = composition.frame;
  if (!stored || !frame) return null;
  const needsConfirm = composition.state === 'needs_review';
  return { values: needsConfirm ? refit(stored, { kind: 'reframe' }, frame) : stored, frame, needsConfirm };
}

/** The machine's state when the editor comes to `composition`; null when the backend has not materialised a rect. */
export function initialState(photoId: number, composition: Composition, stage: Box): ComposeState | null {
  const entry = entryValues(composition);
  if (!entry) return null;
  const { values, frame, needsConfirm } = entry;
  return { phase: IDLE, values, entry: values, live: values.rect, photoId, frame, view: viewFor(values, values.rect, frame, stage), needsConfirm };
}

const noop = (state: ComposeState): Step => ({ state, effects: [] });

function withLive(state: ComposeState, rect: Rect): ComposeState {
  return { ...state, live: rect, view: viewFor(state.values, rect, state.frame, state.view.stage) };
}

/** Rule 15: clamp, then refuse a result that leaves the straightened image. */
function settle(state: ComposeState, candidate: Rect): Rect {
  const clamped = clampRect(candidate);
  const straighten = state.values.straighten ?? 0;
  if (straighten !== 0 && !insideAfterStraighten(clamped, state.view.turned, straighten)) return state.live;
  return clamped;
}

/** Rule 3: back to idle; the visible rect becomes the committed values and is saved when it changed. */
function commit(state: ComposeState): Step {
  const { phase } = state;
  const idle = { ...state, phase: IDLE };
  if (phase.kind === 'adjusting') {
    if (sameValues(state.values, phase.startValues)) return { state: idle, effects: [] };
    return { state: idle, effects: [{ type: 'Save', photoId: state.photoId, values: state.values }] };
  }
  if (sameRect(state.live, state.values.rect)) return { state: idle, effects: [] };
  const values = { ...state.values, rect: state.live };
  if (phase.kind === 'trimming') {
    // The view stayed on the start rect during the trim; it re-fits to the new rect now (the glide).
    const view = viewFor(values, state.live, state.frame, state.view.stage);
    return { state: { ...idle, values, view }, effects: [{ type: 'View', rect: state.live }, { type: 'Save', photoId: state.photoId, values }] };
  }
  return { state: { ...idle, values }, effects: [{ type: 'Save', photoId: state.photoId, values }] };
}

function commitIfActive(state: ComposeState): Step {
  return state.phase.kind === 'idle' ? noop(state) : commit(state);
}

function startDrag(state: ComposeState, pointerId: number, origin: Point): ComposeState {
  return { ...state, phase: { kind: 'dragging', pointerId, startRect: state.live, origin } };
}

function startRectOf(phase: Phase): Rect | null {
  return phase.kind === 'idle' || phase.kind === 'adjusting' ? null : phase.startRect;
}

function adopt(state: ComposeState, photoId: number, composition: Composition): ComposeState {
  const entry = entryValues(composition);
  if (!entry) return { ...state, phase: IDLE };
  const { values, frame, needsConfirm } = entry;
  return { phase: IDLE, values, entry: values, live: values.rect, photoId, frame, view: viewFor(values, values.rect, frame, state.view.stage), needsConfirm };
}

/** The trimmed rect for the pointer at `point` (rule 15 refuses one that leaves the straightened image). */
function trimmed(state: ComposeState, phase: Extract<Phase, { kind: 'trimming' }>, point: Point): Rect {
  const ratio = shapeRatio(state.values.kind, state.values.shape, state.values.orientation, state.view.turned);
  const delta = { x: point.x - phase.origin.x, y: point.y - phase.origin.y };
  const candidate = trim(phase.startRect, phase.handle, delta, { turned: state.view.turned, scale: state.view.scale, ratio });
  const straighten = state.values.straighten ?? 0;
  if (straighten !== 0 && !insideAfterStraighten(candidate, state.view.turned, straighten)) return state.live;
  return candidate;
}

export function reduce(state: ComposeState, event: ComposeEvent): Step {
  const step = reduceEvent(state, event);
  if (step.state.needsConfirm && step.effects.some(effect => effect.type === 'Save')) {
    return { state: { ...step.state, needsConfirm: false }, effects: step.effects };
  }
  return step;
}

function reduceEvent(state: ComposeState, event: ComposeEvent): Step {
  const { phase } = state;
  switch (event.type) {
    case 'PointerDown': {
      if (!(event.isPrimary && event.button === 0)) return noop(state);
      if (phase.kind === 'idle') return { state: startDrag(state, event.pointerId, event.point), effects: [] };
      if (phase.kind === 'wheeling' || phase.kind === 'adjusting') {
        const committed = commit(state);
        return { state: startDrag(committed.state, event.pointerId, event.point), effects: committed.effects };
      }
      return noop(state);
    }
    case 'PointerMove': {
      if (phase.kind === 'trimming' && phase.pointerId === event.pointerId) return { state: { ...state, live: trimmed(state, phase, event.point) }, effects: [] };
      if (phase.kind !== 'dragging' || phase.pointerId !== event.pointerId) return noop(state);
      const dx = event.point.x - phase.origin.x;
      const dy = event.point.y - phase.origin.y;
      const live = settle(state, panBy(phase.startRect, { x: -dx / state.view.width, y: -dy / state.view.height }));
      return { state: withLive(state, live), effects: [{ type: 'View', rect: live }] };
    }
    case 'PointerUp':
    case 'PointerCancel':
    case 'LostCapture': {
      if ((phase.kind === 'dragging' || phase.kind === 'trimming') && phase.pointerId === event.pointerId) return commit(state);
      if (phase.kind === 'pinching' && event.type !== 'PointerUp') return commit(state);
      return noop(state);
    }
    case 'Blur':
    case 'Hidden':
      return commitIfActive(state);
    case 'Wheel': {
      const effects: Effect[] = [PREVENT];
      if (phase.kind === 'dragging' || phase.kind === 'pinching' || phase.kind === 'trimming') return { state, effects };
      const mode = event.ctrlKey || event.metaKey ? 'zoom' : 'pan';
      let current = state;
      if ((phase.kind === 'wheeling' && phase.mode !== mode) || phase.kind === 'adjusting') {
        const committed = commit(state);
        current = committed.state;
        effects.push(...committed.effects);
      }
      const startRect = current.phase.kind === 'wheeling' ? current.phase.startRect : current.live;
      const candidate =
        mode === 'zoom'
          ? zoomAbout(current.live, anchorIn(current.view.frameBox, event.point), Math.exp(-event.dy * ZOOM_PER_PX), current.view.turned)
          : panBy(current.live, { x: event.dx / current.view.width, y: event.dy / current.view.height });
      const live = settle(current, candidate);
      effects.push({ type: 'View', rect: live });
      return { state: { ...withLive(current, live), phase: { kind: 'wheeling', mode, startRect, lastAt: event.at } }, effects };
    }
    case 'GestureStart': {
      if (phase.kind === 'dragging' || phase.kind === 'pinching' || phase.kind === 'trimming') return { state, effects: [PREVENT] };
      const committed = commitIfActive(state);
      const current = committed.state;
      return {
        state: { ...current, phase: { kind: 'pinching', startRect: current.live, anchor: anchorIn(current.view.frameBox, event.point), viaGestureEvents: true } },
        effects: [PREVENT, ...committed.effects],
      };
    }
    case 'GestureChange': {
      if (phase.kind !== 'pinching') return { state, effects: [PREVENT] };
      const live = settle(state, zoomAbout(phase.startRect, phase.anchor, event.scale, state.view.turned));
      return { state: withLive(state, live), effects: [PREVENT, { type: 'View', rect: live }] };
    }
    case 'GestureEnd':
      return phase.kind === 'pinching' ? commit(state) : noop(state);
    case 'Tick':
      if (phase.kind === 'wheeling' && event.at - phase.lastAt >= WHEEL_SETTLE_MS) return commit(state);
      return noop(state);
    case 'KeyEsc': {
      if (phase.kind === 'adjusting') {
        const start = phase.startValues;
        return { state: { ...state, phase: IDLE, values: start, live: start.rect, view: viewFor(start, start.rect, state.frame, state.view.stage) }, effects: [{ type: 'View', rect: start.rect }] };
      }
      const startRect = startRectOf(phase);
      if (startRect) return { state: { ...withLive(state, startRect), phase: IDLE }, effects: [{ type: 'View', rect: startRect }] };
      const effects: Effect[] = [{ type: 'View', rect: state.entry.rect }];
      if (!sameValues(state.entry, state.values)) effects.push({ type: 'Save', photoId: state.photoId, values: state.entry });
      return { state: { ...withLive({ ...state, values: state.entry }, state.entry.rect), phase: IDLE }, effects };
    }
    case 'KeyReturn': {
      const committed = commitIfActive(state);
      const effects: Effect[] = [...committed.effects];
      if (committed.state.needsConfirm && !effects.some(effect => effect.type === 'Save')) {
        effects.push({ type: 'Save', photoId: committed.state.photoId, values: committed.state.values });
      }
      return { state: committed.state, effects: [...effects, { type: 'Exit' }] };
    }
    case 'SwitchPhoto': {
      const committed = commitIfActive(state);
      const next = adopt(committed.state, event.photoId, event.composition);
      const effects: Effect[] = [...committed.effects, { type: 'Switch', photoId: event.photoId }];
      if (next.photoId === event.photoId) effects.push({ type: 'View', rect: next.live });
      return { state: next, effects };
    }
    case 'Nudge': {
      if (phase.kind === 'dragging' || phase.kind === 'pinching' || phase.kind === 'trimming') return noop(state);
      const committed = commitIfActive(state);
      const current = committed.state;
      const live = settle(current, panBy(current.live, { x: -event.dx / current.view.width, y: -event.dy / current.view.height }));
      const applied = commit(withLive(current, live));
      return { state: applied.state, effects: [...committed.effects, { type: 'View', rect: live }, ...applied.effects] };
    }
    case 'SetValues': {
      if (phase.kind === 'dragging' || phase.kind === 'pinching' || phase.kind === 'trimming') return noop(state);
      const cause = event.cause ?? 'set';
      let current = state;
      const effects: Effect[] = [];
      if (phase.kind === 'wheeling') {
        const committed = commit(state);
        current = committed.state;
        effects.push(...committed.effects);
      }
      const startValues = current.phase.kind === 'adjusting' ? current.phase.startValues : current.values;
      const view = viewFor(event.values, event.values.rect, current.frame, current.view.stage);
      effects.push({ type: 'View', rect: event.values.rect });
      if (cause === 'live') {
        return { state: { ...current, phase: { kind: 'adjusting', startValues }, values: event.values, live: event.values.rect, view }, effects };
      }
      if (cause === 'confirm' || !sameValues(startValues, event.values)) effects.push({ type: 'Save', photoId: current.photoId, values: event.values });
      return { state: { ...current, phase: IDLE, values: event.values, live: event.values.rect, view }, effects };
    }
    case 'TrimStart': {
      if (phase.kind === 'dragging' || phase.kind === 'pinching' || phase.kind === 'trimming') return noop(state);
      const committed = commitIfActive(state);
      const current = committed.state;
      return { state: { ...current, phase: { kind: 'trimming', handle: event.handle, pointerId: event.pointerId, startRect: current.live, origin: event.point } }, effects: committed.effects };
    }
    case 'TrimMove': {
      if (phase.kind !== 'trimming' || phase.pointerId !== event.pointerId) return noop(state);
      return { state: { ...state, live: trimmed(state, phase, event.point) }, effects: [] };
    }
    case 'TrimEnd': {
      if (phase.kind !== 'trimming' || phase.pointerId !== event.pointerId) return noop(state);
      return commit(state);
    }
    case 'Saved': {
      if (event.photoId !== state.photoId || phase.kind !== 'idle') return noop(state);
      const stored = valuesOf(event.composition);
      if (!stored || sameValues(stored, state.values)) return noop(state);
      return { state: withLive({ ...state, values: stored }, stored.rect), effects: [{ type: 'View', rect: stored.rect }] };
    }
    case 'Resize': {
      const view = viewFor(state.values, state.live, state.frame, event.stage);
      return { state: { ...state, view }, effects: [{ type: 'View', rect: state.live }] };
    }
  }
}
