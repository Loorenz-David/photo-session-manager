import { commands, type ConversationsSettingsDto_Serialize } from './bindings';
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
