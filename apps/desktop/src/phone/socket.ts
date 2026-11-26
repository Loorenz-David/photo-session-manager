/**
 * The phone's WebSocket (master_plan_live §5 PH5, PH7): text messages in, demand out, live frames
 * accepted by generation and sequence, reconnect by the PH7 delays and by visibility.
 */
import { socketUrl } from './api';

/** PH5 server → phone text messages. */
export type CameraMessage = {
  type: 'camera';
  status: string;
  live: 'live' | 'unavailable' | 'off';
  reason: string | null;
  exposure: { shutter: string; aperture: string; iso: string } | null;
};
export type FocusMessage = {
  type: 'focus';
  focus_id: string;
  outcome: 'focused' | 'not_focused' | 'no_report' | 'superseded' | 'refused' | 'failed';
  reason: string | null;
};
export type CaptureMessage = { type: 'capture'; capture_id: string; state: 'acknowledged' | 'exposed' | 'failed' };
export type ServerMessage =
  | { type: 'hello'; viewer_id: number }
  | { type: 'session'; session_id: number | null; rev: number }
  | CameraMessage
  | FocusMessage
  | CaptureMessage
  | { type: 'focus_frames'; frames: unknown[] }
  | { type: 'demand_ack'; gen: number };

/** One live frame as the Mac sent it (PH5 binary message). */
export type Frame = { seq: number; gen: number; width: number; height: number; capturedAtMs: number; jpeg: Uint8Array };

const HEADER = 32;

/**
 * PH5: the 32-byte little-endian header (`PSMF`, version 1, flags, 2 reserved, `seq` u64, `gen` u32,
 * width u16, height u16, `captured_at_ms` i64), then the JPEG bytes. Anything else is not a frame.
 */
export function decodeFrame(buffer: ArrayBuffer): Frame | null {
  if (buffer.byteLength < HEADER) return null;
  const view = new DataView(buffer);
  const magic = String.fromCharCode(view.getUint8(0), view.getUint8(1), view.getUint8(2), view.getUint8(3));
  if (magic !== 'PSMF' || view.getUint8(4) !== 1) return null;
  return {
    seq: Number(view.getBigUint64(8, true)),
    gen: view.getUint32(16, true),
    width: view.getUint16(20, true),
    height: view.getUint16(22, true),
    capturedAtMs: Number(view.getBigInt64(24, true)),
    jpeg: new Uint8Array(buffer, HEADER),
  };
}

/** PH7: the wait before reconnect attempt `attempt` (0-based): 500, 1000, 2000, 4000, then every 4000 ms. */
export function reconnectDelay(attempt: number): number {
  return Math.min(4000, 500 * 2 ** Math.max(0, attempt));
}

export type SocketHandlers = {
  onMessage: (message: ServerMessage) => void;
  /** A frame of the current demand generation, newer than every frame accepted before it. */
  onFrame: (frame: Frame) => void;
  /** PH7: the demand generation changed (hold, lock, unlock, release, visibility, reconnect). */
  onDemand: (gen: number) => void;
  /** The socket opened (first time and after every reconnect). */
  onOpen: () => void;
  /** The Mac closed the socket because this phone's pairing was removed (4401). */
  onRevoked: () => void;
};

/** PH1: the close code of a removed pairing. */
const REVOKED = 4401;

export class PhoneSocket {
  private socket: WebSocket | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private attempt = 0;
  private stopped = false;
  private live = false;
  private visible = document.visibilityState !== 'hidden';
  /** PH7: the demand generation, in memory only; 0 = no demand sent yet (the Mac's own start value). */
  private generation = 0;
  private lastSeq = -1;
  private readonly onVisibility = () => this.visibility(document.visibilityState !== 'hidden');

  constructor(
    private readonly token: string,
    private readonly handlers: SocketHandlers,
  ) {}

  /** The current demand generation: a frame of any other generation is never accepted. */
  get gen(): number {
    return this.generation;
  }

  start(): void {
    document.addEventListener('visibilitychange', this.onVisibility);
    this.connect();
  }

  stop(): void {
    this.stopped = true;
    document.removeEventListener('visibilitychange', this.onVisibility);
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
    const socket = this.socket;
    this.socket = null;
    socket?.close();
  }

  /** PH7: showing Live changed (hold, lock, unlock, release): a new generation, sent when it differs. */
  setLive(live: boolean): void {
    if (live === this.live) return;
    this.live = live;
    this.demand();
  }

  private visibility(visible: boolean): void {
    if (visible === this.visible) return;
    this.visible = visible;
    this.demand();
    if (!visible) {
      // PH7: no attempt while the screen is off.
      if (this.timer) clearTimeout(this.timer);
      this.timer = null;
    } else if (!this.socket && !this.stopped) {
      // PH7: back on screen with a closed socket: reconnect at once.
      this.attempt = 0;
      this.connect();
    }
  }

  /** A new generation for the demand as it is now; sent if the socket is open (else on open). */
  private demand(): void {
    this.generation += 1;
    this.handlers.onDemand(this.generation);
    this.send();
  }

  private send(): void {
    if (this.socket?.readyState !== WebSocket.OPEN) return;
    // Live is never demanded for a screen that is off (MC-LV2: live and visible).
    this.socket.send(JSON.stringify({ type: 'demand', gen: this.generation, live: this.live && this.visible, visible: this.visible }));
  }

  private connect(): void {
    if (this.stopped || this.socket) return;
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
    const socket = new WebSocket(socketUrl(this.token));
    socket.binaryType = 'arraybuffer';
    this.socket = socket;
    socket.onopen = () => {
      if (this.socket !== socket) return;
      this.attempt = 0;
      // A new connection is a new viewer on the Mac: its frame numbers start over for this phone.
      this.lastSeq = -1;
      // PH7: on open the current demand goes out under a new generation. Before any demand was
      // made there is nothing to say: the Mac's viewer starts as "not showing Live".
      if (this.generation > 0) this.demand();
      this.handlers.onOpen();
    };
    socket.onmessage = event => {
      if (this.socket !== socket) return;
      if (typeof event.data === 'string') {
        let message: ServerMessage;
        try {
          message = JSON.parse(event.data) as ServerMessage;
        } catch {
          return;
        }
        this.handlers.onMessage(message);
        return;
      }
      const frame = decodeFrame(event.data as ArrayBuffer);
      // PH7 (MC-LV1): only the current generation, and only newer than the last accepted.
      if (!frame || frame.gen !== this.generation || frame.seq <= this.lastSeq) return;
      this.lastSeq = frame.seq;
      this.handlers.onFrame(frame);
    };
    socket.onclose = event => {
      if (this.socket !== socket) return;
      this.socket = null;
      if (this.stopped) return;
      if (event.code === REVOKED) {
        this.stopped = true;
        this.handlers.onRevoked();
        return;
      }
      // PH7: closed while visible → retry after 500, 1000, 2000, 4000, 4000… ms; hidden → wait for the screen.
      if (!this.visible) return;
      this.timer = setTimeout(() => {
        this.timer = null;
        this.connect();
      }, reconnectDelay(this.attempt));
      this.attempt += 1;
    };
    // An error is always followed by a close; the close does the work.
    socket.onerror = () => undefined;
  }
}
