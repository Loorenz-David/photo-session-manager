import { commands, type LabelsAccessStateDto, type LabelsStatusDto_Serialize } from './bindings';
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
