import type { LabelMarker } from '@/services/labels';

/** L1 (09C §4, §6, §7; master plan labels §4 Words): the words labels show. */
export const LABELS_ALLOWED = 'Labels: allowed';
export const LABELS_DENIED = "Labels: this key can't read labels. Ask a Pricing admin to add labels:read and labels:write.";
export const LABELS_UNKNOWN = "Labels: couldn't check. Press Test connection.";
/** The ⋯ hint when the key may not read labels. */
export const DENIED_HINT = "This key can't read labels. Ask a Pricing admin to add labels:read and labels:write.";
export const PUBLISH_FIRST = 'Publish first so Pricing can see it';
export const GENERATE_LABEL = 'Generate label';

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

/** `d MMM` of the local calendar day, with the year when it is not `now`'s year (fixed English months). */
function day(ms: number, now: number): string {
  const at = new Date(ms);
  const text = `${at.getDate()} ${MONTHS[at.getMonth()]}`;
  return at.getFullYear() === new Date(now).getFullYear() ? text : `${text} ${at.getFullYear()}`;
}

/**
 * MC-L3.5: the marker's words, first match: no marker → none; printed → `Label printed · <d MMM>` (`Label printed` without
 * a time); a reset reason → its words (a stale label wins over "No price yet"); no price → `No price yet`; a price →
 * `Missing price tag`; price unknown → none.
 */
export function markerText(marker: LabelMarker | null | undefined, now: number): string | null {
  if (!marker) return null;
  if (marker.status === 'printed') return marker.printed_at == null ? 'Label printed' : `Label printed · ${day(marker.printed_at, now)}`;
  switch (marker.reset) {
    case 'price_changed': return 'Price changed: new label needed';
    case 'sku_changed': return 'SKU changed: new label needed';
    case 'barcode_changed': return 'Barcode changed: new label needed';
    default: break;
  }
  if (marker.has_price === false) return 'No price yet';
  if (marker.has_price === true) return 'Missing price tag';
  return null;
}
