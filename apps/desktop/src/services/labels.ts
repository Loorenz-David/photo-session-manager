import {
  commands,
  type CheckOutcomeDto,
  type LabelCheckDto_Serialize,
  type LabelFilterDto,
  type LabelPreviewDto,
  type LabelRunResultDto_Serialize,
  type SavedLabelDto,
  type LabelPageDto_Serialize,
  type LabelsAccessStateDto,
  type LabelsOpenDto,
  type LabelsStatusDto_Serialize,
  type LabelsSummaryDto_Serialize,
  type SelectedDto,
} from './bindings';
import { unwrap } from './sessions';

/**
 * L1 (09C §7): the label markers of sessions with the labels access. `access.state` = `off` (the Pricing connection is
 * not ready; `access.reason` = its state, e.g. `no_key`), `unknown`, `allowed` or `denied`.
 */
export type LabelsStatus = LabelsStatusDto_Serialize;
/** One session with its product (`null` = not in the shop) and its marker (`null` = none known). */
export type SessionLabel = LabelsStatus['labels'][number];
/** A product's label state: `printed` (with `printed_at`, ms) or `not_printed` (with `reset`, `has_price`). */
export type LabelMarker = NonNullable<SessionLabel['marker']>;
export type LabelsAccess = LabelsStatus['access'];
/** Settings → Pricing's Labels line (`off` while the connection is not ready). */
export type LabelsAccessState = LabelsAccessStateDto;

/**
 * The markers of `session_ids`, in that order. `refresh` asks Pricing for products without a fresh answer; without it only
 * what this Mac already knows is answered (never a request). Failures: `labels_off` (`data.reason`), `labels_denied`,
 * `labels_unreachable`, … with the words in the message.
 */
export async function labelsStatus(args: { session_ids: number[]; refresh: boolean }): Promise<LabelsStatus> {
  return unwrap(await commands.labelsStatus(args.session_ids, args.refresh));
}

/** L2 (09C §5): the Labels box's sub-filter. */
export type LabelFilter = LabelFilterDto;
/** One page of the Labels box: rows in Pricing's order, the cursor of the next page, the counts, `work_from`. */
export type LabelPage = LabelPageDto_Serialize;
/** One Labels row: Pricing's fields, its marker, the matched session (`null` = none here), the cached picture. */
export type LabelRow = LabelPage['rows'][number];
export type LabelCounts = LabelPage['counts'];
export type LabelsSummary = LabelsSummaryDto_Serialize;
export type SelectedProduct = SelectedDto;

/** L2 (MC-L1.1): one page (25 rows) for the filter, the search and the page's cursor (`null` = the first page). */
export async function labelsList(args: { filter: LabelFilter; query: string | null; cursor: string | null }): Promise<LabelPage> {
  return unwrap(await commands.labelsList(args.filter, args.query, args.cursor));
}

/** L2 (MC-L1.6): the Labels box's number (`null` = none known). `refresh` asks Pricing when it is not fresh. */
export async function labelsSummary(args: { refresh: boolean }): Promise<LabelsSummary> {
  return unwrap(await commands.labelsSummary(args.refresh));
}

/** L2 (MC-L1.7): every product of the filter and search that has a SKU (Select all). */
export async function labelsSelectAll(args: { filter: LabelFilter; query: string | null }): Promise<SelectedProduct[]> {
  return unwrap(await commands.labelsSelectAll(args.filter, args.query));
}

/** L2 (MC-L1.8): the session a Labels row opens (an unmatched product is adopted from the shop first). */
export async function labelsOpen(args: { product_id: string }): Promise<LabelsOpenDto> {
  return unwrap(await commands.labelsOpen(args.product_id));
}

/** L3 (09C §6.1): what generating will do to one product. */
export type CheckOutcome = CheckOutcomeDto;
/** L3 (MC-L2.1–.4): the check dialog's content; `id` names the check for preview, generate and download. */
export type LabelCheck = LabelCheckDto_Serialize;
/** One group of the check (`template` null = the products without a template, always last). */
export type CheckGroup = LabelCheck['groups'][number];
export type CheckItem = CheckGroup['items'][number];
/** L4 (MC-L2.15): a note of a check item (`not_published` with the QR code's address, or `other`). */
export type CheckNote = CheckItem['notes'][number];
export type CheckUnresolved = LabelCheck['unresolved'][number];
export type CheckSummary = LabelCheck['summary'];
/** L3 (MC-L2.9): the done dialog's content. Fully generated ⇔ `failures` and `not_run` are both empty. */
export type LabelRunResult = LabelRunResultDto_Serialize;
export type LabelFailure = LabelRunResult['failures'][number];
/** One label file of the result (`page_count` = its labels). */
export type LabelFile = LabelRunResult['artifacts'][number];
/** A saved label file: its path (outside the data folder: shown as text, never through `previewUrl`) and whether Preview opened it. */
export type SavedLabel = SavedLabelDto;

/** L3 (MC-L2.1): checks the products (changes nothing). Failures: `labels_off`, `labels_denied`, `labels_no_products`, `labels_busy`, … */
export async function labelsCheck(args: { product_ids: string[] }): Promise<LabelCheck> {
  return unwrap(await commands.labelsCheck(args.product_ids));
}

/** L3 (MC-L2.5): the preview picture of one product of the check (`labels_rejected`, `labels_preview_failed`, …). */
export async function labelsPreview(args: { check_id: string; product_id: string }): Promise<LabelPreviewDto> {
  return unwrap(await commands.labelsPreview(args.check_id, args.product_id));
}

/** L3 (MC-L2.6–.10): generates the check's labels; Try again calls it again with the same `check_id`. */
export async function labelsGenerate(args: { check_id: string }): Promise<LabelRunResult> {
  return unwrap(await commands.labelsGenerate(args.check_id));
}

/** L3 (MC-L2.10): the result over what was generated so far (`not_found` without a generation). */
export async function labelsPartial(args: { check_id: string }): Promise<LabelRunResult> {
  return unwrap(await commands.labelsPartial(args.check_id));
}

/** L3 (MC-L2.11): saves one file in Documents › Session Manager Labels and opens it in Preview. */
export async function labelsDownload(args: { check_id: string; artifact_id: string }): Promise<SavedLabel> {
  return unwrap(await commands.labelsDownload(args.check_id, args.artifact_id));
}
