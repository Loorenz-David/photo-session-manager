// LV8 — the core mechanisms of sort and arrange on the phone (`arrange.ts`; master_plan_live §5 PH9;
// the plan's only unit tests): input → output on the real functions, one case per rule line.
import { expect, test } from 'vitest';
import type { PhoneSession } from '../api';
import { applyDrop, type Drop, dropTarget, type Home, nextPhoto, swipeAction } from '../arrange';
import { previewSubject } from '../state';
import { A, B, C, dragSession, PHOTO_B, PHOTO_C1, PHOTO_C2, PHOTO_EXTRA, sortSession } from './helpers';

// ---- swipeAction (PH9.1's table) ----

test('lv8_core_swipeAction_follows_the_table_of_subject_and_direction', () => {
  const view = sortSession();
  const shot = previewSubject(view, PHOTO_B)!;
  const extra = previewSubject(view, PHOTO_EXTRA)!;
  const condition = previewSubject(view, PHOTO_C1)!;
  expect([shot.kind, extra.kind, condition.kind]).toEqual(['shot', 'extra', 'condition']);
  expect([swipeAction(shot, 'left'), swipeAction(shot, 'right'), swipeAction(shot, 'up')]).toEqual(['retake', 'main_confirm', 'condition']);
  expect([swipeAction(extra, 'left'), swipeAction(extra, 'right'), swipeAction(extra, 'up')]).toEqual(['delete', 'none', 'condition']);
  expect([swipeAction(condition, 'left'), swipeAction(condition, 'right'), swipeAction(condition, 'up')]).toEqual(['delete', 'main', 'none']);
});

test('lv8_core_swipeAction_an_untemplated_main_photo_is_an_extra', () => {
  const view = sortSession(s => {
    s.templated = false;
    s.current = null;
    s.shots = [];
  });
  const main = previewSubject(view, PHOTO_EXTRA)!;
  expect([swipeAction(main, 'left'), swipeAction(main, 'right'), swipeAction(main, 'up')]).toEqual(['delete', 'none', 'condition']);
});

// ---- nextPhoto (plan point P4) ----

test('lv8_core_nextPhoto_goes_through_shots_extras_and_condition_and_ends_after_the_last', () => {
  // Shots in shooting order B (201), A (empty), C (empty); extra 202; condition 203, 207.
  const view = sortSession();
  expect(nextPhoto(view, PHOTO_B)).toBe(PHOTO_EXTRA);
  expect(nextPhoto(view, PHOTO_EXTRA)).toBe(PHOTO_C1);
  expect(nextPhoto(view, PHOTO_C1)).toBe(PHOTO_C2);
  expect(nextPhoto(view, PHOTO_C2)).toBeNull();
});

test('lv8_core_nextPhoto_skips_empty_shots_and_follows_shooting_order', () => {
  // A third shot's photo comes after B's although an empty shot lies between them.
  const view = sortSession(s => {
    s.shots[2].photo = { photo_id: 204, revision: 1, ratio: 1, preview: 'k0' };
  });
  expect(nextPhoto(view, PHOTO_B)).toBe(204);
  expect(nextPhoto(view, 204)).toBe(PHOTO_EXTRA);
});

test('lv8_core_nextPhoto_of_a_photo_not_in_the_session_is_none', () => {
  expect(nextPhoto(sortSession(), 999)).toBeNull();
});

// ---- dropTarget (PH9.3's table) ----

// Fixture D's homes: B 44 · A 108 · C 172 (empty) · 202 236 · 205 300 · 206 364 · Free 428 · 203 511 · 207 575 · 208 639.
const home = (key: string, kind: Home['kind'], centre: number, container_id: number | null = null): Home => ({ key, kind, centre, container_id });
const HOMES: Home[] = [
  home('s12', 'shot', 44, B), home('s11', 'shot', 108, A), home('s13', 'shot', 172, C),
  home('p202', 'extra', 236), home('p205', 'extra', 300), home('p206', 'extra', 364), home('free', 'free', 428),
  home('p203', 'condition', 511), home('p207', 'condition', 575), home('p208', 'condition', 639),
];
const target = (source: string, x: number, inBand = true) => dropTarget(HOMES, source, x, inBand);

test('lv8_core_dropTarget_a_template_shots_photo', () => {
  expect(target('s12', 108)).toEqual({ kind: 'swap', container_id: A });
  expect(target('s12', 172)).toEqual({ kind: 'swap', container_id: C });
  expect(target('s12', 300)).toEqual({ kind: 'main_confirm' });
  expect(target('s12', 575)).toEqual({ kind: 'to_condition' });
  // Its own tile, the Free shot.
  expect(target('s12', 44)).toBeNull();
  expect(target('s12', 428)).toBeNull();
});

test('lv8_core_dropTarget_an_extra', () => {
  expect(target('p202', 44)).toEqual({ kind: 'swap', container_id: B });
  expect(target('p202', 172)).toEqual({ kind: 'swap', container_id: C });
  // Its place after the move = 1 + the other extras left of the finger.
  expect(target('p202', 330)).toEqual({ kind: 'insert', group: 'extra', index: 2 });
  expect(target('p202', 380)).toEqual({ kind: 'insert', group: 'extra', index: 3 });
  expect(target('p206', 210)).toEqual({ kind: 'insert', group: 'extra', index: 1 });
  expect(target('p202', 575)).toEqual({ kind: 'to_condition' });
  // Its own place (over its own tile, or beside it with no other extra passed); the Free shot.
  expect(target('p202', 236)).toBeNull();
  expect(target('p205', 310)).toBeNull();
  expect(target('p205', 290)).toBeNull();
  expect(target('p202', 428)).toBeNull();
});

test('lv8_core_dropTarget_a_condition_photo', () => {
  expect(target('p203', 108)).toEqual({ kind: 'swap', container_id: A });
  expect(target('p203', 300)).toEqual({ kind: 'to_main' });
  expect(target('p208', 540)).toEqual({ kind: 'insert', group: 'condition', index: 2 });
  expect(target('p203', 660)).toEqual({ kind: 'insert', group: 'condition', index: 3 });
  expect(target('p207', 575)).toBeNull();
  expect(target('p203', 428)).toBeNull();
});

test('lv8_core_dropTarget_spans_end_half_a_pitch_from_the_groups_end_tiles', () => {
  // Extras: [236 − 32, 364 + 32) = [204, 396); Condition: [479, 671); a shot: within 32 px of its centre.
  expect(target('p203', 204)).toEqual({ kind: 'to_main' });
  expect(target('p203', 395)).toEqual({ kind: 'to_main' });
  expect(target('p203', 396)).toBeNull();
  expect(target('p202', 478)).toBeNull();
  expect(target('p202', 479)).toEqual({ kind: 'to_condition' });
  expect(target('p202', 671)).toBeNull();
  expect(target('p202', 139)).toEqual({ kind: 'swap', container_id: A });
  expect(target('p202', 140)).toBeNull();
  expect(target('p202', 141)).toEqual({ kind: 'swap', container_id: C });
});

test('lv8_core_dropTarget_outside_the_band_is_nothing', () => {
  expect(target('s12', 108, false)).toBeNull();
  expect(target('p202', 380, false)).toBeNull();
  expect(target('p203', 300, false)).toBeNull();
});

test('lv8_core_dropTarget_never_from_a_tile_without_a_photo_to_carry', () => {
  expect(target('free', 300)).toBeNull();
  expect(target('p999', 300)).toBeNull();
});

test('lv8_core_dropTarget_untemplated_the_main_list_is_the_extras', () => {
  const homes = [home('p1', 'extra', 44), home('p2', 'extra', 108), home('p3', 'extra', 172), home('free', 'free', 236), home('p9', 'condition', 319)];
  expect(dropTarget(homes, 'p3', 30, true)).toEqual({ kind: 'insert', group: 'extra', index: 1 });
  expect(dropTarget(homes, 'p1', 319, true)).toEqual({ kind: 'to_condition' });
  expect(dropTarget(homes, 'p9', 108, true)).toEqual({ kind: 'to_main' });
  // A group with one photo has no other place in it.
  expect(dropTarget(homes, 'p9', 330, true)).toBeNull();
});

// ---- applyDrop (PH9.5: what the strips show while the Mac answers) ----

/** The arrangement as text: each shot's photo (or -) in shooting order | the extras | the Condition images. */
const arrangement = (view: PhoneSession) => [
  view.shots.map(shot => shot.photo?.photo_id ?? '-').join(' '), view.extras.map(image => image.photo_id).join(' '), view.condition.map(image => image.photo_id).join(' '),
].join(' | ');
const dropped = (photoId: number, drop: Drop) => arrangement(applyDrop(dragSession(), photoId, drop));

test('lv8_core_applyDrop_starts_from_fixture_d', () => {
  expect(arrangement(dragSession())).toBe('201 204 - | 202 205 206 | 203 207 208');
});

test('lv8_core_applyDrop_swap_between_template_shots_exchanges_or_fills', () => {
  expect(dropped(201, { kind: 'swap', container_id: A })).toBe('204 201 - | 202 205 206 | 203 207 208');
  expect(dropped(201, { kind: 'swap', container_id: C })).toBe('- 204 201 | 202 205 206 | 203 207 208');
});

test('lv8_core_applyDrop_swap_from_outside_the_template_takes_the_shots_photo_into_its_own_place', () => {
  expect(dropped(205, { kind: 'swap', container_id: B })).toBe('205 204 - | 202 201 206 | 203 207 208');
  expect(dropped(205, { kind: 'swap', container_id: C })).toBe('201 204 205 | 202 206 | 203 207 208');
  expect(dropped(207, { kind: 'swap', container_id: A })).toBe('201 207 - | 202 205 206 | 203 204 208');
  expect(dropped(207, { kind: 'swap', container_id: C })).toBe('201 204 207 | 202 205 206 | 203 208');
});

test('lv8_core_applyDrop_insert_puts_the_photo_at_its_place_and_keeps_the_others_in_order', () => {
  expect(dropped(202, { kind: 'insert', group: 'extra', index: 3 })).toBe('201 204 - | 205 206 202 | 203 207 208');
  expect(dropped(206, { kind: 'insert', group: 'extra', index: 1 })).toBe('201 204 - | 206 202 205 | 203 207 208');
  expect(dropped(208, { kind: 'insert', group: 'condition', index: 2 })).toBe('201 204 - | 202 205 206 | 203 208 207');
});

test('lv8_core_applyDrop_to_the_other_group_appends_and_leaves_a_template_shot_empty', () => {
  expect(dropped(202, { kind: 'to_condition' })).toBe('201 204 - | 205 206 | 203 207 208 202');
  expect(dropped(201, { kind: 'to_condition' })).toBe('- 204 - | 202 205 206 | 203 207 208 201');
  expect(dropped(203, { kind: 'to_main' })).toBe('201 204 - | 202 205 206 203 | 207 208');
  expect(dropped(201, { kind: 'to_main' })).toBe('- 204 - | 202 205 206 201 | 203 207 208');
  expect(dropped(201, { kind: 'main_confirm' })).toBe('- 204 - | 202 205 206 201 | 203 207 208');
});

test('lv8_core_applyDrop_leaves_the_current_shot_and_the_given_view_alone', () => {
  const view = dragSession(s => {
    s.current = { kind: 'shot', container_id: C };
  });
  const shown = applyDrop(view, 201, { kind: 'swap', container_id: C });
  expect(shown.current).toEqual({ kind: 'shot', container_id: C });
  expect(arrangement(view)).toBe('201 204 - | 202 205 206 | 203 207 208');
  // A photo that is not in the view changes nothing.
  expect(applyDrop(view, 999, { kind: 'to_main' })).toBe(view);
});
