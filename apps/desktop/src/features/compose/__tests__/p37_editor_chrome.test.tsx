import { compose, layoutStage, openWorkspace, savedValues, twoPhotos } from './harness';
import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, expect, it } from 'vitest';
import { setComposition } from '@/services/compositions';

beforeEach(() => window.localStorage.removeItem('tools.hidden'));
const start = async () => { layoutStage(); await openWorkspace([twoPhotos()]); await compose(); };
const btn = (name: string) => screen.getByRole('button', { name });
const click = async (name: string, shiftKey = false) => act(async () => fireEvent.click(btn(name), { shiftKey }));
const key = async (name: string, options = {}) => act(async () => fireEvent.keyDown(window, { key: name, ...options }));
const first = () => document.querySelector('main.workspace')!.firstElementChild!;
const names = (element: Element) => Array.from(element.querySelectorAll('button')).map(button => button.getAttribute('aria-label'));
const icon = (element: Element) => { expect(element.textContent?.trim()).toBe(''); expect(element.querySelectorAll('svg')).toHaveLength(1); };

it('C1(a) shows exactly five icon tools and no retired buttons', async () => {
  await start(); const toolbar = document.querySelector('.stage-bar .editor-toolbar')!;
  expect(names(toolbar)).toEqual(['Revert','Loupe','Snapping','Copy framing','Paste framing']);
  toolbar.querySelectorAll('button').forEach(icon);
  expect(btn('Loupe')).toHaveAttribute('aria-pressed','false'); expect(btn('Snapping')).toHaveAttribute('aria-pressed','true');
  expect(btn('Paste framing')).toHaveAttribute('aria-disabled','true');
  expect(screen.queryByRole('button',{name:'Frame meter'})).toBeNull(); expect(screen.queryByRole('button',{name:'Tools panel'})).toBeNull();
  expect(within(document.querySelector('.stage-bar') as HTMLElement).getByRole('button',{name:'Done'})).toBeTruthy();
});

it('C2(a) gives every editor control its exact hover card', async () => {
  await start();
  const rows: [string,string,string,string][] = [
    ['.tools-panel-head','button','Hide Tools panel','Hide Tools panel · ⌘T — Folds the panel into a rail on the left'],
    ['.composition-section','button','Normal','Normal · S — One frame, one file'],
    ['.composition-section','button','Split','Split · S — One 2:1 frame cut into two square files, Left and Right'],
    ['.composition-section','combobox','Shape','Shape · 1–6, 0 — Original, Square, 4:5, 3:2, 2:3, 16:9 or Free'],
    ['.composition-section','button','Turn frame','Turn frame · X — Portrait or landscape, same centre and area'],
    ['.composition-section','button','Rotate and flip','Rotate and flip — Quarter turns and a mirror for the photo'],
    ['.composition-section','button','Rotate left','Rotate 90° left — Turns the photo anticlockwise'],
    ['.composition-section','button','Rotate right','Rotate 90° right — Turns the photo clockwise'],
    ['.composition-section','button','Flip','Flip horizontally — Mirrors the photo left to right'],
    ['.composition-section','slider','Straighten','Straighten — Tilts the photo; double-click levels it'],
    ['.composition-section','textbox','Straighten degrees','Straighten degrees · ↑ ↓ — Typed degrees; ↑ ↓ step 0.1°, ⇧ 1°'],
    ['.composition-section','switch','Straightening grid','Straightening grid — A fine grid over the photo while straightening'],
    ['.composition-section','combobox','Overlay','Overlay · O — Guides inside the frame; O cycles them'],
    ['.composition-section','button','Nudge','Nudge · ← ↑ → ↓ — Moves the frame one screen pixel, ⇧ ×10'],
    ['.composition-section','button','Nudge up','Nudge up · ↑ — One screen pixel up, ⇧ ×10'],
    ['.composition-section','button','Nudge left','Nudge left · ← — One screen pixel left, ⇧ ×10'],
    ['.composition-section','button','Nudge down','Nudge down · ↓ — One screen pixel down, ⇧ ×10'],
    ['.composition-section','button','Nudge right','Nudge right · → — One screen pixel right, ⇧ ×10'],
    ['.composition-section','button','Centre','Centre · . — Centres the frame on the photo'],
    ['.composition-section','button','Reset','Reset · ⌘⌥R — Back to the default framing'],
    ['.composition-section','button','Done','Done · Return — Keeps the changes and leaves the editor'],
    ['.stage-bar','button','Revert','Revert · Esc — Back to how this photo was when the editor opened'],
    ['.stage-bar','button','Loupe','Loupe · Z — 100 % view under the pointer'],
    ['.stage-bar','button','Snapping','Snapping · ⌥ — Snaps to the centre and the image edges; hold ⌥ to suspend'],
    ['.stage-bar','button','Copy framing',"Copy framing · ⌥⌘C — Takes this photo's composition"],
    ['.stage-bar','button','Paste framing','Paste framing · ⌥⌘V — Puts the copied composition on this photo, refitted to its frame'],
    ['.stage-bar','button','Done','Done · Return — Keeps the changes and leaves the editor'],
    ['.stage-body','button','Previous photo','Previous photo · [ — The photo before, keeping the changes'],
    ['.stage-body','button','Next photo','Next photo · ] — The photo after, keeping the changes'],
    ['.folded-strip','button','Unfold panel','Unfold panel · F — Opens the full photo panel beside the editor'],
  ];
  const check = async ([place,role,name,expected]: [string,string,string,string]) => {
    const control = within(document.querySelector(place) as HTMLElement).getByRole(role,{name});
    await act(async () => { (document.activeElement as HTMLElement)?.blur(); control.focus(); });
    await waitFor(() => expect(screen.getByRole('tooltip').textContent?.replace(/\s+/g,' ').trim()).toBe(expected));
  };
  for (const row of rows.slice(0,6)) await check(row);
  await click('Rotate and flip'); for (const row of rows.slice(6,9)) await check(row);
  for (const row of rows.slice(9,14)) await check(row);
  await click('Nudge'); for (const row of rows.slice(14)) await check(row);
  await click('Hide Tools panel'); await check(['.tools-rail','button','Show Tools panel','Show Tools panel · ⌘T — Brings the Tools panel back']);
  await click('Show Tools panel'); await key('f'); await check(['.photo-panel','button','Fold panel','Fold panel · F — Folds the panel to the strip']);
});

it('C3(a) toggles the meter with its key and has no meter button', async () => {
  await start(); expect(screen.queryByRole('button',{name:'Frame meter'})).toBeNull();
  await key('p',{metaKey:true,altKey:true}); expect(document.querySelector('.frame-meter')).not.toBeNull();
  await key('p',{metaKey:true,altKey:true}); expect(document.querySelector('.frame-meter')).toBeNull();
});

it('C4(a) folds the panel into a one-button rail', async () => {
  await start(); expect(first()).toHaveClass('tools-panel'); expect(first().firstElementChild).toHaveClass('tools-panel-head');
  expect(first().firstElementChild).toHaveTextContent('Tools');
  expect(within(first().firstElementChild as HTMLElement).getByRole('button',{name:'Hide Tools panel'})).toBeTruthy();
  await click('Hide Tools panel'); expect(document.querySelector('.tools-panel')).toBeNull();
  expect(first()).toHaveClass('tools-rail'); expect(first().nextElementSibling).toHaveClass('stage-column');
  expect(names(first())).toEqual(['Show Tools panel']); expect(window.localStorage.getItem('tools.hidden')).toBe('1');
  expect(document.querySelector('.compose-stage')).not.toBeNull();
});

it('C4(b) shows the panel by button or key', async () => {
  await start(); await click('Hide Tools panel'); await click('Show Tools panel');
  expect(first()).toHaveClass('tools-panel'); expect(first().querySelector('#compose-shape')).not.toBeNull();
  expect(document.querySelector('.tools-rail')).toBeNull(); expect(window.localStorage.getItem('tools.hidden')).not.toBe('1');
  await key('t',{metaKey:true}); expect(first()).toHaveClass('tools-rail'); expect(window.localStorage.getItem('tools.hidden')).toBe('1');
  await key('t',{metaKey:true}); expect(first()).toHaveClass('tools-panel');
});

it('C4(c) remembers a hidden panel across editor entry but removes it on exit', async () => {
  window.localStorage.setItem('tools.hidden','1'); await start(); expect(first()).toHaveClass('tools-rail');
  expect(document.querySelector('.tools-panel')).toBeNull(); await key('Enter');
  expect(document.querySelector('.tools-panel')).toBeNull(); expect(document.querySelector('.tools-rail')).toBeNull();
  expect(first()).toHaveClass('stage-column');
});

it('C5(a) opens the rotate drawer and saves the turned square', async () => {
  await start(); expect(btn('Rotate and flip')).toHaveAttribute('aria-expanded','false'); expect(document.querySelector('#compose-rotate-flip')).toBeNull();
  for (const name of ['Rotate left','Rotate right','Flip']) expect(screen.queryByRole('button',{name})).toBeNull();
  await click('Rotate and flip'); expect(btn('Rotate and flip')).toHaveAttribute('aria-expanded','true');
  const body = document.querySelector('#compose-rotate-flip.compose-drawer-body')!;
  expect(names(body)).toEqual(['Rotate left','Rotate right','Flip']); body.querySelectorAll('button').forEach(icon);
  await click('Rotate right'); expect(setComposition).toHaveBeenCalledTimes(1); expect(savedValues(0).quarter_turns).toBe(1);
  for (const [actual,want] of [[savedValues(0).rect.x,0],[savedValues(0).rect.y,0.166602],[savedValues(0).rect.w,1],[savedValues(0).rect.h,0.666797]]) expect(Math.abs(actual-want)).toBeLessThan(1e-6);
});

it('C5(b) puts an icon Turn beside Shape and saves the unchanged square', async () => {
  await start(); const turn = btn('Turn frame'); icon(turn); expect(turn.parentElement).toBe(document.querySelector('#compose-shape')!.parentElement);
  await click('Turn frame'); expect(setComposition).toHaveBeenCalledTimes(1);
  expect(savedValues(0)).toMatchObject({orientation:'turned',shape:'1:1'});
  for (const [actual,want] of [[savedValues(0).rect.x,0.166601562],[savedValues(0).rect.y,0],[savedValues(0).rect.w,0.666796875],[savedValues(0).rect.h,1]]) expect(Math.abs(actual-want)).toBeLessThan(1e-6);
});

it('C6(a) puts an icon grid switch beside the degrees field and persists off', async () => {
  await start(); const grid = screen.getByRole('switch',{name:'Straightening grid'}); icon(grid);
  expect(grid.parentElement).toBe(document.querySelector('#compose-straighten-degrees')!.parentElement);
  expect(grid).toHaveAttribute('aria-checked','true'); await act(async () => fireEvent.click(grid));
  expect(grid).toHaveAttribute('aria-checked','false'); expect(window.localStorage.getItem('compose.straightenGrid')).toBe('0');
});

it('C7(a) opens the four-arrow pad in keyboard order', async () => {
  await start(); expect(btn('Nudge')).toHaveAttribute('aria-expanded','false'); expect(document.querySelector('#compose-nudge-pad')).toBeNull();
  for (const name of ['Nudge up','Nudge left','Nudge down','Nudge right']) expect(screen.queryByRole('button',{name})).toBeNull();
  await click('Nudge'); expect(btn('Nudge')).toHaveAttribute('aria-expanded','true');
  const pad = document.querySelector('#compose-nudge-pad[role="group"]')!;
  expect(names(pad)).toEqual(['Nudge up','Nudge left','Nudge down','Nudge right']); pad.querySelectorAll('button').forEach(icon);
});

it('C7(b) nudges in screen pixels and clamps at the image edge', async () => {
  await start(); await click('Nudge'); await click('Nudge right'); await click('Nudge right',true); await click('Nudge left'); await click('Nudge up');
  expect(setComposition).toHaveBeenCalledTimes(3);
  for (const [i,x] of [0.167624,0.177851,0.176828].entries()) { expect(Math.abs(savedValues(i).rect.x-x)).toBeLessThan(1e-5); expect(savedValues(i).rect.y).toBe(0); }
});

it('C8(a) keeps drawers open during one visit and closes them on reopen', async () => {
  await start(); await click('Rotate and flip'); await click('Nudge'); await key(']');
  await key('t',{metaKey:true}); await key('t',{metaKey:true}); await click('Composition'); await click('Composition');
  for (const name of ['Rotate and flip','Nudge']) expect(btn(name)).toHaveAttribute('aria-expanded','true');
  expect(document.querySelector('#compose-rotate-flip')).not.toBeNull(); expect(document.querySelector('#compose-nudge-pad')).not.toBeNull();
  await key('Enter'); await key('r');
  for (const name of ['Rotate and flip','Nudge']) expect(btn(name)).toHaveAttribute('aria-expanded','false');
  expect(document.querySelector('#compose-rotate-flip')).toBeNull(); expect(document.querySelector('#compose-nudge-pad')).toBeNull();
});
