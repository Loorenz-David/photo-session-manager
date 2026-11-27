import { useCallback, useEffect, useReducer, useRef, useState } from 'react';
import { ApiError, capture, classifyPhoto, deletePhoto, focus, focusClear, forgetToken, hello, movePhoto, type PhoneSession, retake, session, setCurrent, takeToken, Unpaired } from './api';
import { applyDrop, type Drop, nextPhoto, type SwipeAction } from './arrange';
import { ConfirmSheet } from './components/ConfirmSheet';
import { ConnectingOverlay } from './components/ConnectingOverlay';
import { ControlBar } from './components/ControlBar';
import { Dock } from './components/Dock';
import { FullscreenButton } from './components/FullscreenButton';
import { ItemDrawer } from './components/ItemDrawer';
import type { LiveCanvasHandle } from './components/LiveCanvas';
import { MoveDrawer } from './components/MoveDrawer';
import { Preview } from './components/Preview';
import { PropertiesPage } from './components/PropertiesPage';
import { Stage } from './components/Stage';
import { Toast } from './components/Toast';
import { isRotated, type Point } from './geometry';
import { buzz, DRAG, HAPTIC } from './gestures';
import { type CaptureMessage, type FocusMessage, PhoneSocket, type ServerMessage } from './socket';
import {
  backLabel, connected, type DockTile, initialState, itemRows, landedToast, type MoveTile, nextStep, pad, photoIds, previewRatio, previewSubject, reduce, stageRatio,
  TEXT, tileImage, wantsLive,
} from './state';

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

  // PH9.5: while the Mac answers a drop, the strips show where the photo was put (`pending`). It
  // ends exactly three ways: the first session read started after the Mac's answer returns
  // (`after` = the reads asked by then), the Mac refuses, or `DRAG.pendingMs` pass.
  const [pending, setPending] = useState<PhoneSession | null>(null);
  const waiting = useRef<{ n: number; after: number | null; timer: ReturnType<typeof setTimeout> } | null>(null);
  const drops = useRef(0);
  const endPending = useCallback(() => {
    if (waiting.current) clearTimeout(waiting.current.timer);
    waiting.current = null;
    setPending(null);
  }, []);
  useEffect(() => endPending, [endPending]);

  // The session is only ever what the Mac last answered; a slower, older answer never replaces a newer one.
  const asked = useRef(0);
  const readSession = useCallback(async () => {
    const mine = ++asked.current;
    try {
      const answer = await session(token);
      if (mine === asked.current) {
        reconcile(answer.session);
        dispatch({ type: 'view', view: answer.session });
        // A read started before the answer may predate the move: only a later one ends the wait.
        if (waiting.current && waiting.current.after !== null && mine > waiting.current.after) endPending();
      }
      return true;
    } catch (error) {
      if (error instanceof Unpaired) onRemoved();
      return false;
    }
  }, [token, onRemoved, reconcile, endPending]);

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
      void readSession();
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

  // The open preview follows its photo wherever the Mac's session has it now (design §Move
  // semantics); a photo that left the session closes it.
  const subject = previewSubject(view, state.preview);
  const orphaned = state.preview !== null && state.view !== undefined && !subject;
  useEffect(() => {
    if (orphaned) dispatch({ type: 'preview', photo_id: null });
  }, [orphaned]);

  /** Design §Preview actions: Retake reopens a template shot (card LV6) and closes the preview. */
  const retakeShot = async () => {
    if (!view || subject?.kind !== 'shot') return;
    buzz(HAPTIC.retake);
    try {
      clearFocus();
      await retake(token, view.id, subject.shot.container_id);
      toast(`Shot ${pad(subject.shot.final_no)} reopened`);
      dispatch({ type: 'preview', photo_id: null });
      void readSession();
    } catch (error) {
      refused(error, TEXT.failed);
    }
  };

  // LV-D20 (1): a template shot's photo goes to the extra Main photos only after "Move"; asked
  // after a right swipe in the Preview, and after a drop among the extras in a strip.
  const [confirming, setConfirming] = useState<{ photo_id: number; dropped: boolean } | null>(null);
  const asksStill = confirming !== null && previewSubject(view, confirming.photo_id)?.kind === 'shot';
  useEffect(() => {
    if (confirming !== null && !asksStill) setConfirming(null);
  }, [confirming, asksStill]);

  /**
   * PH9.2–PH9.5: a lifted photo was released on a target. The Mac is asked (a swap or an insert is
   * a move, the other group is a classify) and the strips show the drop at once; a successful drop
   * says nothing (the Mac's next view shows it), a refused one goes back and says why, and one the
   * Mac has not answered in 8 s goes back and asks the Mac what is true.
   */
  const drop = async (photoId: number, target: Drop) => {
    if (!view) return;
    if (target.kind === 'main_confirm') return setConfirming({ photo_id: photoId, dropped: true });
    const n = ++drops.current;
    if (waiting.current) clearTimeout(waiting.current.timer);
    waiting.current = {
      n,
      after: null,
      timer: setTimeout(() => {
        if (waiting.current?.n !== n) return;
        endPending();
        void readSession();
      }, DRAG.pendingMs),
    };
    setPending(applyDrop(view, photoId, target));
    buzz(HAPTIC.move);
    // A template shot that loses its photo to the other group becomes the current shot (plan point P6).
    if (target.kind !== 'swap' && target.kind !== 'insert' && previewSubject(view, photoId)?.kind === 'shot') clearFocus();
    try {
      if (target.kind === 'swap') await movePhoto(token, photoId, view.id, { kind: 'container', container_id: target.container_id });
      else if (target.kind === 'insert') await movePhoto(token, photoId, view.id, { kind: target.group === 'extra' ? 'extra_insert' : 'condition_insert', index: target.index });
      else await classifyPhoto(token, photoId, view.id, target.kind === 'to_condition' ? 'condition' : 'main');
      if (waiting.current?.n === n) waiting.current.after = asked.current;
      void readSession();
    } catch (error) {
      if (waiting.current?.n === n) endPending();
      refused(error, TEXT.failed);
    }
  };
  /** What the strips show: the drop the Mac is answering, else the Mac's view. */
  const arranged = pending && view && pending.id === view.id ? pending : view;

  /**
   * PH9.1: a counted swipe on the previewed photo. The next photo shows at once (plan point P4),
   * before any re-read could close the preview because its photo left; then the Mac is asked. A
   * refusal says why and re-reads; the preview does not go back.
   */
  const swipe = async (action: Exclude<SwipeAction, 'none' | 'main_confirm'>) => {
    if (!view || !subject) return;
    const id = subject.image.photo_id;
    dispatch({ type: 'preview', photo_id: nextPhoto(view, id) });
    buzz(action === 'retake' || action === 'delete' ? HAPTIC.retake : HAPTIC.move);
    // A template shot that loses its photo becomes the current shot (plan point P6).
    if (subject.kind === 'shot') clearFocus();
    try {
      if (action === 'retake' && subject.kind === 'shot') {
        await retake(token, view.id, subject.shot.container_id);
        toast(`Shot ${pad(subject.shot.final_no)} reopened`);
      } else if (action === 'main' || action === 'condition') {
        await classifyPhoto(token, id, view.id, action);
        toast(action === 'main' ? TEXT.movedToMain : TEXT.sentToCondition);
      } else {
        await deletePhoto(token, id, view.id);
        toast(TEXT.imageDeleted);
      }
      void readSession();
    } catch (error) {
      refused(error, TEXT.failed);
    }
  };

  /** Plan point P11: in the Preview's strip a photo is shown; an empty shot or Free shot becomes current and the Preview closes. */
  const selectInPreview = (tile: DockTile) => {
    const image = tileImage(tile);
    if (image) {
      if (image.photo_id !== state.preview) buzz(HAPTIC.select);
      return dispatch({ type: 'preview', photo_id: image.photo_id });
    }
    dispatch({ type: 'preview', photo_id: null });
    void choose(tile.kind === 'shot' ? tile.shot.container_id : null);
  };

  /** Card LV8: the Mac performs the move (a filled position swaps); the preview stays on the image. */
  const move = async (tile: MoveTile) => {
    if (!view || !subject || tile.state === 'THIS IMAGE') return;
    buzz(HAPTIC.move);
    try {
      await movePhoto(token, subject.image.photo_id, view.id, tile.target);
      dispatch({ type: 'move', open: false });
      toast(tile.done);
      void readSession();
    } catch (error) {
      dispatch({ type: 'move', open: false });
      refused(error, TEXT.failed);
    }
  };

  const next = nextStep(view);
  const rotated = isRotated(stageRatio(state));
  const free = isConnected && !subject && !state.menuOpen && confirming === null;
  if (isConnected && !view) {
    return <div className="ph-app">
      <p className="ph-pairing">{TEXT.noSession}</p>
    </div>;
  }
  return <div className="ph-app">
    <div className="ph-safe-top" />
    <FullscreenButton />
    <Stage token={token} state={state} dispatch={dispatch} enabled={free} canvas={canvas}
      isCurrentGen={gen => socket.current?.gen === gen} onFocus={(mark, point) => void onFocus(mark, point)} onClearFocus={() => {
        buzz(HAPTIC.clearFocus);
        clearFocus();
      }} />
    {arranged && <Dock token={token} view={arranged} rotated={rotated} enabled={free} onSelect={select} onDrop={(photoId, target) => void drop(photoId, target)}
      pending={pending !== null} />}
    <ControlBar rotated={rotated} next={next} onCapture={() => void onCapture()} onNext={() => void choose(next.container_id)} onMenu={() => {
      buzz(HAPTIC.menu);
      dispatch({ type: 'menu', open: true });
    }} />
    <Toast toast={state.toast} />
    {view && subject && <Preview token={token} subject={subject} ratio={previewRatio(state, subject)} overlay={state.overlay} dispatch={dispatch}
      backLabel={backLabel(view)} onBack={() => dispatch({ type: 'preview', photo_id: null })} onRetake={() => void retakeShot()} onMove={() => {
        buzz(HAPTIC.menu);
        dispatch({ type: 'move', open: true });
      }} onSwipe={action => (action === 'main_confirm' ? setConfirming({ photo_id: subject.image.photo_id, dropped: false }) : void swipe(action))}
      strip={arranged && <Dock token={token} view={arranged} rotated={isRotated(previewRatio(state, subject))} enabled={!state.moveOpen && confirming === null}
        onSelect={selectInPreview} onDrop={(photoId, target) => void drop(photoId, target)} pending={pending !== null} testId="preview-strip"
        marked={subject.image.photo_id} />} />}
    {view && subject && <MoveDrawer token={token} view={view} subject={subject} open={state.moveOpen} onClose={() => dispatch({ type: 'move', open: false })}
      onPick={tile => void move(tile)} />}
    {view && <ItemDrawer open={state.menuOpen} itemName={view.item.name} macName={state.hello?.mac_name ?? ''} rows={itemRows(state, view)}
      onClose={() => dispatch({ type: 'menu', open: false })} onProperties={() => {
        buzz(HAPTIC.menu);
        dispatch({ type: 'props', open: true });
      }} />}
    <ConfirmSheet open={asksStill} text={TEXT.confirmMain} confirmLabel="Move" onCancel={() => setConfirming(null)} onConfirm={() => {
      setConfirming(null);
      if (!confirming) return;
      if (confirming.dropped) void drop(confirming.photo_id, { kind: 'to_main' });
      else if (subject?.image.photo_id === confirming.photo_id) void swipe('main');
    }} />
    {state.propsOpen && <PropertiesPage onBack={() => dispatch({ type: 'props', open: false })} />}
    <ConnectingOverlay connected={isConnected} macName={state.hello?.mac_name ?? null} itemName={view?.item.name ?? null} />
  </div>;
}
