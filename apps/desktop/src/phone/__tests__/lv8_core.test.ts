// LV8 — the core mechanisms of sort and arrange on the phone (`arrange.ts`; master_plan_live §5 PH9;
// the plan's only unit tests): input → output on the real functions, one case per rule line.
import { expect, test } from 'vitest';
import { nextPhoto, swipeAction } from '../arrange';
import { previewSubject } from '../state';
import { PHOTO_B, PHOTO_C1, PHOTO_C2, PHOTO_EXTRA, sortSession } from './helpers';

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
