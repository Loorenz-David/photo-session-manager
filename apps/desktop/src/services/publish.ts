import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { commands, type NotificationDto, type PreviewDto, type PublishActivityDto } from './bindings';
import { unwrap } from './sessions';

export type { NotificationDto, PreviewDto, PublishActivityDto };
export type PublishProgressEvent = { session_id: number; item_id: number; step: string; uploaded: number; total: number };

export async function publishPreview(sessionIds: number[]): Promise<PreviewDto> {
  return unwrap(await commands.publishPreview(sessionIds));
}

export async function publishConfirm(sessionIds: number[], summary: string): Promise<number> {
  return unwrap(await commands.publishConfirm(sessionIds, summary));
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
