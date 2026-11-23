import { commands, type BringBackOutcomeDto, type RemotePictureDto_Serialize, type RemoteStateDto, type RemoteViewDto_Serialize } from './bindings';
import { unwrap } from './sessions';

/** Where a session's item's pictures are (10H §3): `local`, `unknown` (an adopted product not read yet), `archived`, `shop_only`. */
export type RemoteState = RemoteStateDto;
/** One of a remote item's Shopify pictures: its 1600 px copy on this Mac once downloaded (`path`), or `gone` when Shopify no longer has it. */
export type RemotePicture = RemotePictureDto_Serialize;
/** What the workspace is given for a session's item (MC-H3.5). */
export type RemoteView = RemoteViewDto_Serialize;
export type BringBackOutcome = BringBackOutcomeDto;

/** True for the two remote kinds (their photos are on Shopify). */
export function isRemote(state: RemoteState | null | undefined): boolean {
  return state === 'archived' || state === 'shop_only';
}

/** The session's remote view (`null` for Unassigned). Never waits for a download: missing pictures arrive as `picture_ready`. */
export async function getRemote(sessionId: number): Promise<RemoteView | null> {
  return unwrap(await commands.sessionRemoteGet(sessionId));
}

/** Refresh from Shopify: re-reads the product's pictures (remote products only). */
export async function refreshRemote(sessionId: number): Promise<RemoteView> {
  return unwrap(await commands.sessionRemoteRefresh(sessionId));
}

/** Bring back: downloads the pictures at full size into the session folder; afterwards the session is ordinary. */
export async function bringBack(sessionId: number): Promise<BringBackOutcome> {
  return unwrap(await commands.sessionBringBack(sessionId));
}
