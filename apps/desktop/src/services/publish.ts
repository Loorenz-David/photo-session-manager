import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { commands, type NotificationDto, type PreviewConflictDto, type PreviewDto, type PublishActivityDto, type ShownSuggestionsDto } from './bindings';
import { unwrap } from './sessions';

export type { NotificationDto, PreviewDto, PublishActivityDto };
/** What the confirmation showed for one session (08B MC-A10): its withheld count and conflict keys. */
export type ShownSuggestions = ShownSuggestionsDto;
/** One conflict as the confirmation shows it (08B MC-A10). */
export type PreviewConflict = PreviewConflictDto;

/** The code of the refusal "AI suggestions changed while you were confirming" (MC-A10): the popover rebuilds on it. */
export const SUGGESTIONS_CHANGED = 'suggestions_changed';

/** "n low-confidence AI suggestions will not be published" (08B MC-A10). */
export function withheldLine(n: number): string {
  return `${n} low-confidence AI ${n === 1 ? 'suggestion' : 'suggestions'} will not be published`;
}

/** "This item waits until you resolve n conflicts" (08B MC-A10). */
export function waitsLine(n: number): string {
  return `This item waits until you resolve ${n} ${n === 1 ? 'conflict' : 'conflicts'}`;
}

/**
 * The text the photographer confirms (07A MC-10: the exact text shown): the preview's summary, then the withheld and
 * waiting lines the popover showed for each item (R5 projection N8).
 */
export function confirmedText(preview: PreviewDto): string {
  const many = preview.items.length > 1;
  const lines = preview.items.flatMap(item => {
    const shown: string[] = [];
    if (item.n_withheld > 0) shown.push(withheldLine(item.n_withheld));
    if (item.conflicts.length > 0) shown.push(waitsLine(item.conflicts.length));
    return many ? shown.map(line => `Session ${item.session_id}: ${line}`) : shown;
  });
  return [preview.summary, ...lines].join('\n');
}
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
