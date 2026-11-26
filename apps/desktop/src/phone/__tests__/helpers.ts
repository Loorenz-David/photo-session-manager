/**
 * What every phone test stands on (LV3 plan §Shared): the Mac is a fake at the two transports the
 * phone uses (`fetch` for PH4, `WebSocket` for PH5), so `api.ts` and `socket.ts` run for real and a
 * row's outcome is the exact request, the exact demand message, or what shows. Also the browser
 * pieces jsdom lacks: pointer capture, `matchMedia`, `navigator.vibrate`, Fullscreen,
 * `createImageBitmap`, the canvas drawer (a spy), and a layout for the stage, viewport and dock.
 */
import { act, cleanup, fireEvent, render } from '@testing-library/react';
import { createElement } from 'react';
import { afterEach, vi } from 'vitest';
import type { Hello, PhoneSession, PhoneSessionView } from '../api';
import { PhoneApp } from '../PhoneApp';

export type SentRequest = { method: string; path: string; authorization: string | null; body: unknown };
type Answer = { status: number; body: unknown };

/** The Mac's HTTP side: answers `hello` and `session` from its fields, every POST with `{ok: true}` unless told otherwise. */
export class FakeMac {
  requests: SentRequest[] = [];
  hello: Hello = { api: 1, pairing: { id: 1, name: 'Phone 1' }, mac_name: 'Studio-Mac', features: ['live', 'capture', 'focus', 'current_shot', 'move'] };
  view: PhoneSessionView = { session: null };
  /** Answers by `"METHOD path"`, e.g. `"POST /api/phone/v1/capture"`; a function sees the request. */
  answers = new Map<string, Answer | ((request: SentRequest) => Answer)>();
  /** Requests held back until `release()` (to observe the app while the Mac has not answered). */
  private held: (() => void)[] | null = null;

  hold(): void {
    this.held = [];
  }
  async release(): Promise<void> {
    const waiting = this.held ?? [];
    this.held = null;
    await act(async () => {
      waiting.forEach(go => go());
      await settle();
    });
  }
  answer(key: string, status: number, body: unknown): void {
    this.answers.set(key, { status, body });
  }
  /** The bodies of the POSTs to `path` (the part after `/api/phone/v1`). */
  posts(path: string): unknown[] {
    return this.requests.filter(r => r.method === 'POST' && r.path === `/api/phone/v1${path}`).map(r => r.body);
  }
  /** Every POST as `[path after /api/phone/v1, body]`, in order. */
  allPosts(): [string, unknown][] {
    return this.requests.filter(r => r.method === 'POST').map(r => [r.path.replace('/api/phone/v1', ''), r.body]);
  }
  gets(path: string): SentRequest[] {
    return this.requests.filter(r => r.method === 'GET' && r.path === `/api/phone/v1${path}`);
  }

  readonly fetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    const method = (init?.method ?? 'GET').toUpperCase();
    const path = String(input);
    const headers = new Headers(init?.headers);
    const request: SentRequest = {
      method,
      path,
      authorization: headers.get('Authorization'),
      body: typeof init?.body === 'string' ? JSON.parse(init.body) : undefined,
    };
    this.requests.push(request);
    if (this.held) await new Promise<void>(resolve => this.held!.push(resolve));
    const given = this.answers.get(`${method} ${path}`);
    const answer: Answer = typeof given === 'function' ? given(request)
      : given ?? (path.endsWith('/hello') ? { status: 200, body: this.hello }
        : path.endsWith('/session') ? { status: 200, body: this.view }
          : { status: 200, body: { ok: true } });
    return { status: answer.status, ok: answer.status >= 200 && answer.status < 300, json: async () => answer.body } as Response;
  };
}

/** One WebSocket the phone opened; the test plays the Mac's end of it. */
export class FakeSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;
  static opened: FakeSocket[] = [];
  readyState = FakeSocket.CONNECTING;
  binaryType = 'blob';
  sent: string[] = [];
  onopen: (() => void) | null = null;
  onmessage: ((event: { data: unknown }) => void) | null = null;
  onclose: ((event: { code: number }) => void) | null = null;
  onerror: (() => void) | null = null;
  constructor(public url: string) {
    FakeSocket.opened.push(this);
  }
  send(data: string): void {
    this.sent.push(data);
  }
  /** The phone closes its own socket (unmount). */
  close(): void {
    this.readyState = FakeSocket.CLOSED;
  }
  /** The demand messages the phone sent on this socket, parsed. */
  demands(): { type: string; gen: number; live: boolean; visible: boolean }[] {
    return this.sent.map(text => JSON.parse(text)).filter(m => m.type === 'demand');
  }
  // --- the Mac's end ---
  async accept(): Promise<void> {
    this.readyState = FakeSocket.OPEN;
    await act(async () => {
      this.onopen?.();
      await settle();
    });
  }
  async text(message: unknown): Promise<void> {
    await act(async () => {
      this.onmessage?.({ data: JSON.stringify(message) });
      await settle();
    });
  }
  async binary(bytes: ArrayBuffer): Promise<void> {
    await act(async () => {
      this.onmessage?.({ data: bytes });
      await settle();
    });
  }
  async drop(code = 1006): Promise<void> {
    this.readyState = FakeSocket.CLOSED;
    await act(async () => {
      this.onclose?.({ code });
      await settle();
    });
  }
}

/** Lets promise chains (fetch → json → state) run to their end without moving the clock. */
export async function settle(): Promise<void> {
  for (let i = 0; i < 20; i += 1) await Promise.resolve();
}

/** Moves the fake clock by `ms` and lets what it started settle. */
export async function advance(ms: number): Promise<void> {
  await act(async () => {
    await vi.advanceTimersByTimeAsync(ms);
    await settle();
  });
}

/** A PH5 binary frame message: the 32-byte header, then the JPEG bytes. */
export function frameBytes(frame: { seq: number; gen: number; width?: number; height?: number; capturedAtMs?: number; jpeg?: number[] }): ArrayBuffer {
  const jpeg = frame.jpeg ?? [0xff, 0xd8, 1, 2, 3, 0xff, 0xd9];
  const buffer = new ArrayBuffer(32 + jpeg.length);
  const view = new DataView(buffer);
  [0x50, 0x53, 0x4d, 0x46].forEach((byte, i) => view.setUint8(i, byte));
  view.setUint8(4, 1);
  view.setBigUint64(8, BigInt(frame.seq), true);
  view.setUint32(16, frame.gen, true);
  view.setUint16(20, frame.width ?? 1024, true);
  view.setUint16(22, frame.height ?? 680, true);
  view.setBigInt64(24, BigInt(frame.capturedAtMs ?? 1_764_150_000_000), true);
  new Uint8Array(buffer, 32).set(jpeg);
  return buffer;
}

export type PhoneEnv = {
  mac: FakeMac;
  /** Every WebSocket the phone opened, in order. */
  sockets: FakeSocket[];
  /** The newest socket. */
  socket: () => FakeSocket;
  vibrate: ReturnType<typeof vi.fn>;
  /** The canvas drawer: called once per live frame drawn. */
  drawImage: ReturnType<typeof vi.fn>;
  requestFullscreen: ReturnType<typeof vi.fn>;
  /** `document.visibilityState`, then the `visibilitychange` event. */
  setVisibility: (state: 'visible' | 'hidden') => Promise<void>;
};

type EnvOptions = {
  /** The page's address at start (default `/phone/`). */
  location?: string;
  /** A token already kept on the phone. */
  storedToken?: string;
  fullscreenEnabled?: boolean;
  fullscreenElement?: boolean;
  standalone?: boolean;
  /** The stage's size in CSS px (default 390 × 610: a 390 × 390 viewport for a 1:1 shot). */
  stage?: { width: number; height: number };
  /** The live frame the fake decoder reports (default 1024 × 680). */
  frame?: { width: number; height: number };
};

function define(target: object, name: string, value: unknown): void {
  Object.defineProperty(target, name, { configurable: true, writable: true, value });
}

/**
 * Installs the fake Mac and the missing browser pieces, with fake timers. Call first in a test;
 * everything is undone after it.
 */
export function phoneEnv(options: EnvOptions = {}): PhoneEnv {
  vi.useFakeTimers();
  vi.setSystemTime(new Date('2026-11-26T10:00:00Z'));
  window.localStorage.clear();
  if (options.storedToken) window.localStorage.setItem('psm.phone.token', options.storedToken);
  window.history.replaceState(null, '', options.location ?? '/phone/');
  document.head.querySelectorAll('link[rel="manifest"]').forEach(link => link.remove());

  const mac = new FakeMac();
  FakeSocket.opened = [];
  vi.stubGlobal('fetch', mac.fetch);
  vi.stubGlobal('WebSocket', FakeSocket);

  // Pointer capture (jsdom has none).
  define(Element.prototype, 'setPointerCapture', () => undefined);
  define(Element.prototype, 'releasePointerCapture', () => undefined);
  define(Element.prototype, 'hasPointerCapture', () => false);

  const vibrate = vi.fn(() => true);
  define(navigator, 'vibrate', vibrate);

  const standalone = options.standalone ?? false;
  define(window, 'matchMedia', (query: string) => ({
    matches: query.includes('display-mode: standalone') ? standalone : false,
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  }));

  const requestFullscreen = vi.fn(async () => undefined);
  define(document, 'fullscreenEnabled', options.fullscreenEnabled ?? false);
  define(document, 'fullscreenElement', options.fullscreenElement ? document.documentElement : null);
  define(document.documentElement, 'requestFullscreen', requestFullscreen);

  let visibility: 'visible' | 'hidden' = 'visible';
  Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => visibility });

  // The decoder and the canvas drawer.
  const frame = options.frame ?? { width: 1024, height: 680 };
  vi.stubGlobal('createImageBitmap', async () => ({ width: frame.width, height: frame.height, close: () => undefined }));
  const drawImage = vi.fn();
  define(HTMLCanvasElement.prototype, 'getContext', () => ({ drawImage }));

  // Layout: jsdom measures nothing, so the stage has a size and the viewport and dock tiles get
  // the rectangles their inline styles and order give them in a browser.
  const stage = options.stage ?? { width: 390, height: 610 };
  Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
    configurable: true,
    get(this: HTMLElement) { return this.classList.contains('ph-stage') ? stage.width : 0; },
  });
  Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
    configurable: true,
    get(this: HTMLElement) { return this.classList.contains('ph-stage') ? stage.height : 0; },
  });
  define(Element.prototype, 'getBoundingClientRect', function rect(this: HTMLElement) {
    const box = (left: number, top: number, width: number, height: number) =>
      ({ left, top, width, height, right: left + width, bottom: top + height, x: left, y: top, toJSON: () => ({}) });
    if (this.classList.contains('ph-viewport')) {
      const width = parseFloat(this.style.width) || 0;
      const height = parseFloat(this.style.height) || 0;
      const turned = this.style.transform.includes('rotate(90deg)');
      const [w, h] = turned ? [height, width] : [width, height];
      return box((stage.width - w) / 2, (stage.height - h) / 2, w, h);
    }
    if (this.classList.contains('ph-dock-strip') || this.classList.contains('ph-dock-scroller')) return box(0, 700, stage.width, 62);
    if (this.hasAttribute('data-dock-k')) {
      // 36 px tiles, 6 px gaps, 16 px side padding; the divider takes 11 px (design §3).
      const scroller = this.closest('.ph-dock-scroller') as HTMLElement | null;
      let left = 16;
      for (const sibling of Array.from(this.parentElement?.children ?? [])) {
        if (sibling === this) break;
        left += sibling.hasAttribute('data-dock-k') ? 42 : 17;
      }
      return box(left - (scroller?.scrollLeft ?? 0), 712, 36, 36);
    }
    return box(0, 0, 0, 0);
  });

  return {
    mac,
    sockets: FakeSocket.opened,
    socket: () => FakeSocket.opened[FakeSocket.opened.length - 1],
    vibrate,
    drawImage,
    requestFullscreen,
    setVisibility: async state => {
      visibility = state;
      await act(async () => {
        document.dispatchEvent(new Event('visibilitychange'));
        await settle();
      });
    },
  };
}

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

// --- pointers: one finger on a gesture surface, in client coordinates ---
export function press(target: Element, x: number, y: number): void {
  fireEvent.pointerDown(target, { pointerId: 1, clientX: x, clientY: y, button: 0 });
}
export function slide(target: Element, x: number, y: number): void {
  fireEvent.pointerMove(target, { pointerId: 1, clientX: x, clientY: y });
}
export function lift(target: Element, x: number, y: number): void {
  fireEvent.pointerUp(target, { pointerId: 1, clientX: x, clientY: y });
}

/**
 * The session of the plan's U3 fixture: shots B (final 02, "Back", filled), A (final 01, "Front",
 * empty), C (final 03, empty) in shooting order B, A, C, each 1:1; one extra (Main position 04);
 * one condition image; current = A; item "Oak Lounge Chair", SKU "BEY-123".
 */
export const A = 11;
export const B = 12;
export const C = 13;
export const PHOTO_B = 201;
export const PHOTO_EXTRA = 202;
export const PHOTO_C1 = 203;
export function chairSession(change: (session: PhoneSession) => void = () => undefined): PhoneSession {
  const shot = (container_id: number, final_no: number, shoot_no: number, name: string, photo_id: number | null) => ({
    container_id, final_no, shoot_no, name, ratio: 1, split: false, zoom: 0, example: true,
    photo: photo_id === null ? null : { photo_id, revision: 1, ratio: 1 },
  });
  const session: PhoneSession = {
    id: 7,
    item: { name: 'Oak Lounge Chair', sku: 'BEY-123', article_number: null },
    templated: true,
    current: { kind: 'shot', container_id: A },
    shots: [shot(B, 2, 1, 'Back', PHOTO_B), shot(A, 1, 2, 'Front', null), shot(C, 3, 3, 'Container 3', null)],
    extras: [{ photo_id: PHOTO_EXTRA, revision: 1, ratio: 1 }],
    condition: [{ photo_id: PHOTO_C1, revision: 1, ratio: 1 }],
    counts: { main: 2, condition: 1 },
  };
  change(session);
  return session;
}

/** The token the started phone is paired with. */
export const TOKEN = 'T';

/** Renders the app without waiting for the Mac (the first requests and the socket are under way). */
export async function mountPhone(): Promise<void> {
  await act(async () => {
    render(createElement(PhoneApp));
    await settle();
  });
}

/**
 * A paired phone, connected: the Mac answered `hello` and `session` (the given one), the socket is
 * open and the connecting overlay has left.
 */
export async function startPhone(env: PhoneEnv, session: PhoneSession | null = chairSession()): Promise<void> {
  env.mac.view = { session };
  window.localStorage.setItem('psm.phone.token', TOKEN);
  await mountPhone();
  await env.socket().accept();
  await advance(400);
}

/** The Mac changed the session: a `session` message, answered by the phone's re-read. */
export async function macChanges(env: PhoneEnv, session: PhoneSession | null): Promise<void> {
  env.mac.view = { session };
  await env.socket().text({ type: 'session', session_id: session?.id ?? null, rev: env.mac.requests.length + 1 });
}
