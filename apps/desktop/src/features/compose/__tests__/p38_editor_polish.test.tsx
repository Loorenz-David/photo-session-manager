import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, expect, it } from 'vitest';
import { compose, layoutStage, openWorkspace, savedValues, twoPhotos } from './harness';
import { setComposition } from '@/services/compositions';

beforeEach(() => { window.localStorage.removeItem('tools.hidden'); window.localStorage.removeItem('compose.snap'); window.localStorage.removeItem('compose.straightenGrid'); });
const start = async () => { layoutStage(); await openWorkspace([twoPhotos()]); await compose(); };
const button = (name: string) => screen.getByRole('button', { name });
const click = async (element: Element, shiftKey = false) => act(async () => { fireEvent.click(element, { shiftKey }); });
const open = async (name: string) => act(async () => { fireEvent.pointerDown(button(name), { button: 0, ctrlKey: false, pointerType: 'mouse' }); fireEvent.click(button(name)); });
const icon = (element: Element) => { expect(element.textContent?.trim()).toBe(''); expect(element.querySelectorAll('svg')).toHaveLength(1); };

it('C1(a) puts the edged split line at the frame middle only in Split', async () => {
  await start(); await act(async () => fireEvent.keyDown(window, { key: 's' }));
  const frame = document.querySelector('.compose-frame')!;
  const lines = Array.from(document.querySelectorAll('line.compose-middle-edge, line.compose-middle'));
  expect(lines.map(line => line.getAttribute('class'))).toEqual(['compose-middle-edge', 'compose-middle']);
  const x = Number(frame.getAttribute('x')) + Number(frame.getAttribute('width')) / 2;
  for (const [line, width] of lines.map((line, i) => [line, i === 0 ? '4' : '2'] as const)) {
    expect(Number(line.getAttribute('x1'))).toBeCloseTo(x, 6); expect(Number(line.getAttribute('x2'))).toBeCloseTo(x, 6);
    expect(Number(line.getAttribute('y1'))).toBe(Number(frame.getAttribute('y')));
    expect(Number(line.getAttribute('y2'))).toBe(Number(frame.getAttribute('y')) + Number(frame.getAttribute('height')));
    expect(line.getAttribute('stroke-width')).toBe(width);
  }
  await act(async () => fireEvent.keyDown(window, { key: 's' }));
  expect(document.querySelectorAll('line.compose-middle-edge, line.compose-middle')).toHaveLength(0);
});

it('C2(a) fills Loupe only while it is on', async () => {
  await start(); const control = button('Loupe');
  expect(control).toHaveAttribute('aria-pressed', 'false'); expect(control).not.toHaveAttribute('data-on'); expect(control.querySelector('svg')).toHaveAttribute('fill', 'none');
  await click(control); expect(control).toHaveAttribute('aria-pressed', 'true'); expect(control).toHaveAttribute('data-on'); expect(control.querySelector('svg')).toHaveAttribute('fill', 'currentColor');
  expect(setComposition).not.toHaveBeenCalled();
});
it('C2(b) fills Snapping by default and persists off', async () => {
  await start(); const control = button('Snapping');
  expect(control).toHaveAttribute('aria-pressed', 'true'); expect(control).toHaveAttribute('data-on'); expect(control.querySelector('svg')).toHaveAttribute('fill', 'currentColor');
  await click(control); expect(control).toHaveAttribute('aria-pressed', 'false'); expect(control).not.toHaveAttribute('data-on'); expect(control.querySelector('svg')).toHaveAttribute('fill', 'none');
  expect(window.localStorage.getItem('compose.snap')).toBe('0');
});
it('C2(c) fills the grid by default and persists off', async () => {
  await start(); const control = screen.getByRole('switch', { name: 'Straightening grid' });
  expect(control).toHaveAttribute('aria-checked', 'true'); expect(control).toHaveAttribute('data-on'); expect(control.querySelector('svg')).toHaveAttribute('fill', 'currentColor');
  await click(control); expect(control).toHaveAttribute('aria-checked', 'false'); expect(control).not.toHaveAttribute('data-on'); expect(control.querySelector('svg')).toHaveAttribute('fill', 'none');
  expect(window.localStorage.getItem('compose.straightenGrid')).toBe('0');
});
it('C3(a) opens three named rotate menu entries from one icon', async () => {
  await start(); const control = button('Rotate and flip'); icon(control);
  expect(control).toHaveAttribute('aria-haspopup', 'menu'); expect(control).toHaveAttribute('aria-expanded', 'false');
  expect(screen.queryByRole('menu')).toBeNull(); expect(document.querySelector('#compose-rotate-flip, .compose-drawer')).toBeNull();
  for (const name of ['Rotate left', 'Rotate right', 'Flip']) expect(screen.queryByRole('button', { name })).toBeNull();
  await open('Rotate and flip');
  expect(screen.getAllByRole('menu')).toHaveLength(1);
  expect(within(screen.getByRole('menu')).getAllByRole('menuitem').map(item => item.textContent?.trim())).toEqual(['Rotate 90° left', 'Rotate 90° right', 'Flip horizontal']);
});
it('C3(b) rotates the square and closes the menu', async () => {
  await start(); await open('Rotate and flip'); await click(screen.getByRole('menuitem', { name: 'Rotate 90° right' }));
  expect(setComposition).toHaveBeenCalledTimes(1); expect(savedValues(0).quarter_turns).toBe(1);
  for (const [actual, want] of [[savedValues(0).rect.x, 0], [savedValues(0).rect.y, 0.166602], [savedValues(0).rect.w, 1], [savedValues(0).rect.h, 0.666797]]) expect(Math.abs(actual - want)).toBeLessThan(1e-6);
  expect(screen.queryByRole('menu')).toBeNull(); expect(button('Rotate and flip')).toHaveAttribute('aria-expanded', 'false');
});
it('C4(a) opens the icon-only nudge pad in keyboard order', async () => {
  await start(); icon(button('Nudge')); expect(document.querySelector('#compose-nudge-pad, .compose-drawer')).toBeNull();
  for (const name of ['Nudge up', 'Nudge left', 'Nudge down', 'Nudge right']) expect(screen.queryByRole('button', { name })).toBeNull();
  await open('Nudge'); const pad = document.querySelector('#compose-nudge-pad[role="group"][aria-label="Nudge"]')!;
  expect(pad).toBeTruthy(); expect(pad).not.toHaveAttribute('role', 'dialog');
  const arrows = within(pad as HTMLElement).getAllByRole('button');
  expect(arrows.map(arrow => arrow.getAttribute('aria-label'))).toEqual(['Nudge up', 'Nudge left', 'Nudge down', 'Nudge right']); arrows.forEach(icon);
});
it('C4(b) keeps the pad open across nudges and clamps upward motion', async () => {
  await start(); await open('Nudge');
  for (const [name, shift] of [['Nudge right', false], ['Nudge right', true], ['Nudge left', false], ['Nudge up', false]] as const) {
    await click(button(name), shift); expect(document.querySelector('#compose-nudge-pad')).toBeTruthy();
  }
  expect(setComposition).toHaveBeenCalledTimes(3);
  for (const [i, x] of [0.167624, 0.177851, 0.176828].entries()) { expect(savedValues(i).rect.x).toBeCloseTo(x, 5); expect(savedValues(i).rect.y).toBe(0); }
});
it('C4(c) dismisses the pad on an outside pointerdown without saving', async () => {
  await start(); await open('Nudge'); await new Promise(resolve => setTimeout(resolve, 0));
  await act(async () => fireEvent.pointerDown(document.querySelector('.compose-stage')!, { button: 0, ctrlKey: false, pointerType: 'mouse' }));
  await waitFor(() => expect(document.querySelector('#compose-nudge-pad')).toBeNull()); expect(setComposition).not.toHaveBeenCalled();
});
it('C4(d) Escape closes the pad without reverting its saved nudge', async () => {
  await start(); await open('Nudge'); await click(button('Nudge right'));
  expect(setComposition).toHaveBeenCalledTimes(1);
  await act(async () => fireEvent.keyDown(document.activeElement || document.body, { key: 'Escape' }));
  expect(document.querySelector('#compose-nudge-pad')).toBeNull(); expect(setComposition).toHaveBeenCalledTimes(1);
});
it('C5(a) places the three icon controls after degrees on one row', async () => {
  await start(); const field = document.querySelector('#compose-straighten-degrees')!; const row = field.parentElement!;
  const controls = [screen.getByRole('switch', { name: 'Straightening grid' }), button('Rotate and flip'), button('Nudge')];
  expect(Array.from(row.children).slice(3)).toEqual(controls); controls.forEach(control => { icon(control); expect(control.parentElement).toBe(row); });
  expect(document.querySelector('.composition-section .compose-drawer, .composition-section .compose-drawer-toggle, .composition-section .compose-drawer-body')).toBeNull();
});
