import { convertFileSrc } from '@tauri-apps/api/core';
import {
  commands,
  type CommandError,
  type FilesFixedResponse,
  type PublishedChangesResponse,
  type SessionDeletedResponse,
  type SessionDeletePreviewResponse,
  type SessionFilter,
  type SessionListResponse_Serialize,
  type SessionPageResponse_Serialize,
  type SessionResponse_Serialize,
} from './bindings';

export type SessionList = SessionListResponse_Serialize;
/** H2 (MC-H2.1): one page of the Sessions page's list. */
export type SessionPage = SessionPageResponse_Serialize;
export type Session = SessionResponse_Serialize;
/** A10: why a published session is open again. */
export type PublishedChanges = PublishedChangesResponse;

/** A command failure carried to the UI as `{code, message, data}` (master plan §6.6). */
export class ServiceError extends Error {
  readonly code: string;
  readonly data: Record<string, string>;
  constructor(error: CommandError) {
    super(error.message);
    this.code = error.code;
    this.data = error.data;
  }
}

export function unwrap<T>(result: { status: 'ok'; data: T } | { status: 'error'; error: CommandError }): T {
  if (result.status === 'error') throw new ServiceError(result.error);
  return result.data;
}

export async function listSessions(args: { filter: SessionFilter; query: string | null }): Promise<SessionList> {
  return unwrap(await commands.sessionsList(args.filter, args.query));
}

/**
 * H2 (MC-H2.1–3): one page (25 rows) of the Sessions page's list. The search, the filter, the time filter
 * (`created_from`, ms) and "To review" (`only_ids`) look through every session; only the answer is cut.
 */
export async function sessionsPage(args: {
  filter: SessionFilter;
  query: string | null;
  created_from: number | null;
  only_ids: number[] | null;
  page: number;
}): Promise<SessionPage> {
  return unwrap(await commands.sessionsPage(args.filter, args.query, args.created_from, args.only_ids, args.page));
}

/** Returns the item's session; the backend returns the existing one when the item already has a session. */
export async function createSession(itemId: number): Promise<Session> {
  return unwrap(await commands.sessionCreate(itemId));
}

/** The session captures are routed to right now (restored from the last run), or null when they go to Unassigned. */
export async function activeSession(): Promise<Session | null> {
  return unwrap(await commands.sessionActive());
}

export async function activateSession(sessionId: number): Promise<void> {
  unwrap(await commands.sessionActivate(sessionId));
}

export async function deactivateSession(): Promise<void> {
  unwrap(await commands.sessionDeactivate());
}

/** Records the arranged mark and returns the next eligible session, if there is one. */
export async function arrangeNext(sessionId: number): Promise<{ next_session_id: number | null; research_queued: boolean }> {
  return unwrap(await commands.sessionArrangeNext(sessionId));
}

/** A17: what the "Delete session" confirm dialog shows. */
export type SessionDeletePreview = SessionDeletePreviewResponse;
/** A17: what a "Delete session" did. */
export type SessionDeleted = SessionDeletedResponse;

export async function sessionDeletePreview(sessionId: number): Promise<SessionDeletePreview> {
  return unwrap(await commands.sessionDeletePreview(sessionId));
}

/** A17: deletes the session in Session Manager only (files to the Trash); Shopify is not touched. */
export async function deleteSession(sessionId: number): Promise<SessionDeleted> {
  return unwrap(await commands.sessionDelete(sessionId));
}

export type FilesFixed = FilesFixedResponse;

/** A21: how many of the session's photos have their files in another session's folder. */
export async function sessionFilesMisplaced(sessionId: number): Promise<number> {
  return unwrap(await commands.sessionFilesMisplaced(sessionId)).photos;
}

/** A21: "Fix photo files location": the photos' files (with Capture One's) move into the session's folder. */
export async function fixSessionFiles(sessionId: number): Promise<FilesFixed> {
  return unwrap(await commands.sessionFilesFix(sessionId));
}

/** URL the webview can load for a preview file on disk. */
export function previewUrl(path: string): string {
  return convertFileSrc(path);
}

/**
 * A10 "Keep as published": a published session that changed since is marked published again as it is, without
 * republishing; photos that arrived after the publish are denied. One undo step in the session.
 */
export async function keepPublished(sessionId: number): Promise<void> {
  unwrap(await commands.sessionKeepPublished(sessionId));
}
