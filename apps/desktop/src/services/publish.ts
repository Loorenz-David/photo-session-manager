import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { commands, type NotificationDto, type PreviewDto, type PublishActivityDto, type ShownSuggestionsDto } from './bindings';
import { unwrap } from './sessions';

export type { NotificationDto, PreviewDto, PublishActivityDto };
/** What the confirmation showed for one session (08B MC-A10): its withheld count and conflict keys. */
export type ShownSuggestions = ShownSuggestionsDto;
export type PublishProgressEvent = { session_id: number; item_id: number; step: string; uploaded: number; total: number };

export async function publishPreview(sessionIds: number[]): Promise<PreviewDto> {
  return unwrap(await commands.publishPreview(sessionIds));
}

/** What `preview` showed, per session: passed back to the confirmation, which refuses when it changed (MC-A10). */
export function shownOf(preview: PreviewDto): ShownSuggestions[] {
  return preview.items.map(item => ({ session_id: item.session_id, n_withheld: item.n_withheld, conflict_keys: item.conflicts.map(c => c.key) }));
}

export async function publishConfirm(sessionIds: number[], summary: string, shown: ShownSuggestions[]): Promise<number> {
  return unwrap(await commands.publishConfirm(sessionIds, summary, shown));
}

export async function publishRetry(itemId: number): Promise<void> {
  unwrap(await commands.publishRetry(itemId));
}

/** P64: the latest publish item's step in words, elapsed time and last error (null when never published). */
export async function publishActivity(sessionId: number): Promise<PublishActivityDto | null> {
  return unwrap(await commands.publishActivity(sessionId));
}

/** Changelog 44: stops the publish item at its next safe point. */
export async function publishCancel(itemId: number): Promise<void> {
  unwrap(await commands.publishCancel(itemId));
}

export async function notificationsList(sessionId: number | null = null): Promise<NotificationDto[]> {
  return unwrap(await commands.notificationsList(sessionId));
}

export async function onPublishProgress(handler: (event: PublishProgressEvent) => void): Promise<UnlistenFn> {
  return listen<PublishProgressEvent>('publish_progress', event => handler(event.payload));
}
