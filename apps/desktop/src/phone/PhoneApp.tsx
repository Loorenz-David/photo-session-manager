import { useCallback, useEffect, useReducer, useRef, useState } from 'react';
import { ApiError, capture, focus, focusClear, forgetToken, hello, type PhoneSession, previewUrl, session, setCurrent, takeToken, Unpaired } from './api';
import { ConnectingOverlay } from './components/ConnectingOverlay';
import { ControlBar } from './components/ControlBar';
import { Dock } from './components/Dock';
import { FullscreenButton } from './components/FullscreenButton';
import type { LiveCanvasHandle } from './components/LiveCanvas';
import { Stage } from './components/Stage';
import { Toast } from './components/Toast';
import { isRotated, type Point } from './geometry';
import { buzz, HAPTIC } from './gestures';
import { type CaptureMessage, type FocusMessage, PhoneSocket, type ServerMessage } from './socket';
import { connected, type DockTile, initialState, landedToast, nextStep, photoIds, reduce, stageRatio, TEXT, tileImage, wantsLive } from './state';

/** While the Mac does not answer the first two requests they are asked again this often. */
const RETRY_MS = 2000;

/**
 * The Shoot Companion (book 10J §The phone design): the phone follows the Mac's active session
 * (LV-D18). PH7: the token comes from the QR code's link once and stays on the phone; a phone the
 * Mac no longer knows is told to pair again.
 */
export function PhoneApp() {
  const [token] = useState(takeToken);
  const [removed, setRemoved] = useState(false);
  const onRemoved = useCallback(() => {
    forgetToken();
    setRemoved(true);
  }, []);
  if (removed) return <p className="ph-pairing">{TEXT.removed}</p>;
  if (!token) return <p className="ph-pairing">{TEXT.notPaired}</p>;
  return <Companion token={token} onRemoved={onRemoved} />;
}

function Companion({ token, onRemoved }: { token: string; onRemoved: () => void }) {
  const [state, dispatch] = useReducer(reduce, initialState);
  const canvas = useRef<LiveCanvasHandle>(null);
  const socket = useRef<PhoneSocket | null>(null);
  const isConnected = connected(state);
  const view = state.view ?? null;
  const toast = useCallback((text: string) => dispatch({ type: 'toast', text }), []);

  // Capture feedback is reconciled with the camera (10J, C3: an acknowledgement is not an
  // exposure, an exposure is not a file): the toast names where the Mac placed the new photo.
  const captures = useRef({ mine: new Set<string>(), early: new Map<string, CaptureMessage['state']>(), exposed: 0, session: null as number | null, known: new Set<number>() });
  const reconcile = useCallback((next: PhoneSession | null) => {
    const c = captures.current;
    const ids = next ? photoIds(next) : [];
    if (next && next.id === c.session) {
      for (const id of ids) {
        if (c.exposed === 0) break;
        if (c.known.has(id)) continue;
        c.exposed -= 1;
        const text = landedToast(next, id);
        if (text) toast(text);
      }
    } else c.exposed = 0;
    c.session = next?.id ?? null;
    c.known = new Set(ids);
  }, [toast]);

  // The session is only ever what the Mac last answered; a slower, older answer never replaces a newer one.
  const asked = useRef(0);
  const readSession = useCallback(async () => {
    const mine = ++asked.current;
    try {
      const answer = await session(token);
      if (mine === asked.current) {
        reconcile(answer.session);
        dispatch({ type: 'view', view: answer.session });
      }
      return true;
    } catch (error) {
      if (error instanceof Unpaired) onRemoved();
      return false;
    }
  }, [token, onRemoved, reconcile]);

  /** A request the Mac refused: its code decides what the person is told (master_plan_live §6). */
  const refused = useCallback((error: unknown, otherwise: string) => {
    if (error instanceof Unpaired) return onRemoved();
    const code = error instanceof ApiError ? error.code : '';
    if (code === 'session_changed') {
      toast(TEXT.sessionChanged);
      void readSession();
    } else if (code === 'camera_not_connected') toast(TEXT.cameraNotConnected);
    else {
      toast(otherwise);
      void readSession();
    }
  }, [onRemoved, readSession, toast]);

  const captureReport = useCallback((id: string, report: CaptureMessage['state']) => {
    const c = captures.current;
    if (report === 'acknowledged') return;
    c.mine.delete(id);
    if (report === 'exposed') c.exposed += 1;
    else toast(TEXT.notTaken);
  }, [toast]);

  // The camera's answer to a tap (LV-D14): the marker is confirmed, or taken away with the reason.
  const focusAsked = useRef<{ id: string | null; early: FocusMessage[] }>({ id: null, early: [] });
  const focusReport = useCallback((message: FocusMessage) => {
    if (message.outcome === 'focused') return dispatch({ type: 'focus-confirmed', id: message.focus_id });
    if (message.outcome === 'superseded') return;
    dispatch({ type: 'focus-lost', id: message.focus_id });
    if (message.outcome !== 'refused') toast(TEXT.couldNotFocus);
    else toast(message.reason === 'focus_area_not_movable' ? TEXT.focusAreaNotMovable : message.reason === 'not_connected' ? TEXT.cameraNotConnected : TEXT.couldNotFocus);
  }, [toast]);

  const onMessage = useRef<(message: ServerMessage) => void>(() => undefined);
  onMessage.current = message => {
    if (message.type === 'session') void readSession();
    else if (message.type === 'camera') dispatch({ type: 'camera', camera: message });
    else if (message.type === 'capture') {
      // The report can overtake the answer to the capture request itself.
      if (captures.current.mine.has(message.capture_id)) captureReport(message.capture_id, message.state);
      else captures.current.early.set(message.capture_id, message.state);
    } else if (message.type === 'focus') {
      if (focusAsked.current.id === message.focus_id) focusReport(message);
      else if (focusAsked.current.id === null) focusAsked.current.early.push(message);
    }
  };

  useEffect(() => {
    let stopped = false;
    let retry: ReturnType<typeof setTimeout> | null = null;
    let greeted = false;
    let loaded = false;
    // Design §1: the overlay stays until `hello` and the session are both answered.
    const load = async () => {
      if (!greeted) {
        try {
          const answer = await hello(token);
          greeted = true;
          if (!stopped) dispatch({ type: 'hello', hello: answer });
        } catch (error) {
          if (error instanceof Unpaired) return onRemoved();
        }
      }
      if (!loaded) loaded = await readSession();
      if (!stopped && !(greeted && loaded)) retry = setTimeout(() => void load(), RETRY_MS);
    };
    void load();
    const live = new PhoneSocket(token, {
      onMessage: message => onMessage.current(message),
      onFrame: frame => canvas.current?.draw(frame),
      onDemand: gen => dispatch({ type: 'demand', gen }),
      // What happened while the socket was away is not replayed: read the session again.
      onOpen: () => void readSession(),
      onRevoked: onRemoved,
    });
    socket.current = live;
    live.start();
    return () => {
      stopped = true;
      if (retry) clearTimeout(retry);
      live.stop();
      socket.current = null;
    };
  }, [token, onRemoved, readSession]);

  // PH7: every change of showing Live is a new demand.
  const wants = isConnected && wantsLive(state);
  useEffect(() => {
    socket.current?.setLive(wants);
  }, [wants]);

  // Design §Reliability: a hold never outlives the window's attention.
  useEffect(() => {
    const release = () => dispatch({ type: 'hold', holding: false });
    const hidden = () => {
      if (document.visibilityState === 'hidden') release();
    };
    window.addEventListener('blur', release);
    document.addEventListener('visibilitychange', hidden);
    return () => {
      window.removeEventListener('blur', release);
      document.removeEventListener('visibilitychange', hidden);
    };
  }, []);

  /** The marker goes, and the camera's focus point goes back to the centre with it (card LV9). */
  const clearFocus = useCallback(() => {
    if (!state.focus || !view) return;
    focusAsked.current = { id: null, early: [] };
    dispatch({ type: 'focus', focus: null });
    focusClear(token, view.id).catch(error => {
      if (error instanceof Unpaired) onRemoved();
    });
  }, [state.focus, view, token, onRemoved]);

  const onFocus = async (mark: Point, point: Point) => {
    if (!view) return;
    buzz(HAPTIC.focus);
    const asking = { id: null as string | null, early: [] as FocusMessage[] };
    focusAsked.current = asking;
    dispatch({ type: 'focus', focus: { ...mark, id: null, confirmed: false } });
    try {
      const { focus_id } = await focus(token, view.id, point.x, point.y);
      if (focusAsked.current !== asking) return;
      asking.id = focus_id;
      dispatch({ type: 'focus-id', id: focus_id });
      asking.early.filter(message => message.focus_id === focus_id).forEach(focusReport);
      asking.early = [];
    } catch (error) {
      if (focusAsked.current === asking) dispatch({ type: 'focus', focus: null });
      refused(error, TEXT.couldNotFocus);
    }
  };

  const onCapture = async () => {
    if (!view) return;
    // Design §Shutter: the flash and the haptic are immediate; the toast waits for the camera.
    buzz(HAPTIC.capture);
    dispatch({ type: 'flash' });
    try {
      const { capture_id } = await capture(token, view.id);
      const c = captures.current;
      c.mine.add(capture_id);
      const early = c.early.get(capture_id);
      c.early.clear();
      if (early) captureReport(capture_id, early);
    } catch (error) {
      refused(error, TEXT.notTaken);
    }
  };

  /** The person chooses the current shot (`null` = Free shot); the Mac decides, the phone re-reads. */
  const choose = async (container_id: number | null) => {
    if (!view?.templated) return;
    const now = view.current?.kind === 'shot' ? view.current.container_id : null;
    if (now === container_id) return;
    buzz(HAPTIC.select);
    clearFocus();
    try {
      await setCurrent(token, view.id, container_id);
    } catch (error) {
      refused(error, TEXT.failed);
    }
  };

  /** Design §Selecting a tile: an image opens its preview; an empty template shot (or Free shot) becomes current. */
  const select = (tile: DockTile) => {
    const image = tileImage(tile);
    if (image) {
      buzz(HAPTIC.select);
      dispatch({ type: 'preview', photo_id: image.photo_id });
    } else void choose(tile.kind === 'shot' ? tile.shot.container_id : null);
  };

  const next = nextStep(view);
  const rotated = isRotated(stageRatio(state));
  const opened = view ? [...view.shots.flatMap(shot => (shot.photo ? [shot.photo] : [])), ...view.extras, ...view.condition].find(image => image.photo_id === state.preview) : undefined;
  if (isConnected && !view) {
    return <div className="ph-app">
      <p className="ph-pairing">{TEXT.noSession}</p>
    </div>;
  }
  return <div className="ph-app">
    <div className="ph-safe-top" />
    <FullscreenButton />
    <Stage token={token} state={state} dispatch={dispatch} enabled={isConnected && !opened} canvas={canvas}
      isCurrentGen={gen => socket.current?.gen === gen} onFocus={(mark, point) => void onFocus(mark, point)} onClearFocus={() => {
        buzz(HAPTIC.clearFocus);
        clearFocus();
      }} />
    {view && <Dock token={token} view={view} rotated={rotated} enabled={isConnected && !opened} onSelect={select} />}
    <ControlBar rotated={rotated} next={next} onMenu={() => undefined} onCapture={() => void onCapture()} onNext={() => void choose(next.container_id)} />
    <Toast toast={state.toast} />
    {opened && <div className="ph-preview" data-testid="preview">
      <img className="ph-cover" alt="" draggable={false} src={previewUrl(token, opened)} />
    </div>}
    <ConnectingOverlay connected={isConnected} macName={state.hello?.mac_name ?? null} itemName={view?.item.name ?? null} />
  </div>;
}
