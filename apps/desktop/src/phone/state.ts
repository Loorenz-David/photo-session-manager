/**
 * The phone app's state (design README §State Management, adapted to the Mac's `PhoneSessionView`,
 * PH6). The Mac owns the session: shots, photos and the current shot are only ever what
 * `GET /session` last answered (MC-LV3: one current shot, server-side).
 */
import type { Hello, PhoneSession } from './api';
import type { CameraMessage } from './socket';

/** master_plan_live §6: the phone's texts. */
export const TEXT = {
  notPaired: 'Scan the QR code in Session Manager → Settings → Phone to connect this phone.',
  removed: 'This phone was removed in Session Manager. Scan a new QR code to connect it again.',
  cameraNotConnected: 'Camera not connected',
  liveUnavailable: 'Live unavailable on this camera',
  cameraBusy: 'Camera busy',
} as const;

/** The design's default template overlay (0–100 %, shared by live view and preview). */
export const DEFAULT_OVERLAY = 35;

export type PhoneState = {
  hello: Hello | null;
  /** `undefined` until `GET /session` first answered; `null` when no session is open on the Mac. */
  view: PhoneSession | null | undefined;
  camera: CameraMessage | null;
  /** Hold for live (released on pointer up, cancel, blur). */
  holding: boolean;
  /** Live stays on (slide up); double-tap unlocks. */
  locked: boolean;
  overlay: number;
  /** PH7: the current demand generation (the socket's). */
  gen: number;
  /** The generation of the last frame drawn: the live layer shows only when it is the current one. */
  drawnGen: number | null;
  /** The size of the last live frame drawn. */
  frame: { width: number; height: number } | null;
};

export const initialState: PhoneState = {
  hello: null,
  view: undefined,
  camera: null,
  holding: false,
  locked: false,
  overlay: DEFAULT_OVERLAY,
  gen: 0,
  drawnGen: null,
  frame: null,
};

export type Action =
  | { type: 'hello'; hello: Hello }
  | { type: 'view'; view: PhoneSession | null }
  | { type: 'camera'; camera: CameraMessage }
  | { type: 'hold'; holding: boolean }
  | { type: 'demand'; gen: number }
  | { type: 'drawn'; gen: number; width: number; height: number };

export function reduce(state: PhoneState, action: Action): PhoneState {
  switch (action.type) {
    case 'hello':
      return { ...state, hello: action.hello };
    case 'view':
      return { ...state, view: action.view };
    case 'camera':
      return { ...state, camera: action.camera };
    case 'hold':
      return state.holding === action.holding ? state : { ...state, holding: action.holding };
    case 'demand':
      return { ...state, gen: action.gen };
    case 'drawn':
      return { ...state, drawnGen: action.gen, frame: { width: action.width, height: action.height } };
  }
}

/** Both first answers are in: the connecting overlay leaves and the stage takes presses (design §1). */
export const connected = (s: PhoneState) => s.hello !== null && s.view !== undefined;
/** The person asks for Live (hold or lock): what the demand's `live` says. */
export const wantsLive = (s: PhoneState) => s.holding || s.locked;
/** PH7: the live layer shows only once a frame of the current generation is drawn (never an old picture as live). */
export const liveShown = (s: PhoneState) => wantsLive(s) && s.drawnGen !== null && s.drawnGen === s.gen;

/** 10J §Status: what stands in the hint's place while the person asks for Live and the camera cannot give it. */
export function cameraNotice(s: PhoneState): string | null {
  if (!wantsLive(s) || !s.camera) return null;
  if (s.camera.status === 'busy') return TEXT.cameraBusy;
  if (s.camera.status !== 'connected') return TEXT.cameraNotConnected;
  if (s.camera.live === 'unavailable') return TEXT.liveUnavailable;
  return null;
}
