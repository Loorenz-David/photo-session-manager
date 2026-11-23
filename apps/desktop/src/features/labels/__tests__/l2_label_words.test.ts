import { expect, it } from 'vitest';
import { formatPrice, workFromLine } from '@/features/labels/labelWords';

/** L2·U2 (`plans/L2_labels_box.md` L2·C9): the price and the work-from line (09C §5). */

const THIN = ' ';
const NBSP = ' ';

it('l2_c9a_whole_kronor_grouped_with_a_thin_space', () => {
  expect(formatPrice(890000)).toBe(`8${THIN}900${NBSP}kr`);
  expect(formatPrice(99900)).toBe(`999${NBSP}kr`);
  expect(formatPrice(0)).toBe(`0${NBSP}kr`);
});

it('l2_c9b_large_amounts_and_a_remainder', () => {
  expect(formatPrice(1234567800)).toBe(`12${THIN}345${THIN}678${NBSP}kr`);
  expect(formatPrice(890050)).toBe(`8${THIN}900,50${NBSP}kr`);
});

it('l2_c9c_no_price', () => {
  expect(formatPrice(null)).toBe('No price');
});

it('l2_c9d_work_from_in_any_time_zone', () => {
  expect(workFromLine('2026-11-19')).toBe('Products created since 19 Nov 2026');
  expect(workFromLine('2026-01-05')).toBe('Products created since 5 Jan 2026');
  expect(workFromLine(null)).toBeNull();
  expect(workFromLine('soon')).toBeNull();
});
