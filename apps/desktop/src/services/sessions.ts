import { convertFileSrc } from '@tauri-apps/api/core';
import {
  commands,
  type CommandError,
  type SessionFilter,
  type SessionListResponse_Serialize,
  type SessionResponse_Serialize,
} from './bindings';

export type SessionList = SessionListResponse_Serialize;
export type Session = SessionResponse_Serialize;

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

/** URL the webview can load for a preview file on disk. */
export function previewUrl(path: string): string {
  return convertFileSrc(path);
}
