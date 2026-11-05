import { compose, layoutStage, openWorkspace, savedValues, twoPhotos } from './harness';
import { act, fireEvent } from '@testing-library/react';
import { setComposition } from '@/services/compositions';
import { beforeEach, describe, expect, it, vi } from 'vitest';

beforeEach(() => window.localStorage.removeItem('compose.straightenGrid'));
const field = () => document.querySelector<HTMLInputElement>('#compose-straighten-degrees')!;
const slider = () => document.querySelector<HTMLInputElement>('#compose-straighten')!;
const start = async () => { layoutStage(); await openWorkspace([twoPhotos()]); await compose(); };
const press = (target: Element | Window, key: string, shiftKey = false) => fireEvent.keyDown(target, { key, shiftKey });

describe('P36 straightening', () => {
  it('C3(a) shows the screen grid only while straightening and remembers its switch', async () => {
    await start();
    expect(document.querySelector('.compose-straighten-grid')).toBeNull();
    act(() => field().focus());
    expect(document.querySelector('.compose-straighten-grid')).not.toBeNull();
    const pattern = document.querySelector('.compose-overlay pattern')!;
    expect(pattern).toHaveAttribute('patternUnits', 'userSpaceOnUse');
    expect(pattern).toHaveAttribute('width', '24'); expect(pattern).toHaveAttribute('height', '24');
    fireEvent.blur(field()); expect(document.querySelector('.compose-straighten-grid')).toBeNull();
    fireEvent.pointerDown(slider()); expect(document.querySelector('.compose-straighten-grid')).not.toBeNull();
    fireEvent.pointerUp(slider()); expect(document.querySelector('.compose-straighten-grid')).toBeNull();
    fireEvent.click(document.querySelector('[role="switch"][aria-label="Straightening grid"]')!);
    expect(document.querySelector('[role="switch"][aria-label="Straightening grid"]')).toHaveAttribute('aria-checked', 'false');
    expect(window.localStorage.getItem('compose.straightenGrid')).toBe('0');
    act(() => field().focus()); expect(document.querySelector('.compose-straighten-grid')).toBeNull();
    await act(async () => press(window.document.activeElement!, 'Enter'));
    await act(async () => press(window, 'Enter'));
    await compose();
    expect(document.querySelector('[role="switch"][aria-label="Straightening grid"]')).toHaveAttribute('aria-checked', 'false');
  });
  it('C4(a) commits typed degrees with a centred re-fit', async () => {
    await start(); act(() => field().focus()); fireEvent.change(field(), { target: { value: '10' } });
    await act(async () => press(field(), 'Enter'));
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(vi.mocked(setComposition).mock.calls[0][0]).toBe(1);
    expect(savedValues(0).straighten).toBe(10);
    expect(savedValues(0).rect.h).toBeCloseTo(0.8632, 3);
    expect(savedValues(0).rect.x + savedValues(0).rect.w/2).toBeCloseTo(0.5, 6);
    expect(field().value).toBe('10.0'); expect(slider().value).toBe('10');
    expect(document.activeElement).not.toBe(field());
  });
  it('C4(b) steps by 0.1 or 1 while the field keeps the keys', async () => {
    await start(); act(() => field().focus());
    for (const [key, shift] of [['ArrowUp', false], ['ArrowUp', true], ['ArrowDown', false]] as const) await act(async () => press(field(), key, shift));
    expect(vi.mocked(setComposition).mock.calls.map(call => call[1].straighten)).toEqual([0.1, 1.1, 1]);
    for (const call of vi.mocked(setComposition).mock.calls) {
      expect(call[1].rect.x + call[1].rect.w/2).toBeCloseTo(0.5, 6);
      expect(call[1].rect.y + call[1].rect.h/2).toBeCloseTo(0.5, 6);
    }
    expect(document.activeElement).toBe(field()); expect(field().value).toBe('1.0');
  });
  it('C4(c) clamps, rejects bad text, accepts comma and cancels Escape without leaving Compose', async () => {
    await start();
    for (const [text, expected] of [['50', 45], ['abc', null], ['-3,5', -3.5]] as const) {
      act(() => field().focus()); fireEvent.change(field(), { target: { value: text } });
      await act(async () => press(field(), 'Enter'));
      if (expected !== null) expect(savedValues(vi.mocked(setComposition).mock.calls.length - 1).straighten).toBe(expected);
      if (text === 'abc') { expect(setComposition).toHaveBeenCalledTimes(1); expect(field().value).toBe('45.0'); }
    }
    expect(setComposition).toHaveBeenCalledTimes(2);
    act(() => field().focus()); fireEvent.change(field(), { target: { value: '7' } });
    await act(async () => press(field(), 'Escape'));
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(field().value).toBe('-3.5'); expect(document.activeElement).not.toBe(field());
    expect(document.querySelector('.compose-stage')).not.toBeNull();
  });
  it('C7(a) returns the keyboard to the editor after choosing Shape', async () => {
    await start();
    const shape = document.querySelector<HTMLSelectElement>('#compose-shape')!;
    act(() => shape.focus());
    fireEvent.change(shape, { target: { value: '4:5' } });
    await act(async () => press(document.activeElement!, 'x'));
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(savedValues(0).shape).toBe('4:5');
    expect(savedValues(1).orientation).toBe('turned');
    expect(document.activeElement).not.toBe(document.querySelector('#compose-shape'));
  });
  it('C7(b) returns the keyboard to the editor after dragging the slider', async () => {
    await start();
    act(() => slider().focus());
    fireEvent.pointerDown(slider()); fireEvent.input(slider(), { target: { value: '10' } });
    await act(async () => { fireEvent.change(slider(), { target: { value: '10' } }); fireEvent.pointerUp(slider()); });
    await act(async () => press(document.activeElement!, 'x'));
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(savedValues(0).straighten).toBe(10);
    expect(savedValues(1).orientation).toBe('turned');
    expect(document.activeElement).not.toBe(slider());
  });
});
