import {
  commands,
  type AttachableDto_Serialize,
  type ConversationCountsDto,
  type ConversationReplyDto_Deserialize,
  type ConversationViewDto_Serialize,
  type PendingViewDto_Serialize,
  type ConversationListDto_Serialize,
  type ConversationScopeDto,
  type ConversationTabDto,
  type ConversationThreadDto_Serialize,
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
/**
 * One row per product (A30): `id` = the question it opens, `questions` = how many the product has; `status` `open` |
 * `answered` | `resolved` over its questions; `waiting` = "Pricing is waiting"; `session_id` null = "Not on this Mac".
 */
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

/**
 * C3 (09B §7): one conversation. `status` `open` | `answered` | `resolved`; `resolved_by` `asker` | `activation` | null;
 * `session_id` null = "Not on this Mac"; `can_send` false with `off_reason` when the key is refused or the store is gone.
 * In the hidden states (no key, no shop, store unmatched) every C3 call fails with code `conversations_off`
 * (`data.reason` = the state).
 */
export type ConversationView = ConversationViewDto_Serialize;
/** A message: `author.kind` `asker` (the pricer) | `me` | `other`. */
export type ConversationMessage = ConversationView['messages'][number];
/** A reply not yet confirmed: `state` `sending` | `failed` (with `failure`) | `sent`. */
export type PendingReply = PendingViewDto_Serialize;
/** A reply to send: `client_mutation_id` is made once, when send is pressed. */
export type ConversationReplyInput = ConversationReplyDto_Deserialize;
/** The photos a reply may attach; `reason` `not_on_this_mac` | `nothing_published` when there are none. */
export type ConversationAttachable = AttachableDto_Serialize;
export type AttachablePhoto = ConversationAttachable['photos'][number];

/** One conversation from the local copy (never waits on the network). */
export async function conversationGet(id: string): Promise<ConversationView> {
  return unwrap(await commands.conversationGet(id));
}

/**
 * A30 (09B §7 Thread): the product is the container. The product of conversation `id` with all its questions, oldest
 * first, each as `conversationGet` shows it; `session_id` null = "Not on this Mac".
 */
export type ConversationThread = ConversationThreadDto_Serialize;
/** The product of a conversation with all its questions, from the local copy (never waits on the network). */
export async function conversationThread(id: string): Promise<ConversationThread> {
  return unwrap(await commands.conversationThread(id));
}
/** The conversation was seen while the window was focused. */
export async function conversationMarkRead(id: string): Promise<void> {
  unwrap(await commands.conversationMarkRead(id));
}
/** Saves the reply on this Mac (it leaves within a quarter of a second); the same id again adds nothing. */
export async function conversationReply(input: ConversationReplyInput): Promise<PendingReply> {
  return unwrap(await commands.conversationReply(input));
}
/** Sends a failed reply again with its own id. */
export async function conversationRetry(clientMutationId: string): Promise<PendingReply> {
  return unwrap(await commands.conversationRetry(clientMutationId));
}
/** Removes a failed reply from this Mac. */
export async function conversationDiscard(clientMutationId: string): Promise<void> {
  unwrap(await commands.conversationDiscard(clientMutationId));
}
/** Marks the conversation answered in Pricing; code `conversation_resolved` when Pricing has already resolved it. */
export async function conversationMarkAnswered(id: string): Promise<void> {
  unwrap(await commands.conversationMarkAnswered(id));
}
/** The published photos of the conversation's product a reply may attach. */
export async function conversationAttachable(id: string): Promise<ConversationAttachable> {
  return unwrap(await commands.conversationAttachable(id));
}
