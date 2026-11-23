import { expect, it } from 'vitest';
import { markerText } from '@/features/labels/labelWords';
import type { LabelMarker } from '@/services/labels';

/** L1·U4 (`plans/L1_labels_provider_and_marker.md` L1·C17): `markerText` (MC-L3.5), times built from local dates. */

const NOW = new Date(2026, 10, 23, 12, 0).getTime();
const THIS_YEAR = new Date(NOW).getFullYear();

function printed(at: number | null): LabelMarker {
  return { status: 'printed', printed_at: at, has_price: true, reset: null };
}

function notPrinted(reset: LabelMarker['reset'], hasPrice: boolean | null): LabelMarker {
  return { status: 'not_printed', printed_at: null, has_price: hasPrice, reset };
}

it('l1_c17a_printed_with_its_day_the_year_when_not_this_year', () => {
  expect(markerText(printed(new Date(THIS_YEAR, 10, 4, 9, 12).getTime()), NOW)).toBe('Label printed · 4 Nov');
  expect(markerText(printed(new Date(THIS_YEAR - 1, 10, 4, 9, 12).getTime()), NOW)).toBe(`Label printed · 4 Nov ${THIS_YEAR - 1}`);
  expect(markerText(printed(null), NOW)).toBe('Label printed');
});

it('l1_c17b_reset_reasons_say_a_new_label_is_needed', () => {
  expect(markerText(notPrinted('price_changed', true), NOW)).toBe('Price changed: new label needed');
  expect(markerText(notPrinted('sku_changed', true), NOW)).toBe('SKU changed: new label needed');
  expect(markerText(notPrinted('barcode_changed', true), NOW)).toBe('Barcode changed: new label needed');
});

it('l1_c17c_not_printed_by_price', () => {
  expect(markerText(notPrinted(null, true), NOW)).toBe('Missing price tag');
  expect(markerText(notPrinted(null, false), NOW)).toBe('No price yet');
  expect(markerText(notPrinted(null, null), NOW)).toBeNull();
});

it('l1_c17d_a_stale_label_wins_over_no_price_and_no_marker_is_none', () => {
  expect(markerText(notPrinted('price_changed', false), NOW)).toBe('Price changed: new label needed');
  expect(markerText(null, NOW)).toBeNull();
});
