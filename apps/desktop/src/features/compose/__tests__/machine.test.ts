import { describe, expect, it } from 'vitest';
import type { Composition, CompositionValues, Rect } from '@/services/compositions';
import { initialState, reduce, WHEEL_SETTLE_MS, type ComposeEvent, type ComposeState, type Effect } from '../machine';

const STAGE = { left: 0, top: 0, width: 1000, height: 700 };
const FRAME = { w: 2560, h: 1707 };
const DEFAULT: Rect = { x: 0.16660156250000002, y: 0, w: 0.666796875, h: 1 };
const PANNED_X = 0.11546683; // the default square moved by 50 px: x − 50 / 977.81
const ZOOMED: Rect = { x: 0.297783626, y: 0.19673467, w: 0.404432748, h: 0.60653066 }; // exp(0.5) about the frame centre

function composition(photoId: number, over: Partial<Composition> = {}): Composition {
  return { photo_id: photoId, kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: DEFAULT, frame: FRAME, origin: 'default', revision: 0, state: 'ok', review_reason: null, updated_at: 0, ...over };
}
const idle = () => initialState(1, composition(1), STAGE)!;

const down = (x: number, y: number, extra: Partial<{ pointerId: number; isPrimary: boolean; button: number }> = {}): ComposeEvent => ({ type: 'PointerDown', pointerId: 1, isPrimary: true, button: 0, point: { x, y }, ...extra });
const move = (x: number, y: number, pointerId = 1): ComposeEvent => ({ type: 'PointerMove', pointerId, point: { x, y } });
const up = (pointerId = 1): ComposeEvent => ({ type: 'PointerUp', pointerId });
const wheel = (over: Partial<{ dx: number; dy: number; ctrlKey: boolean; metaKey: boolean; at: number }> = {}): ComposeEvent => ({ type: 'Wheel', dx: 0, dy: 0, ctrlKey: false, metaKey: false, point: { x: 500, y: 350 }, at: 0, ...over });
const ZOOM_IN = Math.exp(0.5);

/** Runs the events in order, collecting every effect. */
function run(start: ComposeState, events: ComposeEvent[]): { state: ComposeState; effects: Effect[] } {
  let state = start;
  const effects: Effect[] = [];
  for (const event of events) {
    const step = reduce(state, event);
    state = step.state;
    effects.push(...step.effects);
  }
  return { state, effects };
}
const saves = (effects: Effect[]) => effects.filter((e): e is Extract<Effect, { type: 'Save' }> => e.type === 'Save');
const types = (effects: Effect[]) => effects.map(e => e.type);
const near = (actual: number, expected: number, tolerance = 1e-6) => expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance);
const rectNear = (actual: Rect, expected: Rect, tolerance = 1e-6) => {
  near(actual.x, expected.x, tolerance);
  near(actual.y, expected.y, tolerance);
  near(actual.w, expected.w, tolerance);
  near(actual.h, expected.h, tolerance);
};
const DRAG_50: ComposeEvent[] = [down(500, 350), move(550, 350)];

describe('C1(c) the interaction contract, one row per rule', () => {
  it('rule 1: a drag starts only on a primary button-0 PointerDown while idle; other pointers are ignored while dragging', () => {
    expect(reduce(idle(), down(500, 350, { isPrimary: false })).state.phase.kind).toBe('idle');
    expect(reduce(idle(), down(500, 350, { button: 2 })).state.phase.kind).toBe('idle');
    const dragging = reduce(idle(), down(500, 350));
    expect(dragging.state.phase).toEqual({ kind: 'dragging', pointerId: 1, startRect: DEFAULT, origin: { x: 500, y: 350 } });
    expect(dragging.effects).toEqual([]);
    const other = run(dragging.state, [down(600, 350, { pointerId: 2, isPrimary: false }), move(700, 350, 2), up(2)]);
    expect(other.effects).toEqual([]);
    expect(other.state.phase).toEqual(dragging.state.phase);
    expect(other.state.live).toEqual(DEFAULT);
  });

  it('rule 2: PointerMove of the dragging pointer pans the photo with the pointer and yields View', () => {
    const moved = run(idle(), DRAG_50);
    near(moved.state.live.x, PANNED_X);
    expect(moved.state.live.y).toBe(0);
    expect(moved.state.live.w).toBe(DEFAULT.w);
    expect(moved.effects).toEqual([{ type: 'View', rect: moved.state.live }]);
    expect(moved.state.values.rect).toEqual(DEFAULT);
    const vertical = run(idle(), [down(500, 350), move(500, 450)]);
    expect(vertical.state.live.y).toBe(0);
    near(vertical.state.live.x, DEFAULT.x);
  });

  it('rule 3: PointerUp of the dragging pointer commits: idle, values = live, one Save; nothing moved → no Save', () => {
    const released = run(idle(), [...DRAG_50, up()]);
    expect(released.state.phase.kind).toBe('idle');
    near(released.state.values.rect.x, PANNED_X);
    expect(saves(released.effects)).toHaveLength(1);
    expect(saves(released.effects)[0].photoId).toBe(1);
    expect(saves(released.effects)[0].values).toEqual(released.state.values);
    const still = run(idle(), [down(500, 350), up()]);
    expect(still.state.phase.kind).toBe('idle');
    expect(still.effects).toEqual([]);
  });

  it('rule 4: PointerCancel, LostCapture, Blur and Hidden mid-gesture commit the visible state (one Save)', () => {
    const interruptions: ComposeEvent[] = [{ type: 'PointerCancel', pointerId: 1 }, { type: 'LostCapture', pointerId: 1 }, { type: 'Blur' }, { type: 'Hidden' }];
    for (const interruption of interruptions) {
      const result = run(idle(), [...DRAG_50, interruption]);
      expect(result.state.phase.kind, interruption.type).toBe('idle');
      expect(saves(result.effects), interruption.type).toHaveLength(1);
      near(saves(result.effects)[0].values.rect.x, PANNED_X);
    }
    const pinchBlur = run(idle(), [{ type: 'GestureStart', point: { x: 500, y: 350 } }, { type: 'GestureChange', scale: ZOOM_IN }, { type: 'Blur' }]);
    expect(pinchBlur.state.phase.kind).toBe('idle');
    expect(saves(pinchBlur.effects)).toHaveLength(1);
    rectNear(saves(pinchBlur.effects)[0].values.rect, ZOOMED);
  });

  it('rule 5: KeyEsc mid-gesture restores the gesture start, idle, no Save', () => {
    const escaped = run(idle(), [...DRAG_50, { type: 'KeyEsc' }, up()]);
    expect(escaped.state.phase.kind).toBe('idle');
    expect(escaped.state.live).toEqual(DEFAULT);
    expect(escaped.state.values.rect).toEqual(DEFAULT);
    expect(types(escaped.effects)).toEqual(['View', 'View']);
    expect(escaped.effects[1]).toEqual({ type: 'View', rect: DEFAULT });
    const wheelEsc = run(idle(), [wheel({ dx: -50 }), { type: 'KeyEsc' }]);
    expect(wheelEsc.state.phase.kind).toBe('idle');
    expect(wheelEsc.state.live).toEqual(DEFAULT);
    expect(saves(wheelEsc.effects)).toHaveLength(0);
    const pinchEsc = run(idle(), [{ type: 'GestureStart', point: { x: 500, y: 350 } }, { type: 'GestureChange', scale: ZOOM_IN }, { type: 'KeyEsc' }]);
    expect(pinchEsc.state.live).toEqual(DEFAULT);
    expect(saves(pinchEsc.effects)).toHaveLength(0);
  });

  it('rule 6: KeyEsc while idle returns to the entry values and saves only when they differ from the last saved', () => {
    const dragged = run(idle(), [...DRAG_50, up()]);
    const reverted = reduce(dragged.state, { type: 'KeyEsc' });
    expect(reverted.state.values).toEqual(reverted.state.entry);
    expect(reverted.state.live).toEqual(DEFAULT);
    expect(reverted.effects).toEqual([{ type: 'View', rect: DEFAULT }, { type: 'Save', photoId: 1, values: reverted.state.entry }]);
    const again = reduce(reverted.state, { type: 'KeyEsc' });
    expect(again.effects).toEqual([{ type: 'View', rect: DEFAULT }]);
  });

  it('rule 7: a wheel while idle starts a zoom (ctrl or ⌘) or a pan gesture, always PreventDefault; the other mode commits first', () => {
    const ctrl = reduce(idle(), wheel({ dy: -100, ctrlKey: true, at: 10 }));
    expect(ctrl.state.phase).toEqual({ kind: 'wheeling', mode: 'zoom', startRect: DEFAULT, lastAt: 10 });
    rectNear(ctrl.state.live, ZOOMED);
    expect(types(ctrl.effects)).toEqual(['PreventDefault', 'View']);
    const meta = reduce(idle(), wheel({ dy: -100, metaKey: true, at: 10 }));
    rectNear(meta.state.live, ZOOMED);
    const pan = reduce(idle(), wheel({ dx: -50, at: 10 }));
    expect(pan.state.phase).toEqual({ kind: 'wheeling', mode: 'pan', startRect: DEFAULT, lastAt: 10 });
    near(pan.state.live.x, PANNED_X);
    expect(types(pan.effects)).toEqual(['PreventDefault', 'View']);
    const extended = reduce(pan.state, wheel({ dx: -50, at: 40 }));
    expect(extended.state.phase).toEqual({ kind: 'wheeling', mode: 'pan', startRect: DEFAULT, lastAt: 40 });
    near(extended.state.live.x, DEFAULT.x - 2 * (DEFAULT.x - PANNED_X));
    expect(saves(extended.effects)).toHaveLength(0);
    const switched = reduce(pan.state, wheel({ dy: -100, ctrlKey: true, at: 50 }));
    expect(switched.state.phase.kind === 'wheeling' && switched.state.phase.mode).toBe('zoom');
    expect(saves(switched.effects)).toHaveLength(1);
    near(saves(switched.effects)[0].values.rect.x, PANNED_X);
  });

  it('rule 8: a wheel or a gesture change during a drag only prevents the default', () => {
    const dragging = run(idle(), DRAG_50);
    const wheeled = reduce(dragging.state, wheel({ dy: -100, ctrlKey: true }));
    expect(wheeled.effects).toEqual([{ type: 'PreventDefault' }]);
    expect(wheeled.state).toEqual(dragging.state);
    const pinched = reduce(dragging.state, { type: 'GestureChange', scale: ZOOM_IN });
    expect(pinched.effects).toEqual([{ type: 'PreventDefault' }]);
    expect(pinched.state).toEqual(dragging.state);
  });

  it('rule 9: a Tick ≥ WHEEL_SETTLE_MS after the last wheel commits the gesture', () => {
    const wheeling = reduce(idle(), wheel({ dx: -50, at: 0 }));
    const early = reduce(wheeling.state, { type: 'Tick', at: WHEEL_SETTLE_MS - 1 });
    expect(early.state.phase.kind).toBe('wheeling');
    expect(early.effects).toEqual([]);
    const settled = reduce(wheeling.state, { type: 'Tick', at: WHEEL_SETTLE_MS });
    expect(settled.state.phase.kind).toBe('idle');
    expect(saves(settled.effects)).toHaveLength(1);
    near(saves(settled.effects)[0].values.rect.x, PANNED_X);
  });

  it('rule 10: PointerDown while wheeling commits the wheel then starts the drag — two Saves when both changed', () => {
    const result = run(idle(), [wheel({ dx: -50, at: 0 }), down(500, 350), move(550, 350), up()]);
    const all = saves(result.effects);
    expect(all).toHaveLength(2);
    near(all[0].values.rect.x, PANNED_X);
    near(all[1].values.rect.x, DEFAULT.x - 2 * (DEFAULT.x - PANNED_X));
    expect(result.state.phase.kind).toBe('idle');
  });

  it('rule 11: gesture events pinch about the anchor; the duplicate ctrl-wheel is ignored; a wheel zoom in flight commits first', () => {
    const start = reduce(idle(), { type: 'GestureStart', point: { x: 500, y: 350 } });
    expect(start.state.phase).toEqual({ kind: 'pinching', startRect: DEFAULT, anchor: { x: 0.5, y: 0.5 }, viaGestureEvents: true });
    const changed = reduce(start.state, { type: 'GestureChange', scale: ZOOM_IN });
    rectNear(changed.state.live, ZOOMED);
    expect(types(changed.effects)).toEqual(['PreventDefault', 'View']);
    const duplicate = reduce(changed.state, wheel({ dy: -100, ctrlKey: true }));
    expect(duplicate.effects).toEqual([{ type: 'PreventDefault' }]);
    expect(duplicate.state).toEqual(changed.state);
    const ended = reduce(duplicate.state, { type: 'GestureEnd' });
    expect(ended.state.phase.kind).toBe('idle');
    expect(saves(ended.effects)).toHaveLength(1);
    rectNear(saves(ended.effects)[0].values.rect, ZOOMED);
    const afterWheel = run(idle(), [wheel({ dy: -100, ctrlKey: true, at: 0 }), { type: 'GestureStart', point: { x: 500, y: 350 } }]);
    expect(saves(afterWheel.effects)).toHaveLength(1);
    expect(afterWheel.state.phase.kind).toBe('pinching');
  });

  it('rule 12: SwitchPhoto commits the active gesture for the old photo, then adopts the new photo and yields Switch', () => {
    const other = composition(2, { rect: { x: 0.2, y: 0, w: 0.666796875, h: 1 } });
    const result = run(idle(), [...DRAG_50, { type: 'SwitchPhoto', photoId: 2, composition: other }]);
    expect(types(result.effects)).toEqual(['View', 'Save', 'Switch', 'View']);
    expect(saves(result.effects)[0].photoId).toBe(1);
    near(saves(result.effects)[0].values.rect.x, PANNED_X);
    expect(result.effects[2]).toEqual({ type: 'Switch', photoId: 2 });
    expect(result.state.photoId).toBe(2);
    expect(result.state.phase.kind).toBe('idle');
    expect(result.state.values.rect).toEqual({ x: 0.2, y: 0, w: 0.666796875, h: 1 });
    expect(result.state.entry).toEqual(result.state.values);
    expect(result.state.live).toEqual(result.state.values.rect);
  });

  it('rule 13: KeyReturn commits an active gesture, then Exit', () => {
    const mid = run(idle(), [...DRAG_50, { type: 'KeyReturn' }]);
    expect(types(mid.effects)).toEqual(['View', 'Save', 'Exit']);
    expect(mid.state.phase.kind).toBe('idle');
    const quiet = reduce(idle(), { type: 'KeyReturn' });
    expect(quiet.effects).toEqual([{ type: 'Exit' }]);
  });

  it('rule 14: Nudge and SetValues are ignored during a pointer gesture; while wheeling the wheel commits first', () => {
    const dragging = run(idle(), DRAG_50);
    const other: CompositionValues = { ...dragging.state.values, rect: { x: 0.3, y: 0, w: 0.666796875, h: 1 } };
    expect(reduce(dragging.state, { type: 'Nudge', dx: 1, dy: 0 })).toEqual({ state: dragging.state, effects: [] });
    expect(reduce(dragging.state, { type: 'SetValues', values: other })).toEqual({ state: dragging.state, effects: [] });
    const nudged = run(idle(), [wheel({ dx: -50, at: 0 }), { type: 'Nudge', dx: 10, dy: 0 }]);
    expect(types(nudged.effects)).toEqual(['PreventDefault', 'View', 'Save', 'View', 'Save']);
    near(saves(nudged.effects)[0].values.rect.x, PANNED_X);
    near(saves(nudged.effects)[1].values.rect.x, PANNED_X - 10 / 977.81);
    expect(nudged.state.phase.kind).toBe('idle');
    const set = run(idle(), [wheel({ dx: -50, at: 0 }), { type: 'SetValues', values: other }]);
    expect(types(set.effects)).toEqual(['PreventDefault', 'View', 'Save', 'View', 'Save']);
    expect(saves(set.effects)[1].values).toEqual(other);
    expect(set.state.live).toEqual(other.rect);
  });

  it('rule 15: clamps — the rect never passes the edge nor shrinks below the minimum; with straighten a result outside the image is refused', () => {
    const edge = run(idle(), [down(500, 350), move(900, 350), up()]);
    expect(saves(edge.effects)[0].values.rect.x).toBe(0);
    const corner = run(edge.state, [down(500, 350), move(100, 600), up()]);
    near(saves(corner.effects)[0].values.rect.x, 1 - DEFAULT.w, 1e-9);
    expect(saves(corner.effects)[0].values.rect.y).toBe(0);
    const out = reduce(idle(), wheel({ dy: 2000, ctrlKey: true }));
    rectNear(out.state.live, DEFAULT, 1e-9);
    const sliver = reduce(idle(), wheel({ dy: -3000, ctrlKey: true }));
    near(sliver.state.live.h, 0.05, 1e-9);
    near(sliver.state.live.w, 0.03334);
    const tilted = initialState(1, composition(1, { straighten: 5, rect: { x: 0.33330078125, y: 0.25, w: 0.3333984375, h: 0.5 } }), STAGE)!;
    const refused = run(tilted, [down(500, 350), move(1500, 350)]); // would clamp to x = 0: the left corners leave the tilted image
    expect(refused.state.live).toEqual(tilted.live);
    expect(refused.effects).toEqual([{ type: 'View', rect: tilted.live }]);
    const accepted = run(tilted, [down(500, 350), move(505, 350)]);
    expect(accepted.state.live).not.toEqual(tilted.live);
  });

  it('rule 16: a Saved for another photo changes nothing; a stored composition that differs snaps values and live to it', () => {
    const start = idle();
    const other = reduce(start, { type: 'Saved', photoId: 2, composition: composition(2, { rect: { x: 0.3, y: 0, w: 0.666796875, h: 1 } }) });
    expect(other).toEqual({ state: start, effects: [] });
    const same = reduce(start, { type: 'Saved', photoId: 1, composition: composition(1, { revision: 1 }) });
    expect(same).toEqual({ state: start, effects: [] });
    const dragged = run(start, [...DRAG_50, up()]);
    const refused = reduce(dragged.state, { type: 'Saved', photoId: 1, composition: composition(1) });
    expect(refused.state.values.rect).toEqual(DEFAULT);
    expect(refused.state.live).toEqual(DEFAULT);
    expect(refused.effects).toEqual([{ type: 'View', rect: DEFAULT }]);
    const midDrag = run(start, DRAG_50);
    const ignored = reduce(midDrag.state, { type: 'Saved', photoId: 1, composition: composition(1, { rect: { x: 0.3, y: 0, w: 0.666796875, h: 1 } }) });
    expect(ignored).toEqual({ state: midDrag.state, effects: [] });
  });

  it('extension (Review log): Resize recomputes the view for the new stage box and yields View', () => {
    const resized = reduce(idle(), { type: 'Resize', stage: { left: 0, top: 0, width: 800, height: 500 } });
    expect(resized.state.view.frameBox).toEqual({ left: 174, top: 24, width: 452, height: 452 });
    expect(resized.effects).toEqual([{ type: 'View', rect: DEFAULT }]);
  });
});
