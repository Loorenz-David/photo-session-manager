// LV3·U4 — the Item drawer and the toast (design README §6–§7; LV-D17): rows LV3·C24–C25. Fixture session
// as in lv3_preview. The Move drawer's rows (LV3·C23) went with the Move drawer (LV9, book 10J LV-D21 (4),
// plan point P7: dragging in the strips and swiping reach every place it offered).
import { screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, chairSession, lift, type PhoneEnv, phoneEnv, press, slide, startPhone } from './helpers';

const toast = () => screen.getByTestId('toast');
const buzzed = (env: PhoneEnv, pattern: number | number[]) => env.vibrate.mock.calls.filter(call => JSON.stringify(call[0]) === JSON.stringify(pattern)).length;

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
