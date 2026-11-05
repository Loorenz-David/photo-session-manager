import { compose, composition, layoutStage, openWorkspace, photo, pointer, savedValues, twoPhotos, view } from './harness';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { setComposition } from '@/services/compositions';
import { beforeEach, describe, expect, it, vi } from 'vitest';

beforeEach(() => { window.localStorage.removeItem('compose.snap'); window.localStorage.removeItem('tools.hidden'); });
const start = async () => { layoutStage(); await openWorkspace([twoPhotos()]); await compose(); };
const bar = () => document.querySelector('.stage-bar')!;
const click = async (scope: ParentNode, name: string) => act(async () => { fireEvent.click(Array.from(scope.querySelectorAll('button')).find(button => button.getAttribute('aria-label') === name || button.textContent?.trim() === name)!); });

describe('P36 editor controls', () => {
  it('C5(b) keeps unavailable Paste hintable, inert, then enables it after Copy', async () => {
    await start();
    const paste = screen.getByRole('button', { name: 'Paste framing' });
    expect(paste).toHaveAttribute('aria-disabled', 'true');
    await act(async () => paste.focus());
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Paste framing · ⌥⌘V — Puts the copied composition on this photo, refitted to its frame');
    fireEvent.click(paste); expect(setComposition).not.toHaveBeenCalled();
    await click(bar(), 'Copy framing'); expect(paste).not.toHaveAttribute('aria-disabled');
  });
  it('C6(a) Revert restores the entry framing after a drag', async () => {
    await start(); pointer('pointerdown', 500, 350); pointer('pointermove', 550, 350); pointer('pointerup', 550, 350);
    await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
    expect(savedValues(0).rect.x).toBeCloseTo(0.115466, 5);
    await click(bar(), 'Revert');
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(savedValues(1).rect.x).toBeCloseTo(0.1666015625, 8);
  });
  it('C6(b) stage arrows step without wrapping', async () => {
    await start();
    await click(document.querySelector('.stage-body')!, 'Next photo');
    expect(document.querySelector('.compose-image img')?.getAttribute('src')).toBe('asset:///previews/2/adjusted-1.jpg');
    await click(document.querySelector('.stage-body')!, 'Next photo');
    expect(document.querySelector('.compose-image img')?.getAttribute('src')).toBe('asset:///previews/2/adjusted-1.jpg');
    await click(document.querySelector('.stage-body')!, 'Previous photo');
    expect(document.querySelector('.compose-image img')?.getAttribute('src')).toBe('asset:///previews/1/adjusted-1.jpg');
  });
  it('C6(c) Loupe toggles at the last pointer without saving', async () => {
    await start(); pointer('pointermove', 600, 300); await click(bar(), 'Loupe');
    expect(screen.getByRole('button', { name: 'Loupe' })).toHaveAttribute('aria-pressed', 'true');
    expect(document.querySelector<HTMLElement>('.compose-loupe')?.style.left).toBe('616px');
    expect(document.querySelector<HTMLElement>('.compose-loupe')?.style.top).toBe('316px');
    await click(bar(), 'Loupe'); expect(document.querySelector('.compose-loupe')).toBeNull();
    expect(screen.getByRole('button', { name: 'Loupe' })).toHaveAttribute('aria-pressed', 'false');
    expect(setComposition).not.toHaveBeenCalled();
  });
  it('C6(d) Snapping switch persists and controls the drag guide', async () => {
    await start(); const button = screen.getByRole('button', { name: 'Snapping' });
    expect(button).toHaveAttribute('aria-pressed', 'true');
    await click(bar(), 'Snapping'); expect(button).toHaveAttribute('aria-pressed', 'false');
    expect(window.localStorage.getItem('compose.snap')).toBe('0');
    pointer('pointerdown', 500, 350); pointer('pointermove', 503.7, 350);
    expect(document.querySelector('.compose-guide')).toBeNull();
    pointer('pointerup', 503.7, 350);
    await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
    expect(savedValues(0).rect.x).toBeCloseTo(0.162818, 5);
    await click(bar(), 'Revert');
    await click(bar(), 'Snapping'); expect(button).toHaveAttribute('aria-pressed', 'true');
    pointer('pointerdown', 500, 350); pointer('pointermove', 503.7, 350);
    expect(document.querySelector('.compose-guide[data-axis="x"]')).not.toBeNull();
  });
  it('C6(e) Copy and Paste transfer the framing to the next photo', async () => {
    layoutStage();
    const first = photo(1, 'main', { position: 1 });
    first.composition = composition(1, { shape: '4:5', straighten: 2, rect: { x: 0.3, y: 0.18, w: 0.4, h: 0.48 } });
    const second = photo(2, 'main', { position: 2 });
    second.composition = composition(2, { frame: { w: 1707, h: 2560 }, rect: { x: 0, y: 0.1666, w: 1, h: 0.6668 } });
    await openWorkspace([view([first, second])]); await compose();
    await click(bar(), 'Copy framing');
    await click(document.querySelector('.stage-body')!, 'Next photo');
    await click(bar(), 'Paste framing');
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(vi.mocked(setComposition).mock.calls[0][0]).toBe(2);
    expect(savedValues(0)).toMatchObject({ kind: 'normal', shape: '4:5', straighten: 2, orientation: 'named', quarter_turns: 0, flip: 'none' });
    expect(savedValues(0).rect.x).toBeCloseTo(0.26002, 4);
    expect(savedValues(0).rect.y).toBeCloseTo(0.21998, 4);
    expect(savedValues(0).rect.w).toBeCloseTo(0.47995, 4);
    expect(savedValues(0).rect.h).toBeCloseTo(0.40004, 4);
  });
  it('C6(h) the nudge pad follows arrow step sizes', async () => {
    await start();
    await click(document.querySelector('.composition-section')!, 'Nudge');
    await click(document.querySelector('#compose-nudge-pad')!, 'Nudge right');
    await act(async () => fireEvent.click(screen.getByRole('button', { name: 'Nudge right' }), { shiftKey: true }));
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(savedValues(0).rect.x).toBeCloseTo(0.167624, 5);
    expect(savedValues(1).rect.x).toBeCloseTo(0.177852, 5);
    expect(savedValues(0).rect.y).toBe(0);
    expect(savedValues(1).rect.y).toBe(0);
  });
  it('C8(b) draws an edge guide on the edge reached by the drag', async () => {
    await start(); pointer('pointerdown', 500, 350); pointer('pointermove', 659, 350);
    const guide = document.querySelector('.compose-guide[data-axis="x"]');
    expect(guide).toHaveAttribute('x1', '174'); expect(guide).toHaveAttribute('x2', '174');
    pointer('pointerup', 659, 350);
    await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
    expect(savedValues(0).rect.x).toBe(0);
  });
});
