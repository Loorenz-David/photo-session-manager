import type { CheckGroup, CheckItem, CheckOutcome, CheckSummary, LabelRunResult } from '@/services/labels';
import type { LabelMarker } from '@/services/labels';

/** L1 (09C §4, §6, §7; master plan labels §4 Words): the words labels show. */
export const LABELS_ALLOWED = 'Labels: allowed';
export const LABELS_DENIED = "Labels: this key can't read labels. Ask a Pricing admin to add labels:read and labels:write.";
export const LABELS_UNKNOWN = "Labels: couldn't check. Press Test connection.";
/** The ⋯ hint when the key may not read labels. */
export const DENIED_HINT = "This key can't read labels. Ask a Pricing admin to add labels:read and labels:write.";
export const PUBLISH_FIRST = 'Publish first so Pricing can see it';
export const GENERATE_LABEL = 'Generate label';
export const REPRINT_LABEL = 'Reprint label';

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

/** `d MMM` of the local calendar day, with the year when it is not `now`'s year (fixed English months). */
export function day(ms: number, now: number): string {
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

// ---------- L2: the Labels box (09C §5) ----------

export const NO_PRICE = 'No price';

/** Groups the digits of a non-negative whole number in threes with a thin space (U+2009). */
function grouped(whole: number): string {
  return String(whole).replace(/\B(?=(\d{3})+(?!\d))/g, '\u2009');
}

/**
 * 09C §5 Rows (Price): the provider's minor units as kronor: the whole units grouped in threes with a thin space
 * (U+2009), a remainder as `,<two digits>`, then a no-break space (U+00A0) and `kr`; `null` → `No price`.
 */
export function formatPrice(minor: number | null | undefined): string {
  if (minor == null) return NO_PRICE;
  const whole = Math.floor(minor / 100);
  const rest = minor - whole * 100;
  const cents = rest === 0 ? '' : `,${String(rest).padStart(2, '0')}`;
  return `${grouped(whole)}${cents}\u00A0kr`;
}

/**
 * 09C §5: the line under the toolbar, `Products created since <d MMM yyyy>`, built from the text's three parts (never a
 * `Date` made from the text, which would shift by the time zone); anything but `YYYY-MM-DD` → none.
 */
export function workFromLine(text: string | null | undefined): string | null {
  const match = text?.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  if (!match) return null;
  const month = MONTHS[Number(match[2]) - 1];
  const dayOfMonth = Number(match[3]);
  if (!month || dayOfMonth < 1 || dayOfMonth > 31) return null;
  return `Products created since ${dayOfMonth} ${month} ${match[1]}`;
}

// ---------- L3: the check and done dialogs (09C §6, Pricing's LABELS.md §3.3–§3.5) ----------

export const GENERATE_PDF = 'Generate PDF';
export const REPRINT_PDF = 'Reprint PDF';
export const DOWNLOAD_PDF = 'Download PDF';
export const DONE = 'Done';
export const CLOSE = 'Close';
export const CANCEL = 'Cancel';
export const TRY_AGAIN = 'Try again';
export const NO_TEMPLATE = 'No template';
export const NOT_FOUND_IN_PRICING = 'Not found in Pricing';
export const CANT_GENERATE = "Can't generate";
export const PRINT_AT_100 = 'Print at 100 % (not Fit to page).';
export const FAILED_STAY_SELECTED = 'Failed products stay selected so you can fix and retry.';
export const OPEN_PRICING_SETTINGS = 'Open Settings → Pricing';

/** Master plan labels §6: the outcome words of a check line. */
export function outcomeWords(outcome: CheckOutcome): string {
  switch (outcome) {
    case 'mark_printed': return 'Will be marked Printed';
    case 'reprint': return 'Reprint · stays Printed';
    case 'no_price': return 'No price · stays Not printed';
    default: return CANT_GENERATE;
  }
}

/** A millimetre value: whole numbers without decimals. */
function mm(value: number | null): string {
  if (value == null) return '?';
  return Number.isInteger(value) ? String(value) : String(Number(value.toFixed(1)));
}

/** `<w> × <h> mm`. */
export function sizeText(width: number | null, height: number | null): string {
  return `${mm(width)} × ${mm(height)} mm`;
}

/**
 * 09C §6.1 (Pricing's wording, LABELS.md §3.3): the check's footer and notes. A batch: "{mark_printed} products will be
 * marked Printed" (Pricing's own plural), then the failing and reprint sentences when their numbers are not zero. One
 * product (the dialog was opened with one product id, projection r0 D11): by its outcome.
 */
export function checkSentences(summary: CheckSummary, single: CheckItem | null, now: number): string[] {
  if (single) {
    switch (single.outcome) {
      case 'mark_printed': return ['The product will be marked Printed.'];
      case 'reprint': return single.printed_at == null
        ? ['Label status stays Printed.', 'Reprinting keeps it marked Printed.']
        : ['Label status stays Printed.', `Printed ${day(single.printed_at, now)}. Reprinting keeps it marked Printed.`];
      case 'no_price': return ['Label status stays Not printed.', 'This product has no selling price. You can generate its label, but it stays marked Not printed.'];
      default: return [];
    }
  }
  const sentences = [`${summary.mark_printed} products will be marked Printed`];
  if (summary.failing > 0) sentences.push(`${summary.failing} can't be generated with the current template. They're marked below.`);
  if (summary.reprint > 0) sentences.push(`${summary.reprint} already printed. They're reprinted and stay Printed.`);
  return sentences;
}

/** Fully generated ⇔ nothing failed and nothing is left to run (M-L4). */
export function fullyGenerated(result: LabelRunResult): boolean {
  return result.failures.length === 0 && result.not_run.length === 0;
}

/**
 * 09C §6.3: the done dialog's title. One product: "Label generated" / "Label not generated"; a batch: "{rendered} of
 * {total} labels generated" when anything failed or was not generated, else "{total} labels generated".
 */
export function doneTitle(result: LabelRunResult, single: boolean): string {
  const full = fullyGenerated(result);
  if (single) return full && result.rendered > 0 ? 'Label generated' : 'Label not generated';
  return full ? `${result.total} labels generated` : `${result.rendered} of ${result.total} labels generated`;
}

/** 09C §6.3: the result lines (a line whose number is zero is not shown), each with its mark. */
export function resultLines(result: LabelRunResult): { mark: string; text: string }[] {
  const lines = [
    { n: result.marked_printed, mark: '✓', text: `${result.marked_printed} marked Printed` },
    { n: result.reprinted, mark: '↻', text: `${result.reprinted} reprinted · stay Printed` },
    { n: result.no_price, mark: '○', text: `${result.no_price} stay Not printed · No selling price` },
    { n: result.failures.length, mark: '✕', text: `${result.failures.length} failed` },
    { n: result.not_run.length, mark: '✕', text: `${result.not_run.length} not generated` },
  ];
  return lines.filter(line => line.n > 0).map(({ mark, text }) => ({ mark, text }));
}

// ---------- L4: the check's product cards (09C §14.1) ----------

export const NO_PRICE_PILL = NO_PRICE;
export const NOT_PUBLISHED_YET = 'Not published yet';
export const QR_LINK = 'QR link';

/** A red pill of a card: `Can't generate: <message>` (the message as Pricing gave it, 09C §10). */
export function cantGenerate(message: string): string {
  return `${CANT_GENERATE}: ${message}`;
}

/**
 * 09C §14.1: a group's header in two lines: the template's name (`No template` without one), then its product types
 * joined by ", " and its size (only the size when it names no product types); no second line without a template.
 */
export function groupLines(group: CheckGroup): { name: string; detail: string | null } {
  const template = group.template;
  if (!template) return { name: NO_TEMPLATE, detail: null };
  const size = sizeText(template.width_mm, template.height_mm);
  const detail = group.product_types.length > 0 ? `${group.product_types.join(', ')} · ${size}` : size;
  return { name: template.name, detail };
}

/** A file row's label count: `1 label`, `2 labels`. */
export function labelCount(n: number): string {
  return n === 1 ? '1 label' : `${n} labels`;
}
