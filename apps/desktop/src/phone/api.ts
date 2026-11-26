/**
 * The phone's wire client (master_plan_live §5 PH4, PH7). Only relative URLs; no secure-context API.
 * LV2 placeholder: the token and the two first calls; LV3 builds the Shoot Companion on it.
 */

const TOKEN_KEY = 'psm.phone.token';

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
  const manifest = document.getElementById('phone-manifest');
  if (token && manifest) manifest.setAttribute('href', `/phone/manifest.webmanifest?pair=${encodeURIComponent(token)}`);
  return token;
}

export function forgetToken(): void {
  window.localStorage.removeItem(TOKEN_KEY);
}

/** A request the Mac refused because the phone is not (or no longer) paired. */
export class Unpaired extends Error {}

async function getJson<T>(path: string, token: string): Promise<T> {
  const response = await fetch(path, { headers: { Authorization: `Bearer ${token}` } });
  if (response.status === 401) throw new Unpaired('pairing_unknown');
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  return (await response.json()) as T;
}

export type Hello = { api: number; pairing: { id: number; name: string }; mac_name: string; features: string[] };
export type PhoneSessionView = { session: null | { id: number; item: { name: string; sku: string; article_number: string | null } } };

export function hello(token: string): Promise<Hello> {
  return getJson<Hello>('/api/phone/v1/hello', token);
}

export function session(token: string): Promise<PhoneSessionView> {
  return getJson<PhoneSessionView>('/api/phone/v1/session', token);
}
