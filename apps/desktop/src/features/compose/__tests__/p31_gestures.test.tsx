// The harness registers the service mocks; it must be evaluated before this file's own service imports.
import {
  captureFrames,
  compose,
  DEFAULT_RECT,
  fakeTimers,
  gesture,
  image,
  layoutStage,
  openWorkspace,
  pointer,
  saved,
  savedValues,
  stage,
  tile,
  transformOf,
  twoPhotos,
  wheel,
} from './harness';
import { act, cleanup, fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { toast } from 'sonner';
import { setComposition, type Composition, type CompositionValues } from '@/services/compositions';
import { getSessionView } from '@/services/photos';
import { ServiceError } from '@/services/sessions';

const PANNED_X = 0.11546683; // x − 50 / 977.81
const ZOOMED = { x: 0.297783626, y: 0.19673467, w: 0.404432748, h: 0.60653066 }; // exp(0.5) about the frame centre
const ZOOM_IN = Math.exp(0.5);
const near = (actual: number, expected: number, tolerance = 1e-6) => expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance);
const rectNear = (values: CompositionValues, expected: { x: number; y: number; w: number; h: number }, tolerance = 1e-6) => {
  near(values.rect.x, expected.x, tolerance);
  near(values.rect.y, expected.y, tolerance);
  near(values.rect.w, expected.w, tolerance);
  near(values.rect.h, expected.h, tolerance);
};
const placedAt = (x: number, y = 24) => {
  const placed = transformOf();
  expect(Math.abs(placed.x - x)).toBeLessThanOrEqual(0.5);
  expect(Math.abs(placed.y - y)).toBeLessThanOrEqual(0.5);
  expect(placed.deg).toBe(0);
};

/** Opens the two-photo workspace in Compose with the stage laid out and frames captured. */
async function editor() {
  layoutStage();
  await openWorkspace([twoPhotos()]);
  const frames = captureFrames();
  await compose();
  return frames;
}

/** A drag from (x0, 350) to (x1, y1) with one move, released inside an async act so the save settles. */
async function drag(x0: number, x1: number, y1 = 350, flush?: () => void) {
  pointer('pointerdown', x0, 350);
  pointer('pointermove', x1, y1);
  flush?.();
  await act(async () => {
    pointer('pointerup', x1, y1);
  });
}

/** A wheel settle: the Tick timer fires once 120 ms have passed without a wheel. */
const settle = () => act(async () => {
  vi.advanceTimersByTime(130);
});

describe('C3 dragging', () => {
  it('C3(a) a +50 px drag moves the image once per frame and saves the panned rect on release', async () => {
    const frames = await editor();
    pointer('pointerdown', 500, 350);
    const before = frames.calls();
    pointer('pointermove', 530, 350);
    pointer('pointermove', 550, 350);
    expect(frames.calls() - before).toBe(1);
    frames.flush();
    placedAt(61.09);
    await act(async () => {
      pointer('pointerup', 550, 350);
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(vi.mocked(setComposition).mock.calls[0][0]).toBe(1);
    const values = savedValues(0);
    near(values.rect.x, PANNED_X);
    expect(values.rect.y).toBe(0);
    expect(values.rect.w).toBe(0.666796875);
    expect(values.rect.h).toBe(1);
    expect(values).toMatchObject({ kind: 'normal', shape: '1:1', quarter_turns: 0, flip: 'none', straighten: 0 });
  });

  it('C3(b) the composition never passes the photo\'s edge: +400 px clamps x to 0, the opposite drag to 1 − w with y = 0', async () => {
    await editor();
    await drag(500, 900);
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(savedValues(0).rect.x).toBe(0);
    await drag(500, 100, 600);
    expect(setComposition).toHaveBeenCalledTimes(2);
    near(savedValues(1).rect.x, 0.333203125, 1e-9);
    expect(savedValues(1).rect.y).toBe(0);
  });
});

describe('C4 zooming', () => {
  it('C4(a) ctrl + wheel zooms about the pointer: the image grows, the settle saves the zoomed rect, the wheel is prevented', async () => {
    const frames = await editor();
    fakeTimers();
    const event = wheel({ x: 500, y: 350, ctrlKey: true, deltaY: -100 });
    expect(event.defaultPrevented).toBe(true);
    frames.flush();
    near(parseFloat(image()!.style.width), 1612, 1);
    await settle();
    expect(setComposition).toHaveBeenCalledTimes(1);
    rectNear(savedValues(0), ZOOMED);
  });

  it('C4(b) ⌘ + wheel behaves exactly like ctrl + wheel', async () => {
    const frames = await editor();
    fakeTimers();
    const event = wheel({ x: 500, y: 350, metaKey: true, deltaY: -100 });
    expect(event.defaultPrevented).toBe(true);
    frames.flush();
    near(parseFloat(image()!.style.width), 1612, 1);
    await settle();
    expect(setComposition).toHaveBeenCalledTimes(1);
    rectNear(savedValues(0), ZOOMED);
  });

  it('C4(c) WebKit gesture events pinch; the duplicate ctrl-wheel during the gesture is prevented and ignored', async () => {
    await editor();
    gesture('gesturestart', { x: 500, y: 350 });
    gesture('gesturechange', { x: 500, y: 350, scale: ZOOM_IN });
    const duplicate = wheel({ x: 500, y: 350, ctrlKey: true, deltaY: -100 });
    expect(duplicate.defaultPrevented).toBe(true);
    await act(async () => {
      gesture('gestureend');
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    rectNear(savedValues(0), ZOOMED);
  });

  it('C4(d) zooming out stops at the photo\'s edge (the default square); zooming in stops at the minimum rect', async () => {
    await editor();
    fakeTimers();
    wheel({ ctrlKey: true, deltaY: -100 });
    await settle();
    rectNear(savedValues(0), ZOOMED);
    wheel({ ctrlKey: true, deltaY: 2000 });
    await settle();
    expect(setComposition).toHaveBeenCalledTimes(2);
    rectNear(savedValues(1), DEFAULT_RECT, 1e-9);
    wheel({ ctrlKey: true, deltaY: -3000 });
    await settle();
    expect(setComposition).toHaveBeenCalledTimes(3);
    near(savedValues(2).rect.h, 0.05, 1e-9);
    near(savedValues(2).rect.w, 0.03334);
  });
});

describe('C5 swipes and the page', () => {
  it('C5(a) a two-finger swipe (wheel without modifier) moves the photo with the fingers and saves on settle', async () => {
    const frames = await editor();
    fakeTimers();
    const event = wheel({ deltaX: -50, deltaY: 0 });
    expect(event.defaultPrevented).toBe(true);
    frames.flush();
    placedAt(61.09);
    await settle();
    expect(setComposition).toHaveBeenCalledTimes(1);
    near(savedValues(0).rect.x, PANNED_X);
  });

  it('C5(b) the webview\'s page zoom is off only while composing', async () => {
    await editor();
    expect(gesture('gesturestart', { x: 10, y: 10 }, document.body).defaultPrevented).toBe(true);
    fireEvent.keyDown(window, { key: 'Enter' });
    await waitFor(() => expect(stage()).toBeNull());
    expect(gesture('gesturestart', { x: 10, y: 10 }, document.body).defaultPrevented).toBe(false);
  });

  it('C5(c) a ctrl-wheel during a drag is prevented and changes nothing: one save with the drag\'s rect', async () => {
    await editor();
    pointer('pointerdown', 500, 350);
    pointer('pointermove', 550, 350);
    const event = wheel({ ctrlKey: true, deltaY: -100 });
    expect(event.defaultPrevented).toBe(true);
    await act(async () => {
      pointer('pointerup', 550, 350);
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    near(savedValues(0).rect.x, PANNED_X);
    expect(savedValues(0).rect.w).toBe(0.666796875);
  });
});

describe('C6 interrupted gestures and Esc', () => {
  it('C6(a) Esc mid-drag snaps the photo back to the drag start, saves nothing and keeps the editor open', async () => {
    const frames = await editor();
    pointer('pointerdown', 500, 350);
    pointer('pointermove', 550, 350);
    frames.flush();
    placedAt(61.09);
    fireEvent.keyDown(window, { key: 'Escape' });
    frames.flush();
    placedAt(11.09);
    await act(async () => {
      pointer('pointerup', 550, 350);
    });
    expect(setComposition).not.toHaveBeenCalled();
    expect(stage()).not.toBeNull();
  });

  it('C6(b) pointercancel, window blur, page hidden and lost capture each commit the visible state and end the gesture', async () => {
    const interruptions: Record<string, () => void> = {
      pointercancel: () => pointer('pointercancel', 550, 350),
      blur: () => window.dispatchEvent(new Event('blur')),
      hidden: () => {
        Object.defineProperty(document, 'visibilityState', { value: 'hidden', configurable: true });
        fireEvent(document, new Event('visibilitychange'));
        Object.defineProperty(document, 'visibilityState', { value: 'visible', configurable: true });
      },
      lostpointercapture: () => fireEvent.lostPointerCapture(stage()!, { pointerId: 1 }),
    };
    for (const [name, interrupt] of Object.entries(interruptions)) {
      const frames = await editor();
      pointer('pointerdown', 500, 350);
      pointer('pointermove', 550, 350);
      await act(async () => {
        interrupt();
      });
      expect(setComposition, name).toHaveBeenCalledTimes(1);
      near(savedValues(0).rect.x, PANNED_X);
      frames.flush();
      placedAt(61.09);
      pointer('pointermove', 700, 350);
      frames.flush();
      placedAt(61.09);
      expect(setComposition, name).toHaveBeenCalledTimes(1);
      cleanup();
      vi.mocked(setComposition).mockClear();
    }
  });

  it('C6(c) a second, non-primary pointer never moves the photo: one save from pointer 1 alone', async () => {
    await editor();
    pointer('pointerdown', 500, 350);
    pointer('pointerdown', 600, 350, { pointerId: 2, isPrimary: false });
    pointer('pointermove', 700, 350, { pointerId: 2, isPrimary: false });
    pointer('pointerup', 700, 350, { pointerId: 2, isPrimary: false });
    pointer('pointermove', 550, 350);
    await act(async () => {
      pointer('pointerup', 550, 350);
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    near(savedValues(0).rect.x, PANNED_X);
  });

  it('C6(d) Esc while idle reverts to how the editor came to the photo: one more save with the entry values', async () => {
    const frames = await editor();
    await drag(500, 550, 350, frames.flush);
    expect(setComposition).toHaveBeenCalledTimes(1);
    frames.flush();
    placedAt(61.09);
    await act(async () => {
      fireEvent.keyDown(window, { key: 'Escape' });
    });
    expect(setComposition).toHaveBeenCalledTimes(2);
    near(savedValues(1).rect.x, 0.166601562);
    expect(savedValues(1).rect.y).toBe(0);
    frames.flush();
    placedAt(11.09);
    expect(stage()).not.toBeNull();
  });

  it('F2 (review r1) Esc that closes the ⋯ menu over the editor closes the menu only: no revert, one save, placement kept', async () => {
    const frames = await editor();
    await drag(500, 550, 350, frames.flush);
    expect(setComposition).toHaveBeenCalledTimes(1);
    frames.flush();
    placedAt(61.09);
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Photo actions' }), { button: 0, ctrlKey: false });
    const menu = await screen.findByRole('menu');
    await act(async () => {
      fireEvent.keyDown(menu, { key: 'Escape' });
    });
    await waitFor(() => expect(screen.queryByRole('menu')).toBeNull());
    expect(setComposition).toHaveBeenCalledTimes(1);
    frames.flush();
    placedAt(61.09);
    expect(stage()).not.toBeNull();
  });
});

describe('C7 saves', () => {
  it('C7(a) switching photos while a save is pending: each save lands on its own photo, no reload', async () => {
    let resolveP1!: (composition: Composition) => void;
    vi.mocked(setComposition).mockImplementationOnce(() => new Promise(done => { resolveP1 = done; }));
    const frames = await editor();
    const loads = vi.mocked(getSessionView).mock.calls.length;
    await drag(500, 550, 350, frames.flush);
    expect(setComposition).toHaveBeenCalledTimes(1);
    const browser = document.querySelector('.folded-strip')!;
    await act(async () => {
      fireEvent.click(tile(2, browser));
    });
    expect(image()?.querySelector('img')?.getAttribute('src')).toBe('asset:///previews/2/adjusted-1.jpg');
    frames.flush();
    placedAt(11.09);
    await drag(500, 550, 350, frames.flush);
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(vi.mocked(setComposition).mock.calls[0][0]).toBe(1);
    near(savedValues(0).rect.x, PANNED_X);
    expect(vi.mocked(setComposition).mock.calls[1][0]).toBe(2);
    near(savedValues(1).rect.x, PANNED_X);
    frames.flush();
    placedAt(61.09);
    await act(async () => {
      resolveP1(saved(1, savedValues(0), 1));
    });
    expect(tile(1, browser).querySelector<HTMLElement>('.composed-frame')!.style.left).toBe('-17.3166%');
    expect(tile(2, browser).querySelector<HTMLElement>('.composed-frame')!.style.left).toBe('-17.3166%');
    frames.flush();
    placedAt(61.09);
    expect(vi.mocked(getSessionView).mock.calls.length).toBe(loads);
  });

  it('C7(b) two quick drags: B waits behind A, A\'s answer leaves the stage on B while B is in flight, B\'s answer wins, exactly two calls', async () => {
    let resolveA!: (composition: Composition) => void;
    let resolveB!: (composition: Composition) => void;
    vi.mocked(setComposition)
      .mockReturnValueOnce(new Promise(done => { resolveA = done; }))
      .mockReturnValueOnce(new Promise(done => { resolveB = done; }));
    const frames = await editor();
    await drag(500, 550, 350, frames.flush);
    await drag(500, 550, 350, frames.flush);
    // One save in flight per photo: B is queued behind A, and the stage shows B (+100 px).
    expect(setComposition).toHaveBeenCalledTimes(1);
    const a = savedValues(0);
    near(a.rect.x, PANNED_X);
    frames.flush();
    placedAt(111.09);
    await act(async () => {
      resolveA(saved(1, a, 1));
    });
    // A settled and B was sent: the stage must still show B, not snap back to A (review r1 F1).
    expect(setComposition).toHaveBeenCalledTimes(2);
    const b = savedValues(1);
    near(b.rect.x, DEFAULT_RECT.x - 2 * (DEFAULT_RECT.x - PANNED_X));
    frames.flush();
    placedAt(111.09);
    await act(async () => {
      resolveB(saved(1, b, 2));
    });
    const browser = document.querySelector('.folded-strip')!;
    await waitFor(() => expect(tile(1, browser).querySelector<HTMLElement>('.composed-frame')!.style.left).toBe('-9.6479%'));
    frames.flush();
    placedAt(111.09);
    expect(setComposition).toHaveBeenCalledTimes(2);
  });

  it('C7(c) a refused save tells the photographer, snaps the view back to the stored rect and keeps the editor open', async () => {
    vi.mocked(setComposition).mockRejectedValueOnce(new ServiceError({ code: 'composition_invalid', message: 'off_shape', data: { reason: 'off_shape' } }));
    const frames = await editor();
    await drag(500, 550, 350, frames.flush);
    expect(toast.error).toHaveBeenCalledWith('Not saved: off_shape');
    frames.flush();
    placedAt(11.09);
    expect(stage()).not.toBeNull();
  });
});
