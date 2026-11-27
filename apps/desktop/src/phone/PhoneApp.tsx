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
import { PhotoDrawer } from './components/PhotoDrawer';
import { Preview } from './components/Preview';
import { PropertiesPage } from './components/PropertiesPage';
import { ReframeEditor } from './components/ReframeEditor';
import { Stage } from './components/Stage';
import { Toast } from './components/Toast';
import { isRotated, type Point } from './geometry';
import { buzz, DRAG, HAPTIC } from './gestures';
import { type CaptureMessage, type FocusMessage, PhoneSocket, type ServerMessage } from './socket';
import {
  connected, type DockTile, initialState, itemRows, landedToast, nextStep, pad, photoIds, previewHeader, previewRatio, previewSubject, reduce, stageRatio, TEXT,
  tileImage, wantsLive,
} from './state';

/** While the Mac does not answer the first two requests they are asked again this often. */
const RETRY_MS = 2000;

/** PH9.5a: the drops and swipes the Mac is still answering, and what the strips show meanwhile. */
type Chain = {
  shown: PhoneSession;
  /** Each outstanding request's 8 s bound, by the request's number. */
  open: Map<number, ReturnType<typeof setTimeout>>;
  /** Once every request is answered: the session reads asked by then (a later read ends the chain). */
  after: number | null;
  /** The bound of that later read. */
  follow: ReturnType<typeof setTimeout> | null;
};

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

  // PH9.5a, the pending chain: while the Mac answers drops and swipes, both strips show them applied
  // (`pending`), each one on top of what the strips showed when it was made. The chain ends exactly
  // four ways: (a) every request is answered and a session read started after the last answer
  // returns (`after` = the reads asked by then); (b) the Mac refuses one; (c) one has no answer
  // `DRAG.pendingMs` after it was sent (`open` holds each outstanding request's own bound, stopped
  // by its answer); (d) the read that follows the last answer has not returned in that time
  // (`follow`). At every end the strips show the last view from the Mac.
  const [pending, setPending] = useState<PhoneSession | null>(null);
  const chain = useRef<Chain | null>(null);
  const sent = useRef(0);
  const endPending = useCallback(() => {
    const now = chain.current;
    chain.current = null;
    if (now) {
      now.open.forEach(timer => clearTimeout(timer));
      if (now.follow) clearTimeout(now.follow);
    }
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
        // PH9.5a (a): a read started before the last answer may predate that change: only a later one ends the chain.
        if (chain.current && chain.current.after !== null && mine > chain.current.after) endPending();
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

  /** Design §Preview actions, from the photo drawer since LV-D21 (4): Retake reopens a template shot (card LV6) and closes the drawer and the preview. */
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

  // LV-D21 (6): the reframe editor, over the Preview of the photo it was opened for. It closes with
  // nothing sent when that photo leaves the session or the Mac no longer has its full-frame preview.
  const [reframing, setReframing] = useState<number | null>(null);
  const reframed = reframing !== null && subject?.image.photo_id === reframing && subject.image.composition ? { ...subject.image, composition: subject.image.composition } : null;
  useEffect(() => {
    if (reframing !== null && !reframed) setReframing(null);
  }, [reframing, reframed]);

  /** What the strips show: the pending arrangement while the Mac answers, else the Mac's view. */
  const arranged = pending && view && pending.id === view.id ? pending : view;
  /** The same, read where a drop or a swipe is made (a second one may come before the strips are drawn again). */
  const shownNow = (mac: PhoneSession) => (chain.current && chain.current.shown.id === mac.id ? chain.current.shown : mac);

  /** PH9.5a: a drop or a swipe is sent and the strips show `shown` from now on. Returns the request's number. */
  const ask = (shown: PhoneSession): number => {
    const n = ++sent.current;
    const now = chain.current ?? { shown, open: new Map(), after: null, follow: null };
    // Sent while the read after the last answer is awaited: the chain waits for answers again.
    if (now.follow) clearTimeout(now.follow);
    now.follow = null;
    now.after = null;
    now.shown = shown;
    now.open.set(n, setTimeout(() => {
      // (c) No answer in time: the Mac is asked what is true.
      if (!chain.current?.open.has(n)) return;
      endPending();
      void readSession();
    }, DRAG.pendingMs));
    chain.current = now;
    setPending(shown);
    return n;
  };
  /** PH9.5a: the Mac answered request `n`; after the last one, the read asked next ends the chain (a), or its bound does (d). */
  const answered = (n: number) => {
    const now = chain.current;
    const bound = now?.open.get(n);
    if (!now || bound === undefined) return;
    clearTimeout(bound);
    now.open.delete(n);
    if (now.open.size > 0) return;
    now.after = asked.current;
    now.follow = setTimeout(() => {
      if (chain.current === now) endPending();
    }, DRAG.pendingMs);
  };
  /** PH9.5a (b): the Mac refused request `n`: the strips go back to its last view. */
  const refusedAhead = (n: number) => {
    if (chain.current?.open.has(n)) endPending();
  };

  /**
   * PH9.2–PH9.5a: a lifted photo was released on a target. The Mac is asked (a swap or an insert is
   * a move, the other group is a classify) and the strips show the drop at once; a successful drop
   * says nothing (the Mac's next view shows it), a refused one goes back and says why, and one the
   * Mac has not answered in 8 s goes back and asks the Mac what is true.
   */
  const drop = async (photoId: number, target: Drop) => {
    if (!view) return;
    if (target.kind === 'main_confirm') return setConfirming({ photo_id: photoId, dropped: true });
    const base = shownNow(view);
    const n = ask(applyDrop(base, photoId, target));
    buzz(HAPTIC.move);
    // A template shot that loses its photo to the other group becomes the current shot (plan point P6).
    if (target.kind !== 'swap' && target.kind !== 'insert' && previewSubject(base, photoId)?.kind === 'shot') clearFocus();
    try {
      if (target.kind === 'swap') await movePhoto(token, photoId, view.id, { kind: 'container', container_id: target.container_id });
      else if (target.kind === 'insert') await movePhoto(token, photoId, view.id, { kind: target.group === 'extra' ? 'extra_insert' : 'condition_insert', index: target.index });
      else await classifyPhoto(token, photoId, view.id, target.kind === 'to_condition' ? 'condition' : 'main');
      answered(n);
      void readSession();
    } catch (error) {
      refusedAhead(n);
      refused(error, TEXT.failed);
    }
  };

  /**
   * PH9.1, PH9.5a: a counted swipe on the previewed photo. Both strips show it at once (a Deny
   * takes the photo out, Main and Condition put it at that group's end) and the Preview shows the
   * next photo of what the strips show, before any re-read could close it because its photo left;
   * then the Mac is asked. A refusal puts the strips back, says why and re-reads; the Preview does
   * not go back.
   */
  const swipe = async (action: Exclude<SwipeAction, 'none' | 'main_confirm'>) => {
    if (!view || !subject) return;
    const id = subject.image.photo_id;
    const base = shownNow(view);
    dispatch({ type: 'preview', photo_id: nextPhoto(base, id) });
    const n = ask(applyDrop(base, id, { kind: action === 'main' ? 'to_main' : action === 'condition' ? 'to_condition' : 'remove' }));
    buzz(action === 'retake' || action === 'delete' ? HAPTIC.retake : HAPTIC.move);
    // A template shot that loses its photo becomes the current shot (plan point P6).
    if (subject.kind === 'shot') clearFocus();
    try {
      let done: string;
      if (action === 'retake' && subject.kind === 'shot') {
        await retake(token, view.id, subject.shot.container_id);
        done = `Shot ${pad(subject.shot.final_no)} reopened`;
      } else if (action === 'main' || action === 'condition') {
        await classifyPhoto(token, id, view.id, action);
        done = action === 'main' ? TEXT.movedToMain : TEXT.sentToCondition;
      } else {
        await deletePhoto(token, id, view.id);
        done = TEXT.imageDeleted;
      }
      answered(n);
      toast(done);
      void readSession();
    } catch (error) {
      refusedAhead(n);
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
      enabled={!state.detailsOpen && confirming === null && reframed === null} onClose={() => dispatch({ type: 'preview', photo_id: null })} onDetails={() => {
        buzz(HAPTIC.menu);
        dispatch({ type: 'details', open: true });
      }} onSwipe={action => (action === 'main_confirm' ? setConfirming({ photo_id: subject.image.photo_id, dropped: false }) : void swipe(action))}
      strip={arranged && <Dock token={token} view={arranged} rotated={isRotated(previewRatio(state, subject))} enabled={!state.detailsOpen && confirming === null && reframed === null}
        onSelect={selectInPreview} onDrop={(photoId, target) => void drop(photoId, target)} pending={pending !== null} testId="preview-strip"
        marked={subject.image.photo_id} />} />}
    {view && subject && <PhotoDrawer open={state.detailsOpen} {...previewHeader(subject, previewRatio(state, subject))} hasTemplate={subject.kind === 'shot'}
      canReframe={!!subject.image.composition} onClose={() => dispatch({ type: 'details', open: false })} onRetake={() => void retakeShot()} onReframe={() => {
        buzz(HAPTIC.select);
        dispatch({ type: 'details', open: false });
        setReframing(subject.image.photo_id);
      }} />}
    {view && reframed && <ReframeEditor key={reframed.photo_id} token={token} image={reframed} defaultShape={view.default_shape ?? '1:1'} onCancel={() => setReframing(null)} />}
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
