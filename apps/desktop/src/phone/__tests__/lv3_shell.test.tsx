// LV3·U1 — the shell (master_plan_live §5 PH7, design README §1, 10J §Status): rows LV3·C1–C4, C6, C7.
import { screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, chairSession, frameBytes, lift, mountPhone, phoneEnv, press, startPhone } from './helpers';

const viewport = () => screen.getByTestId('viewport');
const live = () => screen.getByTestId('live-layer');
const template = () => screen.getByTestId('template-layer');

// ---- LV3·C1: the token ----

test('lv3_c1a_the_token_from_the_qr_link_is_kept_and_leaves_the_address', async () => {
  const env = phoneEnv({ location: '/phone/?pair=T' });
  await mountPhone();
  expect(window.localStorage.getItem('psm.phone.token')).toBe('T');
  expect(window.location.pathname + window.location.search).toBe('/phone/');
  const manifest = document.head.querySelector('link[rel="manifest"]');
  expect(manifest?.getAttribute('href')).toBe('/phone/manifest.webmanifest?pair=T');
  const hello = env.mac.gets('/hello');
  expect(hello).toHaveLength(1);
  expect(hello[0].authorization).toBe('Bearer T');
});

test('lv3_c1b_without_a_token_the_phone_asks_for_the_qr_code_and_requests_nothing', async () => {
  const env = phoneEnv();
  await mountPhone();
  expect(screen.getByText('Scan the QR code in Session Manager → Settings → Phone to connect this phone.')).toBeTruthy();
  expect(env.mac.requests).toEqual([]);
  expect(env.sockets).toEqual([]);
});

test('lv3_c1c_a_removed_phone_forgets_its_token_and_says_so', async () => {
  const env = phoneEnv({ storedToken: 'T' });
  env.mac.answer('GET /api/phone/v1/hello', 401, { error: { code: 'pairing_unknown', message: 'pairing_unknown' } });
  await mountPhone();
  expect(window.localStorage.getItem('psm.phone.token')).toBeNull();
  expect(screen.getByText('This phone was removed in Session Manager. Delete this app from the home screen, then scan the new QR code in Session Manager → Settings → Phone and add it again.')).toBeTruthy();
});

// ---- LV3·C2: the connecting overlay ----

test('lv3_c2a_until_the_mac_answers_the_overlay_shows_and_a_press_demands_nothing', async () => {
  const env = phoneEnv({ storedToken: 'T' });
  env.mac.view = { session: chairSession() };
  env.mac.hold();
  await mountPhone();
  await env.socket().accept();
  expect(screen.getByText('Connecting to session')).toBeTruthy();
  expect(screen.getByRole('progressbar')).toBeTruthy();
  expect(screen.getByTestId('connecting').style.opacity).toBe('1');
  press(viewport(), 195, 300);
  await advance(300);
  lift(viewport(), 195, 300);
  expect(env.socket().demands()).toEqual([]);
});

test('lv3_c2b_once_both_answered_the_overlay_fades_leaves_after_400_ms_and_the_stage_takes_presses', async () => {
  const env = phoneEnv({ storedToken: 'T' });
  env.mac.view = { session: chairSession() };
  env.mac.hold();
  await mountPhone();
  await env.socket().accept();
  await env.mac.release();
  expect(screen.getByTestId('connecting').style.opacity).toBe('0');
  await advance(399);
  expect(screen.queryByTestId('connecting')).not.toBeNull();
  await advance(1);
  expect(screen.queryByTestId('connecting')).toBeNull();
  press(viewport(), 195, 300);
  expect(env.socket().demands()).toHaveLength(1);
});

// ---- LV3·C3: demand ----

test('lv3_c3a_hold_and_release_send_two_demands', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 195, 300);
  await advance(500);
  lift(viewport(), 195, 300);
  expect(env.socket().demands()).toEqual([
    { type: 'demand', gen: 1, live: true, visible: true },
    { type: 'demand', gen: 2, live: false, visible: true },
  ]);
});

test('lv3_c3b_the_screen_going_off_and_on_sends_two_demands_each_with_a_new_gen', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await env.setVisibility('hidden');
  await env.setVisibility('visible');
  expect(env.socket().demands()).toEqual([
    { type: 'demand', gen: 1, live: false, visible: false },
    { type: 'demand', gen: 2, live: false, visible: true },
  ]);
});

// ---- LV3·C4: frames ----

test('lv3_c4a_a_frame_of_an_older_generation_is_not_drawn', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 195, 300);
  expect(env.socket().demands().at(-1)).toMatchObject({ gen: 1, live: true });
  await env.socket().binary(frameBytes({ seq: 5, gen: 0 }));
  expect(env.drawImage).not.toHaveBeenCalled();
});

test('lv3_c4b_only_a_frame_newer_than_the_last_is_drawn', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 195, 300);
  await env.socket().binary(frameBytes({ seq: 10, gen: 1 }));
  await env.socket().binary(frameBytes({ seq: 9, gen: 1 }));
  await env.socket().binary(frameBytes({ seq: 10, gen: 1 }));
  expect(env.drawImage).toHaveBeenCalledTimes(1);
});

test('lv3_c4c_live_shows_only_from_the_first_frame_of_the_new_generation', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(viewport(), 195, 300);
  await advance(200);
  expect(live().style.opacity).toBe('0');
  expect(template().style.opacity).toBe('1');
  await env.socket().binary(frameBytes({ seq: 1, gen: 1 }));
  expect(live().style.opacity).toBe('1');
  expect(template().style.opacity).toBe('0.35');
});

// ---- LV3·C6: Full screen ----

test('lv3_c6a_a_browser_that_can_go_full_screen_offers_the_button_and_a_tap_asks_once', async () => {
  const env = phoneEnv({ fullscreenEnabled: true });
  await startPhone(env);
  const button = screen.getByRole('button', { name: 'Full screen' });
  button.click();
  expect(env.requestFullscreen).toHaveBeenCalledTimes(1);
});

test('lv3_c6b_no_button_in_the_installed_app', async () => {
  const env = phoneEnv({ fullscreenEnabled: true, standalone: true });
  await startPhone(env);
  expect(screen.queryByRole('button', { name: 'Full screen' })).toBeNull();
});

test('lv3_c6b_no_button_when_already_full_screen', async () => {
  const env = phoneEnv({ fullscreenEnabled: true, fullscreenElement: true });
  await startPhone(env);
  expect(screen.queryByRole('button', { name: 'Full screen' })).toBeNull();
});

// ---- LV3·C7: the camera's status where the hint is ----

const camera = (over: object) => ({ type: 'camera', status: 'connected', live: 'live', reason: null, exposure: null, ...over });

test('lv3_c7a_holding_with_no_camera_says_camera_not_connected', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await env.socket().text(camera({ status: 'disconnected', live: 'off' }));
  expect(screen.queryByText('Camera not connected')).toBeNull();
  press(viewport(), 195, 300);
  expect(screen.getByText('Camera not connected')).toBeTruthy();
});

test('lv3_c7b_holding_with_a_camera_that_has_no_live_view_says_so', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await env.socket().text(camera({ live: 'unavailable', reason: 'not_supported' }));
  press(viewport(), 195, 300);
  expect(screen.getByText('Live unavailable on this camera')).toBeTruthy();
});

test('lv3_c7c_holding_with_a_busy_camera_says_camera_busy', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await env.socket().text(camera({ status: 'busy' }));
  press(viewport(), 195, 300);
  expect(screen.getByText('Camera busy')).toBeTruthy();
});
