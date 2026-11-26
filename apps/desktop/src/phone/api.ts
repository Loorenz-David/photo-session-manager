/**
 * The phone's wire client (master_plan_live §5 PH3, PH4, PH6, PH7). Only relative URLs, so the same
 * code runs on `http://` now and `https://` later (card LV3); no secure-context API. The token is
 * sent as `Authorization: Bearer` on every request and as `?pair=` where a header cannot be set
 * (images, the WebSocket, the manifest). It is never logged and never shown.
 */

const TOKEN_KEY = 'psm.phone.token';
const API = '/api/phone/v1';

/** PH7: `?pair=` on `/phone/` is kept in localStorage and removed from the URL; the manifest link carries it. */
export function takeToken(): string | null {
  const url = new URL(window.location.href);
  const given = url.searchParams.get('pair');
  if (given) {
    window.localStorage.setItem(TOKEN_KEY, given);
    url.searchParams.delete('pair');
    window.history.replaceState(null, '', url.pathname + url.search + url.hash);
  }
  const token = window.localStorage.getItem(TOKEN_KEY);
  if (token) setManifest(token);
  return token;
}

/** PH7: an installed iPhone app starts from the manifest's `start_url`, which carries its token. */
function setManifest(token: string): void {
  let link = document.querySelector<HTMLLinkElement>('link[rel="manifest"]');
  if (!link) {
    link = document.createElement('link');
    link.rel = 'manifest';
    document.head.appendChild(link);
  }
  link.setAttribute('href', `/phone/manifest.webmanifest?pair=${encodeURIComponent(token)}`);
}

export function forgetToken(): void {
  window.localStorage.removeItem(TOKEN_KEY);
}

/** A request the Mac refused because the phone is not (or no longer) paired (401). */
export class Unpaired extends Error {}

/** A request the Mac answered with `{error: {code, message}}`; `code` is the wire's (PH4). */
export class ApiError extends Error {
  constructor(public status: number, public code: string) {
    super(code);
  }
}

export type Hello = { api: number; pairing: { id: number; name: string }; mac_name: string; features: string[] };
/** PH6 `Image`. `ratio` null = the photo keeps the camera's own shape. */
export type PhoneImage = { photo_id: number; revision: number; ratio: number | null };
export type PhoneShot = {
  container_id: number;
  final_no: number;
  shoot_no: number;
  name: string;
  ratio: number | null;
  split: boolean;
  zoom: number;
  example: boolean;
  photo: PhoneImage | null;
};
export type PhoneCurrent = { kind: 'shot'; container_id: number } | { kind: 'free' };
export type PhoneSession = {
  id: number;
  item: { name: string; sku: string; article_number: string | null };
  templated: boolean;
  /** `null` for an untemplated session. */
  current: PhoneCurrent | null;
  /** In shooting order. */
  shots: PhoneShot[];
  /** Main photos without a container, in Main order. */
  extras: PhoneImage[];
  condition: PhoneImage[];
  counts: { main: number; condition: number };
};
export type PhoneSessionView = { session: PhoneSession | null };
/** PH4 `POST /photos/<id>/move` targets. */
export type MoveTarget =
  | { kind: 'container'; container_id: number }
  | { kind: 'main_position'; position: number }
  | { kind: 'extra_end' }
  | { kind: 'condition'; position: number }
  | { kind: 'condition_end' };

async function request<T>(method: 'GET' | 'POST', path: string, token: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = { Authorization: `Bearer ${token}` };
  if (body !== undefined) headers['Content-Type'] = 'application/json';
  const response = await fetch(`${API}${path}`, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) });
  if (response.status === 401) throw new Unpaired('pairing_unknown');
  if (!response.ok) {
    let code = 'internal';
    try {
      const answer = (await response.json()) as { error?: { code?: string } };
      if (answer.error?.code) code = answer.error.code;
    } catch {
      // A body that is not the wire's error shape: the status alone is the answer.
    }
    throw new ApiError(response.status, code);
  }
  return (await response.json()) as T;
}

export const hello = (token: string) => request<Hello>('GET', '/hello', token);
export const session = (token: string) => request<PhoneSessionView>('GET', '/session', token);
export const capture = (token: string, session_id: number) => request<{ capture_id: string }>('POST', '/capture', token, { session_id });
export const focus = (token: string, session_id: number, x: number, y: number) =>
  request<{ focus_id: string }>('POST', '/focus', token, { session_id, x, y });
export const focusClear = (token: string, session_id: number) => request<{ focus_id: string }>('POST', '/focus/clear', token, { session_id });
/** `container_id` null = Free shot (Condition in a templated session, card LV7). */
export const setCurrent = (token: string, session_id: number, container_id: number | null) =>
  request<unknown>('POST', '/current', token, { session_id, container_id });
export const retake = (token: string, session_id: number, container_id: number) =>
  request<unknown>('POST', '/retake', token, { session_id, container_id });
export const movePhoto = (token: string, photo_id: number, session_id: number, target: MoveTarget) =>
  request<unknown>('POST', `/photos/${photo_id}/move`, token, { session_id, target });
export const deletePhoto = (token: string, photo_id: number, session_id: number) =>
  request<unknown>('POST', `/photos/${photo_id}/delete`, token, { session_id });

/** The photo's preview as the Mac shows it; `revision` only makes the browser fetch a changed one again. */
export const previewUrl = (token: string, image: PhoneImage) =>
  `${API}/photos/${image.photo_id}/preview?pair=${encodeURIComponent(token)}&r=${image.revision}`;
/** The container's template example as stored (black and white is the phone's own display filter, H3). */
export const exampleUrl = (token: string, container_id: number) => `${API}/containers/${container_id}/example?pair=${encodeURIComponent(token)}`;
/** PH5: the socket's address follows the page's scheme (`ws:` from `http:`, `wss:` from `https:`). */
export const socketUrl = (token: string) =>
  `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}${API}/ws?pair=${encodeURIComponent(token)}`;
