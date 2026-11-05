import { useCallback, useEffect, useLayoutEffect, useRef, type RefObject } from 'react';
import { isTyping } from '@/lib/keyboard';
import type { CompositionValues } from '@/services/compositions';
import type { Photo } from '@/services/photos';
import { transformCss, viewTransform, type Box, type Point } from './geometry';
import { initialState, reduce, sameRect, WHEEL_SETTLE_MS, type ComposeEvent, type ComposeState } from './machine';
import type { Handle } from './refit';
import type { Rect } from '@/services/compositions';

type GestureOptions = {
  stageRef: RefObject<HTMLElement | null>;
  imageRef: RefObject<HTMLElement | null>;
  /** The photo on the stage; a change dispatches `SwitchPhoto`. */
  photo: Photo;
  /** The stage's measured client box; null until measured. */
  stageBox: Box | null;
  onSave: (photoId: number, values: CompositionValues) => void;
  onExit: () => void;
  onSwitch: (photoId: number) => void;
};

export type ComposeController = {
  /** Feeds the machine; returns true when the event's default must be prevented. */
  dispatch: (event: ComposeEvent) => boolean;
  isIdle: () => boolean;
  /** The machine's state for React (`useSyncExternalStore`): published after every event except the moves of a continuous drag, pinch or wheel. */
  subscribe: (listener: () => void) => () => void;
  getSnapshot: () => ComposeState | null;
};

/** Workspace keys that are off while composing (design handoff §Keyboard; 10D). */
const SWALLOWED = new Set(['a', 'c', 'backspace', 'delete', ' ', 'f', 'i', 'd', 'arrowleft', 'arrowright', 'arrowup', 'arrowdown']);

/** An open layer owns its own keyboard (the guard `useBackKeys` in `lib/keyboard.ts` uses; design handoff §Keyboard "Esc (close topmost)"). */
export const LAYER_OPEN = '[role="dialog"], [role="alertdialog"], [role="menu"]';

/** The glide back after a trim ends (`--dur-med` 280 ms) plus a margin, for a transition that never reports its end. */
const GLIDE_FALLBACK_MS = 400;

type GestureLike = Event & { clientX?: number; clientY?: number; scale?: number };

const CONTINUOUS = new Set(['dragging', 'pinching', 'wheeling']);

/**
 * Binds the DOM to the machine (plan P31 task 3, P32 tasks 2 and 5): pointer, wheel, WebKit gesture, blur, visibility
 * and keyboard events become machine events; a pointer down on a `.compose-handle` starts a trim instead of a drag;
 * `View` effects are coalesced into one transform write per animation frame; the wheel settle runs on a timer; a trim
 * that changed the rect glides the view back (`data-gliding` on the stage until `transitionend`); nothing here waits
 * for Rust.
 */
export function useComposeGesture(options: GestureOptions): ComposeController {
  const latest = useRef(options);
  latest.current = options;
  const stateRef = useRef<ComposeState | null>(null);
  const snapshotRef = useRef<ComposeState | null>(null);
  const listeners = useRef(new Set<() => void>());
  const pending = useRef<Rect | null>(null);
  const raf = useRef<number | null>(null);
  const timer = useRef<number | null>(null);
  const glideTimer = useRef<number | null>(null);
  const dispatchRef = useRef<(event: ComposeEvent) => boolean>(() => false);

  const publish = useCallback(() => {
    snapshotRef.current = stateRef.current;
    listeners.current.forEach(listener => listener());
  }, []);

  const write = useCallback((rect: Rect) => {
    const state = stateRef.current;
    const element = latest.current.imageRef.current;
    if (!state || !element) return;
    const view = viewTransform({ ...state.values, rect }, state.frame, state.view.stage);
    element.style.width = `${view.width}px`;
    element.style.height = `${view.height}px`;
    element.style.transform = transformCss(view);
  }, []);

  const flush = useCallback(() => {
    raf.current = null;
    const rect = pending.current;
    pending.current = null;
    if (rect) write(rect);
  }, [write]);

  const flushNow = useCallback(() => {
    if (raf.current !== null) {
      cancelAnimationFrame(raf.current);
      raf.current = null;
    }
    flush();
  }, [flush]);

  const syncTimer = useCallback(() => {
    const wheeling = stateRef.current?.phase.kind === 'wheeling';
    if (wheeling && timer.current === null) {
      timer.current = window.setInterval(() => dispatchRef.current({ type: 'Tick', at: Date.now() }), WHEEL_SETTLE_MS / 2);
    } else if (!wheeling && timer.current !== null) {
      window.clearInterval(timer.current);
      timer.current = null;
    }
  }, []);

  /** The frame glides back to fill the stage: a CSS transition on the image and overlay while `data-gliding` is set. */
  const glide = useCallback(() => {
    const stage = latest.current.stageRef.current;
    const image = latest.current.imageRef.current;
    if (!stage) return;
    stage.dataset.gliding = 'true';
    const done = () => {
      delete stage.dataset.gliding;
      image?.removeEventListener('transitionend', done);
      if (glideTimer.current !== null) window.clearTimeout(glideTimer.current);
      glideTimer.current = null;
    };
    image?.addEventListener('transitionend', done);
    if (glideTimer.current !== null) window.clearTimeout(glideTimer.current);
    glideTimer.current = window.setTimeout(done, GLIDE_FALLBACK_MS);
  }, []);

  const dispatch = useCallback(
    (event: ComposeEvent): boolean => {
      const current = stateRef.current;
      if (!current) return false;
      const { state, effects } = reduce(current, event);
      stateRef.current = state;
      const before = current.phase;
      if (before.kind === 'trimming' && state.phase.kind === 'idle' && !sameRect(state.live, before.startRect)) glide();
      let prevent = false;
      for (const effect of effects) {
        switch (effect.type) {
          case 'View':
            pending.current = effect.rect;
            if (raf.current === null) raf.current = requestAnimationFrame(flush);
            break;
          case 'Save':
            latest.current.onSave(effect.photoId, effect.values);
            break;
          case 'Exit':
            latest.current.onExit();
            break;
          case 'Switch':
            latest.current.onSwitch(effect.photoId);
            break;
          case 'PreventDefault':
            prevent = true;
            break;
        }
      }
      syncTimer();
      if (!(CONTINUOUS.has(state.phase.kind) && state.phase.kind === before.kind)) publish();
      return prevent;
    },
    [flush, glide, publish, syncTimer],
  );
  dispatchRef.current = dispatch;

  const { stageBox, photo } = options;
  useLayoutEffect(() => {
    if (!stageBox) return;
    const stage = { left: 0, top: 0, width: stageBox.width, height: stageBox.height };
    const state = stateRef.current;
    if (!state) {
      stateRef.current = initialState(photo.id, photo.composition, stage);
      if (stateRef.current) write(stateRef.current.live);
      publish();
      return;
    }
    if (state.photoId !== photo.id) {
      dispatch({ type: 'SwitchPhoto', photoId: photo.id, composition: photo.composition });
      flushNow();
    }
    const current = stateRef.current;
    if (current && (current.view.stage.width !== stage.width || current.view.stage.height !== stage.height)) {
      dispatch({ type: 'Resize', stage });
      flushNow();
    }
  }, [stageBox, photo, dispatch, flushNow, publish, write]);

  useEffect(() => {
    const element = latest.current.stageRef.current;
    if (!element) return;
    const local = (event: { clientX?: number; clientY?: number }): Point => {
      const rect = element.getBoundingClientRect();
      return { x: (event.clientX ?? 0) - rect.left, y: (event.clientY ?? 0) - rect.top };
    };
    const capture = (pointerId: number) => {
      if (typeof element.setPointerCapture !== 'function') return;
      try {
        element.setPointerCapture(pointerId);
      } catch {
        // The pointer cannot be captured (already released); the window listeners still follow it.
      }
    };
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Element | null;
      const handle = target?.closest?.('.compose-handle')?.getAttribute('data-handle') as Handle | null | undefined;
      if (handle && event.isPrimary && event.button === 0) {
        dispatch({ type: 'TrimStart', handle, pointerId: event.pointerId, point: local(event) });
        if (stateRef.current?.phase.kind === 'trimming') capture(event.pointerId);
        return;
      }
      dispatch({ type: 'PointerDown', pointerId: event.pointerId, isPrimary: event.isPrimary, button: event.button, point: local(event) });
      if (stateRef.current?.phase.kind === 'dragging') capture(event.pointerId);
    };
    const onPointerMove = (event: PointerEvent) => dispatch({ type: 'PointerMove', pointerId: event.pointerId, point: local(event) });
    const onPointerUp = (event: PointerEvent) => dispatch({ type: 'PointerUp', pointerId: event.pointerId });
    const onPointerCancel = (event: PointerEvent) => dispatch({ type: 'PointerCancel', pointerId: event.pointerId });
    const onLostCapture = (event: PointerEvent) => dispatch({ type: 'LostCapture', pointerId: event.pointerId });
    const onWheel = (event: WheelEvent) => {
      if (dispatch({ type: 'Wheel', dx: event.deltaX, dy: event.deltaY, ctrlKey: event.ctrlKey, metaKey: event.metaKey, point: local(event), at: Date.now() })) {
        event.preventDefault();
      }
    };
    const onGestureStart = (event: Event) => {
      event.preventDefault();
      dispatch({ type: 'GestureStart', point: local(event as GestureLike) });
    };
    const onGestureChange = (event: Event) => {
      event.preventDefault();
      dispatch({ type: 'GestureChange', scale: (event as GestureLike).scale ?? 1, point: local(event as GestureLike) });
    };
    const onGestureEnd = (event: Event) => {
      event.preventDefault();
      dispatch({ type: 'GestureEnd' });
    };
    const onBlur = () => dispatch({ type: 'Blur' });
    const onVisibility = () => {
      if (document.visibilityState === 'hidden') dispatch({ type: 'Hidden' });
    };
    const onKey = (event: KeyboardEvent) => {
      // The compose keyboard is the stage's only (10D): a key an open menu / dialog already took, or one pressed while
      // such a layer is open, is not the editor's (review r1 F2).
      if (event.defaultPrevented || document.querySelector(LAYER_OPEN)) return;
      if (isTyping(event.target) || event.metaKey || event.ctrlKey || event.altKey) return;
      const key = event.key.toLowerCase();
      if (key === 'escape') {
        event.preventDefault();
        dispatch({ type: 'KeyEsc' });
      } else if (key === 'enter') {
        event.preventDefault();
        dispatch({ type: 'KeyReturn' });
      } else if (SWALLOWED.has(key)) {
        event.preventDefault();
      }
    };
    // The webview's page zoom is off while composing (10D): WebKit reports a pinch outside the stage as a gesture.
    const pageZoomOff = (event: Event) => event.preventDefault();

    element.addEventListener('pointerdown', onPointerDown);
    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
    window.addEventListener('pointercancel', onPointerCancel);
    element.addEventListener('lostpointercapture', onLostCapture);
    element.addEventListener('wheel', onWheel, { passive: false });
    element.addEventListener('gesturestart', onGestureStart);
    element.addEventListener('gesturechange', onGestureChange);
    element.addEventListener('gestureend', onGestureEnd);
    window.addEventListener('blur', onBlur);
    document.addEventListener('visibilitychange', onVisibility);
    window.addEventListener('keydown', onKey);
    document.addEventListener('gesturestart', pageZoomOff);
    return () => {
      element.removeEventListener('pointerdown', onPointerDown);
      window.removeEventListener('pointermove', onPointerMove);
      window.removeEventListener('pointerup', onPointerUp);
      window.removeEventListener('pointercancel', onPointerCancel);
      element.removeEventListener('lostpointercapture', onLostCapture);
      element.removeEventListener('wheel', onWheel);
      element.removeEventListener('gesturestart', onGestureStart);
      element.removeEventListener('gesturechange', onGestureChange);
      element.removeEventListener('gestureend', onGestureEnd);
      window.removeEventListener('blur', onBlur);
      document.removeEventListener('visibilitychange', onVisibility);
      window.removeEventListener('keydown', onKey);
      document.removeEventListener('gesturestart', pageZoomOff);
      // Leaving the editor mid-gesture (a capture arrived and took the stage) commits what was on screen (rule 4).
      const state = stateRef.current;
      if (state && state.phase.kind !== 'idle') {
        for (const effect of reduce(state, { type: 'Blur' }).effects) {
          if (effect.type === 'Save') latest.current.onSave(effect.photoId, effect.values);
        }
      }
      stateRef.current = null;
      snapshotRef.current = null;
      if (timer.current !== null) window.clearInterval(timer.current);
      timer.current = null;
      if (glideTimer.current !== null) window.clearTimeout(glideTimer.current);
      glideTimer.current = null;
      if (raf.current !== null) cancelAnimationFrame(raf.current);
      raf.current = null;
      pending.current = null;
    };
  }, [dispatch]);

  const isIdle = useCallback(() => (stateRef.current?.phase.kind ?? 'idle') === 'idle', []);
  const subscribe = useCallback((listener: () => void) => {
    listeners.current.add(listener);
    return () => {
      listeners.current.delete(listener);
    };
  }, []);
  const getSnapshot = useCallback(() => snapshotRef.current, []);
  return { dispatch, isIdle, subscribe, getSnapshot };
}
