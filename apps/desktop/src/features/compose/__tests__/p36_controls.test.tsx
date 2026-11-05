import { compose, composition, layoutStage, openWorkspace, photo, pointer, savedValues, twoPhotos, view } from './harness';
import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { setComposition } from '@/services/compositions';
import { beforeEach, describe, expect, it, vi } from 'vitest';

beforeEach(() => { window.localStorage.removeItem('compose.snap'); window.localStorage.removeItem('tools.hidden'); });
const start = async () => { layoutStage(); await openWorkspace([twoPhotos()]); await compose(); };
const bar = () => document.querySelector('.stage-bar')!;
const click = async (scope: ParentNode, name: string) => act(async () => { fireEvent.click(Array.from(scope.querySelectorAll('button')).find(button => button.getAttribute('aria-label') === name || button.textContent?.trim() === name)!); });

describe('P36 editor controls', () => {
  it('C5(a) gives every editor shortcut its named control and exact hint', async () => {
    await start();
    const entries: [string, string, string, string][] = [
      ['.tools-panel', 'button', 'Normal', 'Normal (S)'], ['.tools-panel', 'button', 'Split', 'Split (S)'],
      ['.tools-panel', 'combobox', 'Shape', 'Shape (1–6, 0)'], ['.tools-panel', 'button', 'Turn frame', 'Turn frame (X)'],
      ['.tools-panel', 'button', 'Rotate left', 'Rotate 90° left'], ['.tools-panel', 'button', 'Rotate right', 'Rotate 90° right'],
      ['.tools-panel', 'button', 'Flip', 'Flip horizontally'], ['.tools-panel', 'slider', 'Straighten', 'Straighten (double-click to level)'],
      ['.tools-panel', 'textbox', 'Straighten degrees', 'Straighten in degrees (↑ ↓ 0.1°, ⇧ 1°)'],
      ['.tools-panel', 'switch', 'Straightening grid', 'Straightening grid'], ['.tools-panel', 'combobox', 'Overlay', 'Overlay (O)'],
      ['.tools-panel', 'button', 'Nudge left', 'Nudge left (←)'], ['.tools-panel', 'button', 'Nudge right', 'Nudge right (→)'],
      ['.tools-panel', 'button', 'Nudge up', 'Nudge up (↑)'], ['.tools-panel', 'button', 'Nudge down', 'Nudge down (↓)'],
      ['.tools-panel', 'button', 'Centre', 'Centre (.)'], ['.tools-panel', 'button', 'Reset', 'Reset (⌘⌥R)'],
      ['.tools-panel', 'button', 'Done', 'Done (Return)'],
      ['.stage-bar', 'button', 'Revert', 'Revert (Esc)'], ['.stage-bar', 'button', 'Loupe', 'Loupe (hold Z)'],
      ['.stage-bar', 'button', 'Snapping', 'Snapping (hold ⌥ to suspend)'], ['.stage-bar', 'button', 'Copy framing', 'Copy framing (⌥⌘C)'],
      ['.stage-bar', 'button', 'Paste framing', 'Paste framing (⌥⌘V)'], ['.stage-bar', 'button', 'Tools panel', 'Tools panel (⌘T)'],
      ['.stage-bar', 'button', 'Frame meter', 'Frame meter (⌥⌘P)'], ['.stage-bar', 'button', 'Done', 'Done (Return)'],
      ['.stage-body', 'button', 'Previous photo', 'Previous photo ([)'], ['.stage-body', 'button', 'Next photo', 'Next photo (])'],
      ['.folded-strip', 'button', 'Unfold panel', 'Unfold panel (F)'],
    ];
    for (const [place, role, name, hint] of entries) {
      const scope = document.querySelector(place)!;
      const control = within(scope as HTMLElement).getByRole(role, { name });
      await act(async () => { (document.activeElement as HTMLElement)?.blur(); control.focus(); });
      await waitFor(() => expect(screen.getByRole('tooltip').textContent?.trim()).toBe(hint));
    }
    fireEvent.keyDown(window, { key: 'f' });
    const fold = screen.getByRole('button', { name: 'Fold panel' });
    await act(async () => { (document.activeElement as HTMLElement)?.blur(); fold.focus(); });
    await waitFor(() => expect(screen.getByRole('tooltip').textContent?.trim()).toBe('Fold panel (F)'));
  });
  it('C5(b) keeps unavailable Paste hintable, inert, then enables it after Copy', async () => {
    await start();
    const paste = screen.getByRole('button', { name: 'Paste framing' });
    expect(paste).toHaveAttribute('aria-disabled', 'true');
    await act(async () => paste.focus());
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Paste framing (⌥⌘V)');
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
  it('C6(f) Tools panel button toggles the panel and remembered state', async () => {
    await start(); await click(bar(), 'Tools panel');
    expect(document.querySelector('.tools-panel')).toHaveAttribute('hidden');
    expect(screen.getByRole('button', { name: 'Tools panel' })).toHaveAttribute('aria-pressed', 'false');
    expect(window.localStorage.getItem('tools.hidden')).toBe('1');
    await click(bar(), 'Tools panel'); expect(document.querySelector('.tools-panel')).not.toHaveAttribute('hidden');
    expect(screen.getByRole('button', { name: 'Tools panel' })).toHaveAttribute('aria-pressed', 'true');
  });
  it('C6(g) Frame meter button toggles the meter', async () => {
    await start();
    await click(bar(), 'Frame meter');
    expect(document.querySelector('.frame-meter')).not.toBeNull();
    expect(screen.getByRole('button', { name: 'Frame meter' })).toHaveAttribute('aria-pressed', 'true');
    await click(bar(), 'Frame meter');
    expect(document.querySelector('.frame-meter')).toBeNull();
    expect(screen.getByRole('button', { name: 'Frame meter' })).toHaveAttribute('aria-pressed', 'false');
  });
  it('C6(h) the nudge pad follows arrow step sizes', async () => {
    await start();
    await click(document.querySelector('.compose-nudge')!, 'Nudge right');
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
