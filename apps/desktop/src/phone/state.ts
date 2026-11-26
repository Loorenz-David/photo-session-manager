/**
 * The phone app's state (design README §State Management, adapted to the Mac's `PhoneSessionView`,
 * PH6). The Mac owns the session: shots, photos and the current shot are only ever what
 * `GET /session` last answered (MC-LV3: one current shot, server-side).
 */
import type { Hello, MoveTarget, PhoneImage, PhoneSession, PhoneShot } from './api';
import { ratioLabel } from './geometry';
import type { CameraMessage } from './socket';

/** master_plan_live §6: the phone's texts. */
export const TEXT = {
  notPaired: 'Scan the QR code in Session Manager → Settings → Phone to connect this phone.',
  /** LV-D19 (2): a removed phone's installed app stays unpaired; it is deleted and added again. For the 401 and the socket's 4401. */
  removed: 'This phone was removed in Session Manager. Delete this app from the home screen, then scan the new QR code in Session Manager → Settings → Phone and add it again.',
  cameraNotConnected: 'Camera not connected',
  liveUnavailable: 'Live unavailable on this camera',
  cameraBusy: 'Camera busy',
  couldNotFocus: 'Could not focus there',
  focusAreaNotMovable: "Set the camera's Focus Area to a movable spot (e.g. Flexible Spot) to focus from the phone",
  notTaken: 'Not taken. Try again.',
  sessionChanged: 'Session changed on the Mac',
  conditionAdded: 'Condition image added',
  noSession: 'Open a session in Session Manager to start shooting.',
  failed: 'That did not work. Try again.',
} as const;

/** The design's default template overlay (0–100 %, shared by live view and preview). */
export const DEFAULT_OVERLAY = 35;
/** A shot that keeps the camera's own shape uses the live frame's; before any frame, the sensor's 3:2. */
const SENSOR_ASPECT = 1.5;
/** The Free shot's viewport (the design's session ratio). */
const FREE_RATIO = 1;

/** The focus marker: where the person tapped, as 0…1 of the image (so it turns with a rotated viewport). */
export type FocusMark = { x: number; y: number; id: string | null; confirmed: boolean };

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
  focus: FocusMark | null;
  /** `n` counts toasts, so the same text twice shows twice. */
  toast: { text: string; n: number } | null;
  /** Counts captures: each one flashes. */
  flash: number;
  /** The photo whose preview is open (by id, so the preview follows the image when the Mac moves it). */
  preview: number | null;
  /** The Item drawer, the Move drawer (over a preview) and the item properties page. */
  menuOpen: boolean;
  moveOpen: boolean;
  propsOpen: boolean;
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
  focus: null,
  toast: null,
  flash: 0,
  preview: null,
  menuOpen: false,
  moveOpen: false,
  propsOpen: false,
};

export type Action =
  | { type: 'hello'; hello: Hello }
  | { type: 'view'; view: PhoneSession | null }
  | { type: 'camera'; camera: CameraMessage }
  | { type: 'hold'; holding: boolean }
  | { type: 'lock'; locked: boolean }
  | { type: 'overlay'; overlay: number }
  | { type: 'demand'; gen: number }
  | { type: 'drawn'; gen: number; width: number; height: number }
  | { type: 'focus'; focus: FocusMark | null }
  | { type: 'focus-id'; id: string }
  | { type: 'focus-confirmed'; id: string }
  | { type: 'focus-lost'; id: string }
  | { type: 'toast'; text: string }
  | { type: 'flash' }
  | { type: 'preview'; photo_id: number | null }
  | { type: 'menu'; open: boolean }
  | { type: 'move'; open: boolean }
  | { type: 'props'; open: boolean };

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
    case 'lock':
      return { ...state, locked: action.locked, holding: false };
    case 'overlay':
      return { ...state, overlay: action.overlay };
    case 'demand':
      return { ...state, gen: action.gen };
    case 'drawn':
      return { ...state, drawnGen: action.gen, frame: { width: action.width, height: action.height } };
    case 'focus':
      return { ...state, focus: action.focus };
    case 'focus-id':
      return state.focus && state.focus.id === null ? { ...state, focus: { ...state.focus, id: action.id } } : state;
    case 'focus-confirmed':
      return state.focus?.id === action.id ? { ...state, focus: { ...state.focus, confirmed: true } } : state;
    case 'focus-lost':
      return state.focus?.id === action.id ? { ...state, focus: null } : state;
    case 'toast':
      return { ...state, toast: { text: action.text, n: (state.toast?.n ?? 0) + 1 } };
    case 'flash':
      return { ...state, flash: state.flash + 1 };
    case 'preview':
      return { ...state, preview: action.photo_id, moveOpen: false };
    case 'menu':
      return { ...state, menuOpen: action.open };
    case 'move':
      return { ...state, moveOpen: action.open };
    case 'props':
      return { ...state, propsOpen: action.open };
  }
}

export const pad = (n: number) => String(n).padStart(2, '0');

/** Both first answers are in: the connecting overlay leaves and the stage takes presses (design §1). */
export const connected = (s: PhoneState) => s.hello !== null && s.view !== undefined;

/** The template shot the Mac says is current, if it is one. */
export function currentShot(view: PhoneSession | null | undefined): PhoneShot | null {
  if (!view || view.current?.kind !== 'shot') return null;
  const id = view.current.container_id;
  return view.shots.find(shot => shot.container_id === id) ?? null;
}

/** Free shot: a session is open and no template shot is current (after the last shot, or no template at all). */
export const inFreeShot = (view: PhoneSession | null | undefined) => !!view && currentShot(view) === null;

/**
 * The person sees Live: hold, lock, or Free shot (there is no template to show, so live is the
 * picture; the design's "FREE SHOT · LIVE"). This is the demand's `live` (MC-LV2).
 */
export const wantsLive = (s: PhoneState) => s.holding || s.locked || inFreeShot(s.view);
/** PH7: the live layer shows only once a frame of the current generation is drawn (never an old picture as live). */
export const liveShown = (s: PhoneState) => wantsLive(s) && s.drawnGen !== null && s.drawnGen === s.gen;

/** A ratio the Mac leaves open (`null`: the camera's own shape) is the live frame's. */
export function frameAspect(s: PhoneState): number {
  return s.frame && s.frame.height > 0 ? s.frame.width / s.frame.height : SENSOR_ASPECT;
}

/** The main viewport's aspect ratio. */
export function stageRatio(s: PhoneState): number {
  const shot = currentShot(s.view);
  if (!shot) return FREE_RATIO;
  return shot.ratio ?? frameAspect(s);
}

/** 10J §Status: what stands in the hint's place while the person asks for Live and the camera cannot give it. */
export function cameraNotice(s: PhoneState): string | null {
  if (!wantsLive(s) || !s.camera) return null;
  if (s.camera.status === 'busy') return TEXT.cameraBusy;
  if (s.camera.status !== 'connected') return TEXT.cameraNotConnected;
  if (s.camera.live === 'unavailable') return TEXT.liveUnavailable;
  return null;
}

/** Design §2 mode pill. */
export function modeLabel(s: PhoneState): string {
  if (inFreeShot(s.view)) return s.locked ? 'FREE SHOT · LOCKED' : 'FREE SHOT · LIVE';
  return s.locked ? 'LIVE · LOCKED' : s.holding ? 'LIVE' : 'TEMPLATE';
}

/** Design §2 hint chip. In a templated session the Free shot adds Condition images (card LV7). */
export function hintText(s: PhoneState): string {
  if (inFreeShot(s.view)) return s.view?.templated ? 'No template · each capture adds a condition image' : 'No template · each capture adds an extra image';
  return s.locked ? 'Tap to focus · double-tap to unlock' : 'Hold for live · slide up to lock · slide sideways for overlay';
}

/** The exposure readout on live (read-only from the camera). */
export function exposureText(camera: CameraMessage | null): string | null {
  const e = camera?.exposure;
  return e ? `${e.shutter} · ${e.aperture} · ${e.iso}` : null;
}

/**
 * The control bar's Next: the next empty template shot after the current one in shooting order,
 * and Free shot after the last (then the label says so). Off in Free shot.
 */
export function nextStep(view: PhoneSession | null | undefined): { enabled: boolean; label: 'Next shot' | 'Free shot'; container_id: number | null } {
  const shot = currentShot(view);
  if (!view || !shot) return { enabled: false, label: 'Next shot', container_id: null };
  const after = view.shots.slice(view.shots.indexOf(shot) + 1).find(candidate => candidate.photo === null);
  return after ? { enabled: true, label: 'Next shot', container_id: after.container_id } : { enabled: true, label: 'Free shot', container_id: null };
}

/** Every photo the session view lists. */
export function photoIds(view: PhoneSession): number[] {
  return [...view.shots.flatMap(shot => (shot.photo ? [shot.photo.photo_id] : [])), ...view.extras.map(i => i.photo_id), ...view.condition.map(i => i.photo_id)];
}

/** The Main position of extra `index` (extras follow the template's containers). */
export const extraPosition = (view: PhoneSession, index: number) => view.shots.length + index + 1;

/** The capture toast for a photo, by where the Mac placed it (10J: reconciled with the camera, C3). */
export function landedToast(view: PhoneSession, photo_id: number): string | null {
  const shot = view.shots.find(candidate => candidate.photo?.photo_id === photo_id);
  if (shot) return `Shot ${pad(shot.final_no)} captured`;
  if (view.condition.some(image => image.photo_id === photo_id)) return TEXT.conditionAdded;
  const extra = view.extras.findIndex(image => image.photo_id === photo_id);
  return extra >= 0 ? `Extra image ${pad(extraPosition(view, extra))} added` : null;
}

/** One tile of the shot dock (design §3), in the dock's order. */
export type DockTile =
  | { key: string; kind: 'shot'; shot: PhoneShot; tag: string; label: string; current: boolean }
  | { key: string; kind: 'extra'; image: PhoneImage; tag: string; label: string; current: false }
  | { key: 'free'; kind: 'free'; tag: ''; label: string; current: boolean }
  | { key: string; kind: 'condition'; image: PhoneImage; tag: string; label: string; current: false };

/**
 * The dock's tiles: template shots in shooting order with their final numbers (card LV5), extra
 * Main images (tag = Main position), the Free shot tile, then the Condition images (C1…).
 */
export function dockTiles(view: PhoneSession): DockTile[] {
  const now = currentShot(view);
  return [
    ...view.shots.map((shot): DockTile => ({
      key: `s${shot.container_id}`, kind: 'shot', shot, tag: pad(shot.final_no), label: `${pad(shot.final_no)} · ${shot.name}`, current: shot === now,
    })),
    ...view.extras.map((image, index): DockTile => ({
      key: `p${image.photo_id}`, kind: 'extra', image, tag: pad(extraPosition(view, index)), label: `${pad(extraPosition(view, index))} · Extra image`, current: false,
    })),
    { key: 'free', kind: 'free', tag: '', label: 'Free shot · no template', current: now === null },
    ...view.condition.map((image, index): DockTile => ({
      key: `p${image.photo_id}`, kind: 'condition', image, tag: `C${index + 1}`, label: `Condition ${pad(index + 1)}`, current: false,
    })),
  ];
}

/** The image a dock tile shows, if it has one. */
export const tileImage = (tile: DockTile): PhoneImage | null => (tile.kind === 'shot' ? tile.shot.photo : tile.kind === 'free' ? null : tile.image);

/** What an open preview shows: a template shot's photo, an extra Main image, or a Condition image. */
export type PreviewSubject =
  | { kind: 'shot'; image: PhoneImage; shot: PhoneShot }
  | { kind: 'extra'; image: PhoneImage; position: number }
  | { kind: 'condition'; image: PhoneImage; position: number };

/** Where the photo is now in the Mac's session (`null`: it is no longer there). */
export function previewSubject(view: PhoneSession | null | undefined, photo_id: number | null): PreviewSubject | null {
  if (!view || photo_id === null) return null;
  const shot = view.shots.find(candidate => candidate.photo?.photo_id === photo_id);
  if (shot?.photo) return { kind: 'shot', image: shot.photo, shot };
  const extra = view.extras.findIndex(image => image.photo_id === photo_id);
  if (extra >= 0) return { kind: 'extra', image: view.extras[extra], position: extraPosition(view, extra) };
  const condition = view.condition.findIndex(image => image.photo_id === photo_id);
  return condition >= 0 ? { kind: 'condition', image: view.condition[condition], position: condition + 1 } : null;
}

/** Design §4 header: kicker, title and the ratio chip. A split composition is two squares (ratio 2). */
export function previewHeader(subject: PreviewSubject, ratio: number): { kicker: string; title: string; chip: string; mode: string } {
  const split = subject.kind === 'shot' ? subject.shot.split : subject.image.ratio === 2;
  const chip = ratioLabel(ratio, split);
  if (subject.kind === 'shot') return { kicker: `SHOT ${pad(subject.shot.final_no)} · CAPTURED`, title: subject.shot.name, chip, mode: 'CAPTURED' };
  if (subject.kind === 'extra') return { kicker: `IMAGE ${pad(subject.position)} · EXTRA`, title: 'Extra image', chip, mode: 'EXTRA' };
  return { kicker: `CONDITION ${pad(subject.position)}`, title: 'Condition image', chip, mode: 'CONDITION' };
}

/** The preview image's aspect ratio: its own composition's; a split shot is 2; an open one the live frame's. */
export function previewRatio(s: PhoneState, subject: PreviewSubject): number {
  if (subject.kind === 'shot' && subject.shot.split) return 2;
  return subject.image.ratio ?? (subject.kind === 'shot' ? subject.shot.ratio : null) ?? frameAspect(s);
}

/** "Back to shot 04" / "Back to free shot" (design §4). */
export function backLabel(view: PhoneSession): string {
  const shot = currentShot(view);
  return shot ? `Back to shot ${pad(shot.final_no)}` : 'Back to free shot';
}

/** One tile of the Move drawer (design §5): where the image can go, and what the Mac is asked (PH4). */
export type MoveTile = {
  key: string;
  /** The number tag ("03", "C1"); none on an "add at end" tile. */
  tag: string;
  name: string;
  state: 'THIS IMAGE' | 'SWAP' | 'EMPTY' | 'ADD AT END';
  image: PhoneImage | null;
  /** The template example shown in an empty template position. */
  example: number | null;
  target: MoveTarget;
  /** The toast after the move (design §7). */
  done: string;
};

/**
 * The Move drawer's tiles for `subject` (card LV8: a position that has an image swaps the two).
 * Main positions in Main order (the template's containers by final number, then the extras) and
 * "New extra"; the Condition images and "Add".
 */
export function moveTiles(view: PhoneSession, subject: PreviewSubject): { main: MoveTile[]; condition: MoveTile[] } {
  const source = subject.image.photo_id;
  const fromCondition = subject.kind === 'condition';
  const main: MoveTile[] = [
    ...[...view.shots].sort((a, b) => a.final_no - b.final_no).map((shot): MoveTile => ({
      key: `s${shot.container_id}`, tag: pad(shot.final_no), name: shot.name, image: shot.photo, example: shot.example ? shot.container_id : null,
      state: shot.photo?.photo_id === source ? 'THIS IMAGE' : shot.photo ? 'SWAP' : 'EMPTY',
      target: { kind: 'container', container_id: shot.container_id },
      done: shot.photo ? `Swapped with ${pad(shot.final_no)}` : `Moved to shot ${pad(shot.final_no)}`,
    })),
    ...view.extras.map((image, index): MoveTile => {
      const position = extraPosition(view, index);
      return {
        key: `p${image.photo_id}`, tag: pad(position), name: 'Extra', image, example: null, state: image.photo_id === source ? 'THIS IMAGE' : 'SWAP',
        target: { kind: 'main_position', position }, done: `Swapped with ${pad(position)}`,
      };
    }),
    { key: 'extra-end', tag: '', name: 'New extra', image: null, example: null, state: 'ADD AT END', target: { kind: 'extra_end' }, done: 'Added as extra image' },
  ];
  const condition: MoveTile[] = [
    ...view.condition.map((image, index): MoveTile => ({
      key: `p${image.photo_id}`, tag: `C${index + 1}`, name: `Condition ${pad(index + 1)}`, image, example: null, state: image.photo_id === source ? 'THIS IMAGE' : 'SWAP',
      target: { kind: 'condition', position: index + 1 }, done: fromCondition ? `Reordered to C${index + 1}` : `Swapped with C${index + 1}`,
    })),
    {
      key: 'condition-end', tag: '', name: fromCondition ? 'Move to end' : 'Add', image: null, example: null, state: 'ADD AT END', target: { kind: 'condition_end' },
      done: fromCondition ? 'Moved to end of condition images' : 'Sent to condition images',
    },
  ];
  return { main, condition };
}

/** The Item drawer's rows (LV-D17, design §6). "Article number" only for an item that has a barcode. */
export function itemRows(s: PhoneState, view: PhoneSession): { label: string; value: string }[] {
  const shot = currentShot(view);
  const ratio = ratioLabel(stageRatio(s), shot?.split ?? false);
  return [
    { label: 'SKU', value: view.item.sku },
    ...(view.item.article_number !== null ? [{ label: 'Article number', value: view.item.article_number }] : []),
    { label: 'Current shot', value: shot ? `${pad(shot.final_no)}/${pad(view.shots.length)} · ${ratio}` : `FREE SHOT · ${ratio}` },
    { label: 'Angle', value: shot?.name ?? '' },
    { label: 'Images', value: `${view.counts.main} main · ${view.counts.condition} condition` },
  ];
}
