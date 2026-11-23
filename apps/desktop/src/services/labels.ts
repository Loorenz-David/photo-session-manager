import {
  commands,
  type LabelFilterDto,
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
