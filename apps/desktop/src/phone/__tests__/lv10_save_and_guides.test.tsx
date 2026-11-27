// LV10·U4 — Done saves into the Mac's composition; refusals; the composition guides on the live view (book 10J
// LV-D21 (6), (7), 10D Overlay; master_plan_live §5 PH4b, PH12, §6 texts v1.3; plan points P7, P10): rows
// LV10·C14–C16.
// C14–C15 on fixture K (helpers `composedSession`: session 7, photo 201 at revision 1 with the default 1:1 framing
// of its 2560 × 1707 frame), the editor opened from the Preview of 201 ("Photo details", "Reframe"); its stage is
// 390 × 520, the frame box's centre (195, 260). C16 on the live screen: the viewport of a 1:1 shot is 390 × 390 at
// (0, 110) in the 390 × 610 stage.
import { cleanup, screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { transformCss, viewTransform } from '@/features/compose/geometry';
import { overlayLines } from '@/features/compose/overlays';
import {
  advance, chairSession, composed45Session, composedSession, K_FRAME, K_VALUES, lift, macChanges, type PhoneEnv, phoneEnv, press, REFRAME_STAGE, slide, startPhone,
} from './helpers';

const X_B = 44;
const Y = 742;
const SAVE = 'POST /api/phone/v1/photos/201/composition';
const dock = () => screen.getByTestId('dock');
const preview = () => screen.getByTestId('preview');
const editor = () => screen.queryByTestId('reframe-editor');
const stage = () => screen.getByTestId('reframe-stage');
const done = () => within(editor()!).getByRole('button', { name: 'Done' }) as HTMLButtonElement;
const toast = () => screen.getByTestId('toast').textContent;
const sessionReads = (env: PhoneEnv) => env.mac.gets('/session').length;

async function click(element: HTMLElement): Promise<void> {
  element.click();
  await advance(0);
}
/** A paired phone with the editor open on photo 201 of `session`; what happened before is forgotten. */
async function openEditor(session = composedSession()): Promise<PhoneEnv> {
  const env = phoneEnv();
  await startPhone(env, session);
  press(dock(), X_B, Y);
  lift(dock(), X_B, Y);
  await advance(0);
  await click(within(preview()).getByRole('button', { name: 'Photo details' }));
  await click(within(screen.getByTestId('photo-drawer')).getByRole('button', { name: 'Reframe' }));
  expect(editor()).not.toBeNull();
  env.mac.requests.length = 0;
  return env;
}
/** C10(a)'s pan: one finger 34.2 px to the right (a tenth of the frame box): the rect moves left by a tenth of its width. */
function pan(): void {
  press(stage(), 195, 260);
  slide(stage(), 229.2, 260);
  lift(stage(), 229.2, 260);
}
const PANNED = { ...K_VALUES, rect: { ...K_VALUES.rect, x: 0.0999219 } };
/** The picture is where the Mac's view maths puts it for C10(a)'s framing (within a step of the transform's 0.01 px). */
function showsPanned(): boolean {
  const number = (text: string) => /translate\((-?[\d.]+)px/.exec(text)![1];
  const want = Number(number(transformCss(viewTransform(PANNED, K_FRAME, REFRAME_STAGE))));
  return Math.abs(Number(number(screen.getByTestId('reframe-picture').style.transform)) - want) < 0.0101;
}
const refusal = (env: PhoneEnv, status: number, code: string) => env.mac.answer(SAVE, status, { error: { code, message: code } });

// ---- LV10·C14: Done ----

test('lv10_c14a_done_saves_the_framing_into_the_macs_composition', async () => {
  const env = await openEditor();
  pan();
  await click(done());
  const posts = env.mac.allPosts() as [string, { session_id: number; revision: number; values: typeof K_VALUES }][];
  expect(posts.map(post => post[0])).toEqual(['/photos/201/composition']);
  const body = posts[0][1];
  expect({ ...body, values: { ...body.values, rect: null } }).toEqual({
    session_id: 7, revision: 1,
    values: { kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: null },
  });
  expect(Object.keys(body.values.rect).sort()).toEqual(['h', 'w', 'x', 'y']);
  expect(Math.abs(body.values.rect.x - 0.0999219)).toBeLessThan(1e-6);
  expect(body.values.rect.y).toBe(0);
  expect(Math.abs(body.values.rect.w - 0.6667969)).toBeLessThan(1e-6);
  expect(body.values.rect.h).toBe(1);
  expect(editor()).toBeNull();
  expect(toast()).toBe('Framing saved');
  // The Mac is asked what the session is now (after the save's answer).
  const order = env.mac.requests.map(request => `${request.method} ${request.path.replace('/api/phone/v1', '')}`);
  expect(order).toEqual(['POST /photos/201/composition', 'GET /session']);
});

test('lv10_c14b_done_without_a_change_sends_nothing', async () => {
  const env = await openEditor();
  await click(done());
  expect(editor()).toBeNull();
  expect(env.mac.requests).toEqual([]);
  expect(toast()).toBe('');
  // The Preview is still there under it.
  expect(screen.getByTestId('preview')).toBeTruthy();
});

test('lv10_c14c_done_after_reset_asks_the_mac_for_its_own_reset', async () => {
  const env = await openEditor(composed45Session());
  await click(within(editor()!).getByRole('button', { name: 'Reset' }));
  await click(done());
  expect(env.mac.allPosts()).toEqual([['/photos/201/composition/reset', { session_id: 7, revision: 1 }]]);
  expect(editor()).toBeNull();
  expect(toast()).toBe('Framing saved');
});

test('lv10_c14d_one_request_per_done', async () => {
  const env = await openEditor();
  pan();
  env.mac.hold();
  await click(done());
  expect(done().disabled).toBe(true);
  await click(done());
  expect(env.mac.allPosts()).toHaveLength(1);
  expect(editor()).not.toBeNull();
  await env.mac.release();
  await advance(0);
  expect(env.mac.allPosts()).toHaveLength(1);
  expect(editor()).toBeNull();
});

// ---- LV10·C15: refusals ----

test('lv10_c15a_a_framing_changed_on_the_mac_closes_the_editor_and_says_so', async () => {
  const env = await openEditor();
  refusal(env, 409, 'composition_changed');
  pan();
  await click(done());
  expect(env.mac.allPosts()).toHaveLength(1);
  expect(editor()).toBeNull();
  expect(toast()).toBe('Framing changed on the Mac. Open Reframe again.');
});

test('lv10_c15b_a_refused_framing_stays_in_the_editor', async () => {
  const env = await openEditor();
  refusal(env, 400, 'composition_invalid');
  pan();
  await click(done());
  expect(env.mac.allPosts()).toHaveLength(1);
  expect(editor()).not.toBeNull();
  expect(showsPanned()).toBe(true);
  expect(toast()).toBe('That framing could not be saved.');
  // Done can be tried again.
  expect(done().disabled).toBe(false);
});

test('lv10_c15c_a_session_changed_on_the_mac_closes_the_editor_and_rereads', async () => {
  const env = await openEditor();
  refusal(env, 409, 'session_changed');
  pan();
  await click(done());
  expect(editor()).toBeNull();
  expect(toast()).toBe('Session changed on the Mac');
  const order = env.mac.requests.map(request => `${request.method} ${request.path.replace('/api/phone/v1', '')}`);
  expect(order).toEqual(['POST /photos/201/composition', 'GET /session']);
});

test('lv10_c15d_the_editor_closes_when_its_photo_leaves_the_session', async () => {
  const env = await openEditor();
  pan();
  await macChanges(env, composedSession(session => {
    session.shots[0].photo = null;
    session.counts.main = 1;
  }));
  expect(editor()).toBeNull();
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
});

// ---- LV10·C16: the composition guides on the live view ----

const viewport = () => screen.getByTestId('viewport');
const liveGuides = () => within(viewport()).queryByTestId('live-guides');
const lines = () => Array.from(liveGuides()?.querySelectorAll('line') ?? [])
  .map(line => ({ x1: Number(line.getAttribute('x1')), y1: Number(line.getAttribute('y1')), x2: Number(line.getAttribute('x2')), y2: Number(line.getAttribute('y2')) }));
const guidesRow = () => within(screen.getByTestId('item-drawer')).getByTestId('guides-row');
const pressed = () => within(guidesRow()).getAllByRole('button').filter(button => button.getAttribute('aria-pressed') === 'true').map(button => button.textContent);
async function openMenu(): Promise<void> {
  await click(screen.getByRole('button', { name: 'Item' }));
}
async function chooseGuides(name: string): Promise<void> {
  await openMenu();
  await click(within(guidesRow()).getByRole('button', { name }));
}
const THIRDS_390 = [
  { x1: 130, y1: 0, x2: 130, y2: 390 }, { x1: 260, y1: 0, x2: 260, y2: 390 },
  { x1: 0, y1: 130, x2: 390, y2: 130 }, { x1: 0, y1: 260, x2: 390, y2: 260 },
];

test('lv10_c16a_the_item_drawer_has_a_guides_row_off_the_first_time', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await openMenu();
  expect(within(guidesRow()).getAllByRole('button').map(button => button.textContent)).toEqual(['Off', 'Thirds', 'Grid', 'Golden ratio', 'Diagonals']);
  expect(within(screen.getByTestId('item-drawer')).getByText('GUIDES')).toBeTruthy();
  expect(pressed()).toEqual(['Off']);
  expect(lines()).toEqual([]);
});

test('lv10_c16b_thirds_draws_the_thirds_over_the_live_view', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await chooseGuides('Thirds');
  expect(pressed()).toEqual(['Thirds']);
  expect(lines()).toEqual(THIRDS_390);
  // Over the template (nobody holds for live) and over live alike.
  expect(screen.getByTestId('mode-pill').textContent).toBe('TEMPLATE');
  await click(within(screen.getByTestId('item-drawer')).getByRole('button', { name: 'Close' }));
  press(viewport(), 195, 305);
  expect(screen.getByTestId('mode-pill').textContent).toBe('LIVE');
  expect(lines()).toEqual(THIRDS_390);
  lift(viewport(), 195, 305);
});

test('lv10_c16c_the_choice_is_remembered_on_the_phone', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await chooseGuides('Thirds');
  expect(window.localStorage.getItem('psm.phone.liveGuides')).toBe('thirds');
  cleanup();
  await startPhone(env);
  expect(lines()).toEqual(THIRDS_390);
  expect(window.localStorage.getItem('psm.phone.liveGuides')).toBe('thirds');
  await openMenu();
  expect(pressed()).toEqual(['Thirds']);
});

test('lv10_c16d_a_split_shot_has_the_guides_in_each_half', async () => {
  const env = phoneEnv();
  await startPhone(env, chairSession(session => {
    session.shots[1].ratio = 2;
    session.shots[1].split = true;
  }));
  await chooseGuides('Thirds');
  // A 2:1 shot is long: its viewport is 610 × 305, turned; the lines are in the viewport's own px.
  expect(viewport().style.transform).toBe('rotate(90deg)');
  const box = { left: 0, top: 0, width: parseFloat(viewport().style.width), height: parseFloat(viewport().style.height) };
  expect(box).toEqual({ left: 0, top: 0, width: 610, height: 305 });
  expect(lines()).toEqual(overlayLines('thirds', box, true));
  expect(lines()).toHaveLength(8);
  for (const line of lines()) {
    for (const x of [line.x1, line.x2]) expect(x >= 0 && x <= 610).toBe(true);
    for (const y of [line.y1, line.y2]) expect(y >= 0 && y <= 305).toBe(true);
  }
});

test('lv10_c16e_the_guides_take_no_touch', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await chooseGuides('Thirds');
  await click(within(screen.getByTestId('item-drawer')).getByRole('button', { name: 'Close' }));
  env.mac.requests.length = 0;
  // Lock live (slide up 55 px), then tap: the tap reaches the stage and focuses.
  press(viewport(), 195, 405);
  slide(viewport(), 197, 390);
  slide(viewport(), 195, 350);
  lift(viewport(), 195, 350);
  press(viewport(), 200, 200);
  lift(viewport(), 200, 200);
  await advance(260);
  expect(env.mac.posts('/focus')).toHaveLength(1);
  expect(lines()).toEqual(THIRDS_390);
  expect(liveGuides()!.style.pointerEvents).toBe('none');
});
