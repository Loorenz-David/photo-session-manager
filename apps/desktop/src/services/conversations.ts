import {
  commands,
  type ConversationCountsDto,
  type ConversationListDto_Serialize,
  type ConversationScopeDto,
  type ConversationTabDto,
  type ConversationsSettingsDto_Serialize,
  type SessionBadgeDto,
} from './bindings';
import { unwrap } from './sessions';

/**
 * C1 (09B §4): Settings → Pricing as saved: the effective server address, whether a key is saved (never the key), the two
 * notification switches and the connection (`state`: `no_key`, `no_shop`, `store_unmatched`, `ready`, `refused`, `store_gone`).
 */
export type ConversationsSettings = ConversationsSettingsDto_Serialize;
export type ConversationsConnection = ConversationsSettings['connection'];
/** The fields to save; the others are kept. `server: ''` returns to the default address. */
export type ConversationsSettingsPatch = { server?: string; sound?: boolean; banner?: boolean };

export async function conversationsSettingsGet(): Promise<ConversationsSettings> {
  return unwrap(await commands.conversationsSettingsGet());
}
/** Saves the fields given; an address that is not https is refused (code `invalid_value`, naming `server`). */
export async function conversationsSettingsSave(patch: ConversationsSettingsPatch): Promise<ConversationsSettings> {
  return unwrap(await commands.conversationsSettingsSave(patch));
}
/** Saves the key in the Keychain and tests the connection at once; an empty key removes it. */
export async function conversationsKeySet(key: string): Promise<ConversationsSettings> {
  return unwrap(await commands.conversationsKeySet(key));
}
/** Test connection. Pricing unreachable or answering oddly: code `provider`, its words in the message. */
export async function conversationsTest(): Promise<ConversationsSettings> {
  return unwrap(await commands.conversationsTest());
}

/** C2 (09B §7): which conversations a list shows, all or one session's. */
export type ConversationScope = ConversationScopeDto;
/** C2 (09B §7): `open` (unresolved) or `resolved` (the last 30 days). */
export type ConversationTab = ConversationTabDto;
/**
 * C2 (09B §7): the panel's list. `state` as in Settings; `problem` = the sentence when the key is refused or the store
 * is gone; `news_elsewhere` = conversations with news outside the scope.
 */
export type ConversationList = ConversationListDto_Serialize;
/** One row: `status` `open` | `answered` | `resolved`; `waiting` = "Pricing is waiting"; `session_id` null = "Not on this Mac". */
export type ConversationListRow = ConversationList['rows'][number];
/** C2 (09B §6): conversations with news here (all, off a session) and in other sessions. */
export type ConversationCounts = ConversationCountsDto;
/** C2 (09B §6): "N open", or "Pricing is waiting" when `waiting`. */
export type ConversationSessionBadge = SessionBadgeDto;

/** The panel's rows from the local copy (never waits on the network). */
export async function conversationsList(scope: ConversationScope, tab: ConversationTab): Promise<ConversationList> {
  return unwrap(await commands.conversationsList(scope, tab));
}
/** The header count for the session on screen (`null` = all sessions). */
export async function conversationsCounts(sessionId: number | null): Promise<ConversationCounts> {
  return unwrap(await commands.conversationsCounts(sessionId));
}
/** One badge per session with an unresolved conversation. */
export async function conversationsSessionBadges(): Promise<ConversationSessionBadge[]> {
  return unwrap(await commands.conversationsSessionBadges());
}
