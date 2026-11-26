import { useCallback, useEffect, useReducer, useRef, useState } from 'react';
import { exampleUrl, forgetToken, hello, session, takeToken, Unpaired } from './api';
import { ConnectingOverlay } from './components/ConnectingOverlay';
import { FullscreenButton } from './components/FullscreenButton';
import { LiveCanvas, type LiveCanvasHandle } from './components/LiveCanvas';
import { PhoneSocket } from './socket';
import { cameraNotice, connected, initialState, liveShown, reduce, TEXT } from './state';

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

  // The session is only ever what the Mac last answered; a slower, older answer never replaces a newer one.
  const asked = useRef(0);
  const readSession = useCallback(async () => {
    const mine = ++asked.current;
    try {
      const answer = await session(token);
      if (mine === asked.current) dispatch({ type: 'view', view: answer.session });
      return true;
    } catch (error) {
      if (error instanceof Unpaired) onRemoved();
      return false;
    }
  }, [token, onRemoved]);

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
      onMessage: message => {
        if (message.type === 'session') void readSession();
        else if (message.type === 'camera') dispatch({ type: 'camera', camera: message });
      },
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
  const wants = state.holding || state.locked;
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

  const view = state.view ?? null;
  const shot = view?.current?.kind === 'shot' ? view.shots.find(s => s.container_id === (view.current as { container_id: number }).container_id) ?? null : null;
  const showing = liveShown(state);
  const notice = cameraNotice(state);
  const hold = (holding: boolean) => {
    if (isConnected) dispatch({ type: 'hold', holding });
  };

  return <div className="ph-app">
    <div className="ph-safe-top" />
    <FullscreenButton />
    <div className="ph-stage">
      <div className="ph-viewport" data-testid="viewport" style={{ width: '100%', aspectRatio: '1' }}
        onPointerDown={event => {
          event.currentTarget.setPointerCapture(event.pointerId);
          hold(true);
        }}
        onPointerUp={() => hold(false)} onPointerCancel={() => hold(false)} onContextMenu={event => event.preventDefault()}>
        <LiveCanvas ref={canvas} isCurrent={gen => socket.current?.gen === gen} shown={showing} scale={1}
          onDrawn={(gen, width, height) => dispatch({ type: 'drawn', gen, width, height })} />
        <div className="ph-layer ph-template-layer" data-testid="template-layer" style={{ opacity: shot ? (showing ? state.overlay / 100 : 1) : 0 }}>
          {shot?.example && <img className="ph-cover ph-grey" alt="" draggable={false} src={exampleUrl(token, shot.container_id)} />}
        </div>
        {notice && <div className="ph-hint-row"><div className="ph-hint">{notice}</div></div>}
      </div>
    </div>
    <ConnectingOverlay connected={isConnected} macName={state.hello?.mac_name ?? null} itemName={view?.item.name ?? null} />
  </div>;
}
