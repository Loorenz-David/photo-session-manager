// LV3·U4 — the Move drawer, the Item drawer and the toast (design README §5–§7, §Move semantics;
// master_plan_live §5 PH4; card LV8; LV-D17): rows LV3·C23–C25. Fixture session as in lv3_preview.
import { screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, C, chairSession, lift, type PhoneEnv, PHOTO_B, PHOTO_EXTRA, phoneEnv, press, slide, startPhone } from './helpers';

const X = { b: 34 };
const dock = () => screen.getByTestId('dock');
const toast = () => screen.getByTestId('toast');
const buzzed = (env: PhoneEnv, pattern: number | number[]) => env.vibrate.mock.calls.filter(call => JSON.stringify(call[0]) === JSON.stringify(pattern)).length;
const moveDrawer = () => screen.getByTestId('move-drawer');
/** A tile as the person reads it: number tag, name, state. */
const read = (grid: HTMLElement) => Array.from(grid.children).map(tile => [
  tile.querySelector('.ph-tag')?.textContent ?? '+',
  tile.querySelector('.ph-move-name')!.textContent,
  tile.querySelector('.ph-move-state')!.textContent,
]);
const mainTile = (index: number) => screen.getByTestId('move-main').children[index] as HTMLButtonElement;
const conditionTile = (index: number) => screen.getByTestId('move-condition').children[index] as HTMLButtonElement;

/** Opens B's photo and its Move drawer. */
async function openMove(env: PhoneEnv): Promise<void> {
  await startPhone(env);
  press(dock(), X.b, 730);
  lift(dock(), X.b, 730);
  await advance(0);
  within(screen.getByTestId('preview')).getByRole('button', { name: 'Move image' }).click();
  await advance(0);
}
async function pick(tile: HTMLElement): Promise<void> {
  tile.click();
  await advance(0);
}

// ---- LV3·C23: the Move drawer ----

test('lv3_c23a_the_main_positions_in_main_order_with_what_a_tap_does', async () => {
  const env = phoneEnv();
  await openMove(env);
  expect(moveDrawer().style.transform).toBe('translateY(0%)');
  expect(read(screen.getByTestId('move-main'))).toEqual([
    ['01', 'Front', 'EMPTY'],
    ['02', 'Back', 'THIS IMAGE'],
    ['03', 'Container 3', 'EMPTY'],
    ['04', 'Extra', 'SWAP'],
    ['+', 'New extra', 'ADD AT END'],
  ]);
  // The image's own position cannot be picked.
  expect(mainTile(1).disabled).toBe(true);
  await pick(mainTile(1));
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv3_c23b_a_position_that_has_an_image_swaps', async () => {
  const env = phoneEnv();
  await openMove(env);
  await pick(mainTile(3));
  expect(env.mac.allPosts()).toEqual([[`/photos/${PHOTO_B}/move`, { session_id: 7, target: { kind: 'main_position', position: 4 } }]]);
  expect(buzzed(env, [30, 40, 60])).toBe(1);
  expect(toast().textContent).toBe('Swapped with 04');
  expect(toast().style.opacity).toBe('1');
  expect(moveDrawer().style.transform).toBe('translateY(105%)');
});

test('lv3_c23b_the_preview_stays_open_on_the_image_in_its_new_place', async () => {
  const env = phoneEnv();
  await openMove(env);
  // The Mac swapped them: B's photo is now the extra, the old extra is in B.
  env.mac.view = { session: chairSession(s => {
    s.shots[0].photo = { photo_id: PHOTO_EXTRA, revision: 1, ratio: 1, preview: 'k0' };
    s.extras = [{ photo_id: PHOTO_B, revision: 1, ratio: 1, preview: 'k0' }];
  }) };
  await pick(mainTile(3));
  const preview = within(screen.getByTestId('preview'));
  expect(preview.getByText('IMAGE 04 · EXTRA')).toBeTruthy();
  expect(screen.getByTestId('preview-photo').getAttribute('src')).toBe(`/api/phone/v1/photos/${PHOTO_B}/preview?pair=T&v=k0`);
  // An extra has no Retake (LV8, book 10J LV-D20 (4): the Delete button it used to show is gone).
  expect(preview.queryByRole('button', { name: 'Retake' })).toBeNull();
});

test('lv3_c23c_an_empty_template_shot_takes_the_image', async () => {
  const env = phoneEnv();
  await openMove(env);
  await pick(mainTile(2));
  expect(env.mac.allPosts()).toEqual([[`/photos/${PHOTO_B}/move`, { session_id: 7, target: { kind: 'container', container_id: C } }]]);
  expect(toast().textContent).toBe('Moved to shot 03');
});

test('lv3_c23d_new_extra_adds_it_at_the_end_of_main', async () => {
  const env = phoneEnv();
  await openMove(env);
  await pick(mainTile(4));
  expect(env.mac.allPosts()).toEqual([[`/photos/${PHOTO_B}/move`, { session_id: 7, target: { kind: 'extra_end' } }]]);
  expect(toast().textContent).toBe('Added as extra image');
});

test('lv3_c23e_a_condition_position_swaps_with_the_image_there', async () => {
  const env = phoneEnv();
  await openMove(env);
  const row = screen.getByRole('button', { name: /Condition images/ });
  expect(row.getAttribute('aria-expanded')).toBe('false');
  expect(row.textContent).toContain('1 IMAGE');
  row.click();
  await advance(0);
  expect(row.getAttribute('aria-expanded')).toBe('true');
  expect(read(screen.getByTestId('move-condition'))).toEqual([['C1', 'Condition 01', 'SWAP'], ['+', 'Add', 'ADD AT END']]);
  await pick(conditionTile(0));
  expect(env.mac.allPosts()).toEqual([[`/photos/${PHOTO_B}/move`, { session_id: 7, target: { kind: 'condition', position: 1 } }]]);
  expect(toast().textContent).toBe('Swapped with C1');
});

test('lv3_c23f_add_sends_it_to_the_end_of_the_condition_images', async () => {
  const env = phoneEnv();
  await openMove(env);
  screen.getByRole('button', { name: /Condition images/ }).click();
  await advance(0);
  await pick(conditionTile(1));
  expect(env.mac.allPosts()).toEqual([[`/photos/${PHOTO_B}/move`, { session_id: 7, target: { kind: 'condition_end' } }]]);
  expect(toast().textContent).toBe('Sent to condition images');
});

// ---- LV3·C24: the Item drawer ----

const rows = () => Array.from(screen.getByTestId('item-drawer').querySelectorAll('.ph-row')).map(row => [row.querySelector('dt')!.textContent, row.querySelector('dd')!.textContent]);
async function openMenu(): Promise<void> {
  screen.getByRole('button', { name: 'Item' }).click();
  await advance(0);
}

test('lv3_c24a_the_menu_button_opens_the_items_information', async () => {
  const env = phoneEnv();
  await startPhone(env);
  expect(screen.getByTestId('item-drawer').style.transform).toBe('translateY(105%)');
  await openMenu();
  expect(buzzed(env, 12)).toBe(1);
  const drawer = within(screen.getByTestId('item-drawer'));
  expect(screen.getByTestId('item-drawer').style.transform).toBe('translateY(0%)');
  expect(drawer.getByText('Oak Lounge Chair')).toBeTruthy();
  expect(drawer.getByText('CONNECTED · STUDIO-MAC')).toBeTruthy();
  // No "Article number" row: the item has no barcode. (The fixture's two Main photos, B's and the
  // extra, are the Mac's `counts.main`.)
  expect(rows()).toEqual([
    ['SKU', 'BEY-123'],
    ['Current shot', '01/03 · 1:1'],
    ['Angle', 'Front'],
    ['Images', '2 main · 1 condition'],
  ]);
});

test('lv3_c24b_an_item_with_a_barcode_shows_its_article_number_after_the_sku', async () => {
  const env = phoneEnv();
  await startPhone(env, chairSession(s => {
    s.item.article_number = 'A-77';
  }));
  await openMenu();
  expect(rows().slice(0, 3)).toEqual([['SKU', 'BEY-123'], ['Article number', 'A-77'], ['Current shot', '01/03 · 1:1']]);
});

test('lv3_c24c_in_free_shot_the_current_shot_says_so_and_there_is_no_angle', async () => {
  const env = phoneEnv();
  await startPhone(env, chairSession(s => {
    s.current = { kind: 'free' };
  }));
  await openMenu();
  expect(rows().slice(1, 3)).toEqual([['Current shot', 'FREE SHOT · 1:1'], ['Angle', '']]);
});

test('lv3_c24d_fill_item_properties_opens_an_empty_page_and_back_returns_to_the_drawer', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await openMenu();
  within(screen.getByTestId('item-drawer')).getByRole('button', { name: 'Fill item properties' }).click();
  await advance(0);
  const page = screen.getByTestId('properties-page');
  expect(screen.getByTestId('properties-container').childElementCount).toBe(0);
  within(page).getByRole('button', { name: 'Back' }).click();
  await advance(0);
  expect(screen.queryByTestId('properties-page')).toBeNull();
  expect(screen.getByTestId('item-drawer').style.transform).toBe('translateY(0%)');
  expect(env.mac.allPosts()).toEqual([]);
});

// ---- LV3·C25: the toast ----

test('lv3_c25a_a_toast_hides_after_1_4_s_and_locking_never_shows_one', async () => {
  const env = phoneEnv();
  await startPhone(env);
  // Lock, then unlock by a double tap: no toast either time.
  const viewport = screen.getByTestId('viewport');
  press(viewport, 195, 405);
  slide(viewport, 195, 390);
  slide(viewport, 195, 295);
  lift(viewport, 195, 295);
  expect(screen.getByTestId('mode-pill').textContent).toBe('LIVE · LOCKED');
  expect([toast().textContent, toast().style.opacity]).toEqual(['', '0']);
  for (const x of [195, 205]) {
    press(viewport, x, 305);
    lift(viewport, x, 305);
  }
  await advance(600);
  expect(screen.getByTestId('mode-pill').textContent).toBe('TEMPLATE');
  expect([toast().textContent, toast().style.opacity]).toEqual(['', '0']);

  // Any toast: here the Mac says the session changed.
  env.mac.answer('POST /api/phone/v1/current', 409, { error: { code: 'session_changed', message: 'session_changed' } });
  screen.getByTestId('next').click();
  await advance(0);
  expect([toast().textContent, toast().style.opacity]).toEqual(['Session changed on the Mac', '1']);
  await advance(1399);
  expect(toast().style.opacity).toBe('1');
  await advance(1);
  expect(toast().style.opacity).toBe('0');
});
