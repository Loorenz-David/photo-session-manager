import { expect, it } from 'vitest';
import { INITIAL_PAGES, askCursor, canNext, pageLabel, pageStep, type PageState } from '@/features/labels/labelPages';

/** L2·U2 (`plans/L2_labels_box.md` L2·C8): the Labels box's page state (MC-L1.9). */

it('l2_c8a_the_initial_state_is_the_first_page_without_next', () => {
  expect(INITIAL_PAGES.index).toBe(0);
  expect(askCursor(INITIAL_PAGES)).toBeNull();
  expect(canNext(INITIAL_PAGES)).toBe(false);
});

it('l2_c8b_an_answer_with_a_cursor_offers_next_which_asks_it', () => {
  const answered = pageStep(INITIAL_PAGES, { type: 'answer', index: 0, nextCursor: 'c1', rows: 25 });
  expect(canNext(answered)).toBe(true);
  const next = pageStep(answered, { type: 'next' });
  expect(next.index).toBe(1);
  expect(askCursor(next)).toBe('c1');
});

it('l2_c8c_previous_and_next_reuse_the_cursors_already_seen', () => {
  const atOne: PageState = { cursors: [null, 'c1'], index: 1 };
  const answered = pageStep(atOne, { type: 'answer', index: 1, nextCursor: 'c2', rows: 25 });
  const back = pageStep(answered, { type: 'previous' });
  expect(back.index).toBe(0);
  expect(askCursor(back)).toBeNull();
  const forward = pageStep(back, { type: 'next' });
  expect(forward.index).toBe(1);
  expect(askCursor(forward)).toBe('c1');
});

it('l2_c8d_an_answer_without_a_cursor_offers_no_next', () => {
  const atOne: PageState = { cursors: [null, 'c1'], index: 1 };
  expect(canNext(pageStep(atOne, { type: 'answer', index: 1, nextCursor: null, rows: 5 }))).toBe(false);
});

it('l2_c8e_a_newer_answer_drops_the_later_cursors', () => {
  const state: PageState = { cursors: [null, 'c1', 'c2', 'c3'], index: 1 };
  expect(pageStep(state, { type: 'answer', index: 1, nextCursor: 'c2b', rows: 25 })).toEqual({
    cursors: [null, 'c1', 'c2b'],
    index: 1,
  });
});

it('l2_c8f_an_empty_later_page_goes_back_to_the_first', () => {
  const state: PageState = { cursors: [null, 'c1', 'c2'], index: 2 };
  expect(pageStep(state, { type: 'answer', index: 2, nextCursor: null, rows: 0 })).toEqual(INITIAL_PAGES);
});

it('l2_c8g_stale_answers_and_impossible_moves_change_nothing', () => {
  const state: PageState = { cursors: [null], index: 0 };
  expect(pageStep(state, { type: 'answer', index: 2, nextCursor: 'x', rows: 25 })).toBe(state);
  expect(pageStep(state, { type: 'previous' })).toBe(state);
  expect(canNext(state)).toBe(false);
  expect(pageStep(state, { type: 'next' })).toBe(state);
});

it('l2_c8h_reset_is_the_first_page', () => {
  const state: PageState = { cursors: [null, 'c1', 'c2'], index: 2 };
  expect(pageStep(state, { type: 'reset' })).toEqual(INITIAL_PAGES);
});

it('l2_c8i_page_label_from_the_filters_count', () => {
  expect(pageLabel(1, 101)).toBe('Page 2 of 5');
  expect(pageLabel(0, 0)).toBe('Page 1 of 1');
  expect(pageLabel(5, 30)).toBe('Page 6 of 6');
  expect(pageLabel(0, 25)).toBe('Page 1 of 1');
});
