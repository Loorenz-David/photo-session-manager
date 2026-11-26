// LV3·U2 — the main screen (design README §2, §Main viewport gestures, §Shutter; master_plan_live
// §5 PH4, PH7, §6): rows LV3·C8–C11, C13, C14. The viewport is 390 × 390 at (0, 110) in a
// 390 × 610 stage (shot ratio 1.0); the overlay starts at 35 %.
import { act, fireEvent, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import type { PhoneSession } from '../api';
import { advance, C, chairSession, frameBytes, lift, macChanges, type PhoneEnv, phoneEnv, press, slide, startPhone } from './helpers';

const viewport = () => screen.getByTestId('viewport');
const pill = () => screen.getByTestId('mode-pill').textContent;
const toast = () => screen.getByTestId('toast');
const lastDemand = (env: PhoneEnv) => env.socket().demands().at(-1);
const buzzed = (env: PhoneEnv, pattern: number | number[]) => env.vibrate.mock.calls.filter(call => JSON.stringify(call[0]) === JSON.stringify(pattern)).length;
const freeShot = (session: PhoneSession) => {
  session.current = { kind: 'free' };
};

/** Slide up 55 px from (195, 405) and let go: live is locked (LV6: LV-D19 (3) halved the design's 110 px). */
function lock(): void {
  press(viewport(), 195, 405);
  slide(viewport(), 197, 390);
  slide(viewport(), 195, 350);
  lift(viewport(), 195, 350);
}
function tap(x: number, y: number): void {
  press(viewport(), x, y);
  lift(viewport(), x, y);
}

// ---- LV3·C8: slide up to lock, slide sideways for the overlay ----

test('lv3_c8a_sliding_up_past_14_px_shows_the_lock_control_at_its_progress', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 195, 405);
  expect(screen.queryByTestId('lock-control')).toBeNull();
  slide(viewport(), 197, 390);
  expect(screen.getByTestId('lock-control')).toBeTruthy();
  // 15 of 55 px.
  expect(screen.getByTestId('lock-fill').style.height).toBe('27%');
  expect(pill()).toBe('LIVE');
});

test('lv3_c8b_sliding_up_55_px_locks_live_and_it_stays_on_after_release', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 195, 405);
  slide(viewport(), 197, 390);
  slide(viewport(), 195, 350);
  expect(buzzed(env, [25, 40, 70])).toBe(1);
  expect(pill()).toBe('LIVE · LOCKED');
  lift(viewport(), 195, 350);
  await advance(1000);
  expect(pill()).toBe('LIVE · LOCKED');
  expect(lastDemand(env)).toMatchObject({ live: true, visible: true });
  expect(env.socket().demands().filter(d => !d.live)).toEqual([]);
});

test('lv3_c8c_releasing_before_the_end_does_not_lock_and_the_control_fades', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 195, 405);
  slide(viewport(), 197, 390);
  slide(viewport(), 195, 365);
  lift(viewport(), 195, 365);
  await advance(0);
  expect(pill()).toBe('TEMPLATE');
  expect(screen.getByTestId('lock-control').style.opacity).toBe('0');
  await advance(260);
  expect(screen.queryByTestId('lock-control')).toBeNull();
  expect(lastDemand(env)).toMatchObject({ live: false });
  expect(buzzed(env, [25, 40, 70])).toBe(0);
});

test('lv3_c8d_sliding_sideways_sets_the_template_overlay_with_one_tick_at_40', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 100, 300);
  slide(viewport(), 120, 303);
  // round(35 + 20 / 390 · 130) = 42
  expect(screen.getByTestId('overlay-bar').textContent).toBe('TEMPLATE 42%');
  expect(buzzed(env, 5)).toBe(1);
  slide(viewport(), 121, 303);
  expect(buzzed(env, 5)).toBe(1);
  // The value is what the template shows through live.
  await env.socket().binary(frameBytes({ seq: 1, gen: 1 }));
  expect(screen.getByTestId('template-layer').style.opacity).toBe('0.42');
});

test('lv3_c8e_free_shot_has_no_overlay', async () => {
  const env = phoneEnv();
  await startPhone(env, chairSession(freeShot));
  press(viewport(), 100, 300);
  slide(viewport(), 140, 303);
  expect(screen.queryByTestId('overlay-bar')).toBeNull();
  lift(viewport(), 140, 303);
  // Back on a template shot the overlay is still the default 35 %.
  await macChanges(env, chairSession());
  press(viewport(), 100, 300);
  await env.socket().binary(frameBytes({ seq: 1, gen: lastDemand(env)!.gen }));
  expect(screen.getByTestId('template-layer').style.opacity).toBe('0.35');
});

// ---- LV3·C9: hold for live ----

test('lv3_c9a_a_press_shows_live_at_once', async () => {
  const env = phoneEnv();
  await startPhone(env);
  expect(pill()).toBe('TEMPLATE');
  press(viewport(), 195, 305);
  expect(buzzed(env, 10)).toBe(1);
  expect(pill()).toBe('LIVE');
  expect(env.socket().demands()).toEqual([{ type: 'demand', gen: 1, live: true, visible: true }]);
});

test('lv3_c9b_a_cancelled_touch_goes_back_to_the_template', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 195, 305);
  fireEvent.pointerCancel(viewport(), { pointerId: 1 });
  expect(pill()).toBe('TEMPLATE');
  expect(lastDemand(env)).toEqual({ type: 'demand', gen: 2, live: false, visible: true });
});

test('lv3_c9c_the_window_losing_focus_releases_the_hold', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 195, 305);
  act(() => {
    window.dispatchEvent(new Event('blur'));
  });
  expect(pill()).toBe('TEMPLATE');
  expect(lastDemand(env)).toEqual({ type: 'demand', gen: 2, live: false, visible: true });
});

// ---- LV3·C10: taps while locked ----

test('lv3_c10a_a_tap_while_locked_focuses_there_after_260_ms', async () => {
  const env = phoneEnv();
  await startPhone(env);
  lock();
  tap(195, 305);
  await advance(259);
  expect(env.mac.posts('/focus')).toEqual([]);
  expect(screen.queryByTestId('reticle')).toBeNull();
  await advance(1);
  expect(env.mac.posts('/focus')).toEqual([{ session_id: 7, x: 0.5, y: 0.5 }]);
  expect(buzzed(env, 25)).toBe(1);
  const reticle = screen.getByTestId('reticle');
  expect([reticle.style.left, reticle.style.top]).toEqual(['50%', '50%']);
});

test('lv3_c10a_the_focus_point_is_the_tapped_point_of_the_live_frame', async () => {
  const env = phoneEnv();
  await startPhone(env);
  lock();
  await env.socket().binary(frameBytes({ seq: 1, gen: 1 }));
  // The left edge of the square viewport, a quarter down: the 1024 × 680 frame is cropped left and right.
  tap(0, 110 + 97.5);
  await advance(260);
  const [body] = env.mac.posts('/focus') as { session_id: number; x: number; y: number }[];
  expect(body.session_id).toBe(7);
  expect(body.x).toBeCloseTo(0.16797, 4);
  expect(body.y).toBeCloseTo(0.25, 4);
  const reticle = screen.getByTestId('reticle');
  expect([reticle.style.left, reticle.style.top]).toEqual(['0%', '25%']);
});

test('lv3_c10b_a_double_tap_unlocks_and_does_not_focus', async () => {
  const env = phoneEnv();
  await startPhone(env);
  lock();
  tap(195, 305);
  await advance(200);
  tap(215, 305);
  await advance(1000);
  expect(env.mac.posts('/focus')).toEqual([]);
  expect(pill()).toBe('TEMPLATE');
  expect(buzzed(env, [20, 30, 20])).toBe(1);
  expect(lastDemand(env)).toMatchObject({ live: false, visible: true });
});

test('lv3_c10c_a_tap_while_not_locked_does_not_focus', async () => {
  const env = phoneEnv();
  await startPhone(env);
  tap(195, 305);
  await advance(1000);
  expect(env.mac.posts('/focus')).toEqual([]);
  expect(screen.queryByTestId('reticle')).toBeNull();
});

// ---- LV3·C11: the AF chip ----

async function focusedAt(env: PhoneEnv): Promise<void> {
  env.mac.answer('POST /api/phone/v1/focus', 200, { focus_id: 'f1' });
  lock();
  tap(195, 305);
  await advance(260);
}

test('lv3_c11a_the_cameras_focused_report_shows_af_locked', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await focusedAt(env);
  expect(screen.queryByTestId('af-chip')).toBeNull();
  await env.socket().text({ type: 'focus', focus_id: 'f1', outcome: 'focused', reason: null });
  expect(screen.getByTestId('af-chip').textContent).toBe('AF LOCKED×');
});

test('lv3_c11b_the_x_recentres_the_cameras_focus_and_removes_the_marker', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await focusedAt(env);
  await env.socket().text({ type: 'focus', focus_id: 'f1', outcome: 'focused', reason: null });
  // (The press that began the lock slide already ticked 10 once.)
  const ticks = buzzed(env, 10);
  fireEvent.pointerDown(screen.getByRole('button', { name: 'Clear focus' }), { pointerId: 2 });
  await advance(0);
  expect(env.mac.posts('/focus/clear')).toEqual([{ session_id: 7 }]);
  expect(buzzed(env, 10)).toBe(ticks + 1);
  expect(screen.queryByTestId('af-chip')).toBeNull();
  expect(screen.queryByTestId('reticle')).toBeNull();
  expect(pill()).toBe('LIVE · LOCKED');
});

test('lv3_c11c_not_focused_says_could_not_focus_there', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await focusedAt(env);
  await env.socket().text({ type: 'focus', focus_id: 'f1', outcome: 'not_focused', reason: null });
  expect(toast().textContent).toBe('Could not focus there');
  expect(toast().style.opacity).toBe('1');
  expect(screen.queryByTestId('af-chip')).toBeNull();
});

test('lv3_c11c_a_focus_area_that_cannot_move_says_what_to_set_on_the_camera', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await focusedAt(env);
  await env.socket().text({ type: 'focus', focus_id: 'f1', outcome: 'refused', reason: 'focus_area_not_movable' });
  expect(toast().textContent).toBe("Set the camera's Focus Area to a movable spot (e.g. Flexible Spot) to focus from the phone");
  expect(toast().style.opacity).toBe('1');
});

// ---- LV3·C13: viewport, layers, readout, Next ----

test('lv3_c13a_a_long_shot_turns_the_viewport_and_the_buttons', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const menu = () => screen.getByRole('button', { name: 'Item' });
  const next = () => screen.getByTestId('next').parentElement!;
  expect(viewport().style.transform).toBe('rotate(0deg)');
  expect([menu().style.transform, next().style.transform]).toEqual(['rotate(0deg)', 'rotate(0deg)']);

  await macChanges(env, chairSession(s => {
    s.shots[1].ratio = 1.5;
  }));
  expect(viewport().style.transform).toBe('rotate(90deg)');
  expect([menu().style.transform, next().style.transform]).toEqual(['rotate(90deg)', 'rotate(90deg)']);

  // A shot that keeps the camera's own shape takes the live frame's (1024 × 680).
  await macChanges(env, chairSession(s => {
    s.shots[1].ratio = null;
  }));
  press(viewport(), 195, 305);
  await env.socket().binary(frameBytes({ seq: 1, gen: 1, width: 1024, height: 680 }));
  expect(viewport().style.transform).toBe('rotate(90deg)');
  expect(parseFloat(viewport().style.width) / parseFloat(viewport().style.height)).toBeCloseTo(1024 / 680, 3);
  expect(1024 / 680).toBeCloseTo(1.506, 3);
});

test('lv3_c13b_a_split_shot_shows_the_centre_guide_and_left_right', async () => {
  const env = phoneEnv();
  await startPhone(env, chairSession(s => {
    s.shots[1].split = true;
    s.shots[1].ratio = 2;
  }));
  expect(screen.getByTestId('split-line').style.left).toBe('50%');
  expect(screen.getByText('Left · 1:1').style.left).toBe('25%');
  expect(screen.getByText('Right · 1:1').style.left).toBe('75%');
  // A shot that is not split has no guide.
  await macChanges(env, chairSession());
  expect(screen.queryByTestId('split-line')).toBeNull();
  expect(screen.queryByText('Left · 1:1')).toBeNull();
});

test('lv3_c13c_the_template_is_full_then_the_overlay_then_absent_in_free_shot', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const template = () => screen.getByTestId('template-layer').style.opacity;
  expect(template()).toBe('1');
  press(viewport(), 195, 305);
  await env.socket().binary(frameBytes({ seq: 1, gen: 1 }));
  expect(template()).toBe('0.35');
  lift(viewport(), 195, 305);
  await macChanges(env, chairSession(freeShot));
  expect(template()).toBe('0');
});

test('lv3_c13d_the_exposure_readout_shows_on_live', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await env.socket().text({ type: 'camera', status: 'connected', live: 'live', reason: null, exposure: { shutter: '1/125', aperture: 'f/8', iso: 'ISO 100' } });
  expect(screen.queryByText('1/125 · f/8 · ISO 100')).toBeNull();
  press(viewport(), 195, 305);
  await env.socket().binary(frameBytes({ seq: 1, gen: 1 }));
  expect(screen.getByText('1/125 · f/8 · ISO 100')).toBeTruthy();
});

test('lv3_c13e_next_goes_to_the_next_empty_shot_in_shooting_order_then_to_free_shot', async () => {
  const env = phoneEnv();
  // Shooting order B (02, filled), A (01, current), C (03, empty).
  await startPhone(env);
  const next = () => screen.getByTestId('next');
  expect(screen.getByText('Next shot')).toBeTruthy();
  next().click();
  await advance(0);
  expect(env.mac.posts('/current')).toEqual([{ session_id: 7, container_id: C }]);

  // On the last template shot the next stop is Free shot.
  await macChanges(env, chairSession(s => {
    s.current = { kind: 'shot', container_id: C };
  }));
  expect(screen.getByText('Free shot')).toBeTruthy();
  next().click();
  await advance(0);
  expect(env.mac.posts('/current')).toEqual([{ session_id: 7, container_id: C }, { session_id: 7, container_id: null }]);

  // In Free shot there is no next.
  await macChanges(env, chairSession(freeShot));
  expect(next().style.opacity).toBe('0.3');
  next().click();
  await advance(0);
  expect(env.mac.posts('/current')).toHaveLength(2);
});

// ---- LV3·C14: the shutter ----

const shutter = () => screen.getByRole('button', { name: 'Capture' });
async function shoot(): Promise<void> {
  fireEvent.pointerDown(shutter(), { pointerId: 3 });
  fireEvent.pointerUp(shutter(), { pointerId: 3 });
  await advance(0);
}
/** The plan's fixture with a fourth shot D (final 04), empty and current. */
const D = 14;
const withFourth = (photo: number | null) => chairSession(s => {
  s.shots.push({ container_id: D, final_no: 4, shoot_no: 4, name: 'Detail', ratio: 1, split: false, zoom: 0, example: false, photo: photo === null ? null : { photo_id: photo, revision: 1, ratio: 1, preview: 'k0' } });
  s.extras = [];
  s.current = { kind: 'shot', container_id: D };
});

test('lv3_c14a_the_shutter_presses_in_then_captures_with_a_flash', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const disc = () => screen.getByTestId('shutter-disc').style.transform;
  const flash = () => screen.getByTestId('flash').style;
  expect(disc()).toBe('scale(1)');
  fireEvent.pointerDown(shutter(), { pointerId: 3 });
  expect(buzzed(env, 20)).toBe(1);
  expect(disc()).toBe('scale(0.86)');
  expect(env.mac.posts('/capture')).toEqual([]);
  fireEvent.pointerUp(shutter(), { pointerId: 3 });
  expect(env.mac.posts('/capture')).toEqual([{ session_id: 7 }]);
  expect(flash().opacity).toBe('1');
  expect(buzzed(env, [80, 40, 120])).toBe(1);
  expect(disc()).toBe('scale(1)');
  await advance(70);
  expect(flash().opacity).toBe('0');
  expect(flash().transition).toContain('.45s');
});

test('lv3_c14b_sliding_off_the_shutter_cancels', async () => {
  const env = phoneEnv();
  await startPhone(env);
  fireEvent.pointerDown(shutter(), { pointerId: 3 });
  fireEvent.pointerLeave(shutter(), { pointerId: 3 });
  fireEvent.pointerUp(shutter(), { pointerId: 3 });
  await advance(0);
  expect(env.mac.posts('/capture')).toEqual([]);
  expect(buzzed(env, [80, 40, 120])).toBe(0);
});

test('lv3_c14c_the_toast_names_the_shot_once_the_mac_shows_where_the_photo_landed', async () => {
  const env = phoneEnv();
  await startPhone(env, withFourth(null));
  env.mac.answer('POST /api/phone/v1/capture', 200, { capture_id: 'c1' });
  await shoot();
  await env.socket().text({ type: 'capture', capture_id: 'c1', state: 'acknowledged' });
  await env.socket().text({ type: 'capture', capture_id: 'c1', state: 'exposed' });
  // The camera fired; the file is not there yet.
  await macChanges(env, withFourth(null));
  expect(toast().style.opacity).toBe('0');
  await macChanges(env, withFourth(300));
  expect(toast().textContent).toBe('Shot 04 captured');
  expect(toast().style.opacity).toBe('1');
});

test('lv3_c14d_a_capture_bound_to_no_shot_is_a_condition_image', async () => {
  const env = phoneEnv();
  await startPhone(env, chairSession(freeShot));
  env.mac.answer('POST /api/phone/v1/capture', 200, { capture_id: 'c1' });
  await shoot();
  await env.socket().text({ type: 'capture', capture_id: 'c1', state: 'exposed' });
  await macChanges(env, chairSession(s => {
    freeShot(s);
    s.condition.push({ photo_id: 301, revision: 1, ratio: 1, preview: 'k0' });
  }));
  expect(toast().textContent).toBe('Condition image added');
  expect(toast().style.opacity).toBe('1');
});

test('lv3_c14e_no_camera_says_camera_not_connected', async () => {
  const env = phoneEnv();
  await startPhone(env);
  env.mac.answer('POST /api/phone/v1/capture', 409, { error: { code: 'camera_not_connected', message: 'camera_not_connected' } });
  await shoot();
  expect(toast().textContent).toBe('Camera not connected');
  expect(toast().style.opacity).toBe('1');
});

test('lv3_c14e_a_capture_the_camera_did_not_take_says_not_taken', async () => {
  const env = phoneEnv();
  await startPhone(env);
  env.mac.answer('POST /api/phone/v1/capture', 200, { capture_id: 'c1' });
  await shoot();
  await env.socket().text({ type: 'capture', capture_id: 'c1', state: 'failed' });
  expect(toast().textContent).toBe('Not taken. Try again.');
  expect(toast().style.opacity).toBe('1');
});

test('lv3_c14e_a_session_changed_on_the_mac_says_so_and_is_read_again', async () => {
  const env = phoneEnv();
  await startPhone(env);
  env.mac.answer('POST /api/phone/v1/capture', 409, { error: { code: 'session_changed', message: 'session_changed' } });
  const reads = env.mac.gets('/session').length;
  await shoot();
  expect(toast().textContent).toBe('Session changed on the Mac');
  expect(env.mac.gets('/session')).toHaveLength(reads + 1);
});

test('lv3_c14f_next_follows_the_macs_current_shot_and_the_phone_never_advances_by_itself', async () => {
  const env = phoneEnv();
  await startPhone(env);
  env.mac.answer('POST /api/phone/v1/capture', 200, { capture_id: 'c1' });
  await shoot();
  await env.socket().text({ type: 'capture', capture_id: 'c1', state: 'exposed' });
  await advance(2000);
  // Nothing moved on the phone: the Mac has not said so.
  expect(screen.getByText('Next shot')).toBeTruthy();
  expect(env.mac.posts('/current')).toEqual([]);
  // The Mac filled A and made C (the last shot) current.
  await macChanges(env, chairSession(s => {
    s.shots[1].photo = { photo_id: 302, revision: 1, ratio: 1, preview: 'k0' };
    s.current = { kind: 'shot', container_id: C };
  }));
  expect(screen.getByText('Free shot')).toBeTruthy();
  expect(env.mac.posts('/current')).toEqual([]);
});
