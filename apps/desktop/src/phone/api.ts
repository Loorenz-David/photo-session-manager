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
/**
 * PH6 `Image`. `ratio` null = the photo keeps the camera's own shape. PH6a: `preview` is null while
 * the Mac has no preview file to serve, otherwise a token that changes whenever that file does.
 */
export type PhoneImage = {
  photo_id: number;
  revision: number;
  ratio: number | null;
  preview: string | null;
  /**
   * PH6c: the photo's composition (10D MC-27), given only while the preview the Mac serves is
   * Capture One's full-frame preview; `null` (or absent) = the picture is shown as it is and cannot
   * be reframed yet.
   */
  composition?: PhoneComposition | null;
};
/**
 * PH6c: a composition as the Mac stores it, in its own spellings (`normal`/`split`; `original`,
 * `1:1`, `4:5`, `3:2`, `2:3`, `16:9`, `free`; `named`/`turned`; `none`/`horizontal`), so the values go
 * to the Mac's composition geometry and back to its save route unchanged. `rect` is in fractions of
 * the turned frame; `frame` is the full frame's size in px.
 */
export type PhoneComposition = {
  kind: string;
  shape: string;
  orientation: string;
  quarter_turns: number;
  flip: string;
  straighten: number;
  rect: { x: number; y: number; w: number; h: number };
  frame: { w: number; h: number };
  state: 'ok' | 'needs_review';
};
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
  /** PH6b: no photo yet, and a reported shutter whose photo is on its way is bound to this shot. */
  incoming: boolean;
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
  /**
   * PH6b: the reported shutters whose photo is on its way to a group: the end of Main in a session
   * without a template (`extras`), the Condition images in a templated one (`condition`).
   */
  incoming: { extras: number; condition: number };
  /** PH6c: the Mac's `default_shape` setting (what Reset frames with); absent = `1:1`. */
  default_shape?: string;
};
export type PhoneSessionView = { session: PhoneSession | null };
/** PH4 `POST /photos/<id>/move` targets. */
export type MoveTarget =
  | { kind: 'container'; container_id: number }
  | { kind: 'main_position'; position: number }
  | { kind: 'extra_end' }
  | { kind: 'condition'; position: number }
  | { kind: 'condition_end' }
  /** PH4a: put an extra at place `index` among the extras (1-based), the others keep their order. */
  | { kind: 'extra_insert'; index: number }
  /** PH4a: the same among the Condition images. */
  | { kind: 'condition_insert'; index: number };

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
/** PH4a: the swipe to Main or to Condition (and a drop into the other group); the Mac decides what that changes. */
export const classifyPhoto = (token: string, photo_id: number, session_id: number, to: 'main' | 'condition') =>
  request<unknown>('POST', `/photos/${photo_id}/classify`, token, { session_id, to });
export const deletePhoto = (token: string, photo_id: number, session_id: number) =>
  request<unknown>('POST', `/photos/${photo_id}/delete`, token, { session_id });
/** PH4b `values`: a framing in the Mac's spellings (what `PhoneComposition` carries, without its frame and state). */
export type CompositionValuesBody = Pick<PhoneComposition, 'kind' | 'shape' | 'orientation' | 'quarter_turns' | 'flip' | 'straighten' | 'rect'>;
/**
 * PH4b: the reframe editor's Done. `revision` is the photo's when the editor opened: the Mac saves
 * only if its framing is still that one (else 409 `composition_changed`), validates as for its own
 * editor (400 `composition_invalid`) and answers the new revision.
 */
export const saveComposition = (token: string, photo_id: number, session_id: number, revision: number, values: CompositionValuesBody) =>
  request<{ revision: number }>('POST', `/photos/${photo_id}/composition`, token, { session_id, revision, values });
/** PH4b: Done after Reset: the Mac's own default framing, behind the same revision check. */
export const resetComposition = (token: string, photo_id: number, session_id: number, revision: number) =>
  request<{ revision: number }>('POST', `/photos/${photo_id}/composition/reset`, token, { session_id, revision });

/**
 * The photo's preview as the Mac shows it, or `null` while the Mac has none to serve (PH6a: never
 * ask before `preview` is a string). PH7 (v1.3): `v` is the only reason to fetch a picture again
 * (the Mac changes it whenever the file it serves changes); the composition `revision` is not in
 * the URL, so a revision raised by a container's presets never refetches an unchanged picture.
 * `attempt` > 0 marks a retry after a failed load (`&try=<n>`; the Mac ignores it).
 */
export function previewUrl(token: string, image: PhoneImage, attempt = 0): string | null {
  if (image.preview === null) return null;
  const url = `${API}/photos/${image.photo_id}/preview?pair=${encodeURIComponent(token)}&v=${encodeURIComponent(image.preview)}`;
  return attempt > 0 ? `${url}&try=${attempt}` : url;
}
/** The container's template example as stored (black and white is the phone's own display filter, H3). */
export const exampleUrl = (token: string, container_id: number) => `${API}/containers/${container_id}/example?pair=${encodeURIComponent(token)}`;
/** PH5: the socket's address follows the page's scheme (`ws:` from `http:`, `wss:` from `https:`). */
export const socketUrl = (token: string) =>
  `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}${API}/ws?pair=${encodeURIComponent(token)}`;
