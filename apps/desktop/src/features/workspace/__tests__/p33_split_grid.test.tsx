// The compose harness registers the service mocks; it must be evaluated before this file's own service imports.
import { composition, openWorkspace, photo, tile, view } from '@/features/compose/__tests__/harness';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { setComposition, type Rect } from '@/services/compositions';
import { reorderPhoto, classifyPhoto } from '@/services/photos';

const near = (actual: number, expected: number, tolerance = 1e-6) => expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance);

/** The split default of the 2560×1707 frame (P33 C1(a)(1)): w_px = sqrt(1707²·2) centred. */
const SPLIT_W_PX = Math.sqrt(1707 * 1707 * 2);
const SPLIT: Rect = { x: 0.5 - SPLIT_W_PX / 2560 / 2, y: 0.5 - SPLIT_W_PX / 2 / 1707 / 2, w: SPLIT_W_PX / 2560, h: SPLIT_W_PX / 2 / 1707 };
const split = (id: number) => composition(id, { kind: 'split', rect: SPLIT, origin: 'user', revision: 1 });

/** The plan's workspace: photo 1 (main, split), 2 (main), 3 (main, split), unreviewed 4; condition 5 (split), 6. */
const workspace = () =>
  view([
    photo(1, 'main', { position: 1, composition: split(1) }),
    photo(2, 'main', { position: 2 }),
    photo(3, 'main', { position: 3, composition: split(3) }),
    photo(4, 'unreviewed'),
    photo(5, 'condition', { position: 1, composition: split(5) }),
    photo(6, 'condition', { position: 2 }),
  ]);

const region = (name: string) => screen.getByRole('region', { name });
const numbers = (scope: ParentNode) => [...scope.querySelectorAll('.tile-number')].map(node => node.textContent);
const order = (scope: ParentNode) => [...scope.querySelectorAll('[data-photo-id]')].map(node => Number(node.getAttribute('data-photo-id')));
const panelTile = (id: number) => tile(id, document.querySelector('.photo-panel, .folded-strip')!);

/** Lays out tiles for dnd-kit, which measures with getBoundingClientRect (zero everywhere in jsdom). */
function layoutTiles(rects: Record<number, [number, number]>, size: [number, number] = [100, 75]) {
  vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function (this: Element) {
    for (const [id, [left, top]] of Object.entries(rects)) {
      if (this.matches(`.photo-panel [data-photo-id="${id}"]`)) {
        const [width, height] = size;
        return { left, top, width, height, right: left + width, bottom: top + height, x: left, y: top, toJSON: () => ({}) } as DOMRect;
      }
    }
    return { left: 0, top: 0, width: 0, height: 0, right: 0, bottom: 0, x: 0, y: 0, toJSON: () => ({}) } as DOMRect;
  });
}

/** Picks a tile up and holds it over `to` (P22 harness); `release()` drops it there. */
async function hold(from: Element, to: [number, number]) {
  const start = from.getBoundingClientRect();
  const x = start.left + start.width / 2;
  const y = start.top + start.height / 2;
  fireEvent.pointerDown(from, { button: 0, isPrimary: true, clientX: x, clientY: y });
  await act(async () => { fireEvent.pointerMove(document, { clientX: x + 10, clientY: y + 10 }); });
  await act(async () => { fireEvent.pointerMove(document, { clientX: to[0], clientY: to[1] }); });
  return { release: () => act(() => { fireEvent.pointerUp(document, { clientX: to[0], clientY: to[1] }); }) };
}

const overlay = () => document.querySelector('.drag-overlay')!;
const menuItem = (name: string) => screen.findByRole('menuitem', { name });
const openStageMenu = () => fireEvent.pointerDown(screen.getByRole('button', { name: 'Photo actions' }), { button: 0, ctrlKey: false });

describe('C4 the grid, the strip, the drag overlay and the stage bar count outputs', () => {
  it('C4(a) tiles read 01–02, 03, 04–05, 06 in Main and 01–02, 03 in Condition; split tiles carry a middle hairline; the Cover badge stays on tile 1', async () => {
    await openWorkspace([workspace()]);
    const main = region('Main images');
    const condition = region('Condition');
    expect(order(main)).toEqual([1, 2, 3, 4]);
    expect(numbers(main)).toEqual(['01–02', '03', '04–05', '06']);
    expect(order(condition)).toEqual([5, 6]);
    expect(numbers(condition)).toEqual(['01–02', '03']);
    for (const id of [1, 3, 5]) {
      const hairline = panelTile(id).querySelector<HTMLElement>('.tile-split')!;
      expect(hairline).not.toBeNull();
      // At the middle of the composition rect: (x + w/2) · 100 % = 50 % for the centred split default.
      expect(hairline.style.left).toBe(`${Number(((SPLIT.x + SPLIT.w / 2) * 100).toFixed(4))}%`);
      expect(hairline.style.left).toBe('50%');
      expect(hairline.style.top).toBe(`${Number((SPLIT.y * 100).toFixed(4))}%`);
      expect(hairline.style.height).toBe(`${Number((SPLIT.h * 100).toFixed(4))}%`);
      expect(panelTile(id).querySelector('.tile-frame')).not.toBeNull();
    }
    for (const id of [2, 4, 6]) expect(panelTile(id).querySelector('.tile-split')).toBeNull();
    expect(panelTile(1).getAttribute('aria-label')).toBe('Photo 01–02');
    expect(panelTile(3).getAttribute('aria-label')).toBe('Photo 04–05');
    expect(panelTile(4).getAttribute('aria-label')).toBe('Photo 06, to review');
    expect(panelTile(5).getAttribute('aria-label')).toBe('Photo 01–02');
    const covers = [...document.querySelectorAll('.photo-panel .tile-cover')].map(node => node.closest('[data-photo-id]')!.getAttribute('data-photo-id'));
    expect(covers).toEqual(['1']);
  });

  it('C4(b) the folded strip keeps the order and the split hairlines, without numbers', async () => {
    await openWorkspace([workspace()]);
    await act(async () => { fireEvent.keyDown(window, { key: 'f' }); });
    const strip = document.querySelector('.folded-strip')!;
    expect(strip).not.toBeNull();
    expect(order(strip)).toEqual([1, 2, 3, 4, 5, 6]);
    expect(strip.querySelectorAll('.tile-number').length).toBe(0);
    const withHairline = [...strip.querySelectorAll('[data-photo-id]')].filter(node => node.querySelector('.tile-split')).map(node => Number(node.getAttribute('data-photo-id')));
    expect(withHairline).toEqual([1, 3, 5]);
  });

  it('C4(c) the held overlay of a split tile reads 03–04 over slot 2, a normal tile over slot 3 reads 05; the drops reorder by photo slot as before', async () => {
    await openWorkspace([workspace()]);
    layoutTiles({ 1: [0, 0], 2: [110, 0], 3: [0, 85], 4: [110, 85], 5: [0, 300], 6: [110, 300] });
    const main = () => region('Main images');
    expect(order(main())).toEqual([1, 2, 3, 4]);

    const first = await hold(panelTile(3), [160, 37]);
    expect(overlay().querySelector('.tile-number')).toHaveTextContent('03–04');
    expect(overlay().querySelector('.tile-split')).not.toBeNull();
    expect(overlay().querySelector('.tile-cover')).toBeNull();
    await first.release();
    await waitFor(() => expect(reorderPhoto).toHaveBeenCalledWith(3, 2));
    expect(classifyPhoto).not.toHaveBeenCalled();
    // The command reloads the (unchanged) view: back to the stored order before the second drag.
    await waitFor(() => expect(order(main())).toEqual([1, 2, 3, 4]));

    const second = await hold(panelTile(2), [50, 122]);
    expect(overlay().querySelector('.tile-number')).toHaveTextContent('05');
    expect(overlay().querySelector('.tile-split')).toBeNull();
    await second.release();
    await waitFor(() => expect(reorderPhoto).toHaveBeenCalledWith(2, 3));
    expect(reorderPhoto).toHaveBeenCalledTimes(2);
  });

  it('C4(d) the stage bar reads the places the active photo fills: Main 4–5 of 6 for photo 3, Main 1–2 of 6 for photo 1, Condition 3 of 3 for photo 6', async () => {
    await openWorkspace([workspace()]);
    const place = () => document.querySelector('.stage-place')!;
    await act(async () => { fireEvent.click(panelTile(3)); });
    expect(place()).toHaveTextContent('Main 4–5 of 6');
    await act(async () => { fireEvent.click(panelTile(1)); });
    expect(place()).toHaveTextContent('Main 1–2 of 6');
    await act(async () => { fireEvent.click(panelTile(2)); });
    expect(place()).toHaveTextContent('Main 3 of 6');
    await act(async () => { fireEvent.click(panelTile(6)); });
    expect(place()).toHaveTextContent('Condition 3 of 3');
    await act(async () => { fireEvent.click(panelTile(5)); });
    expect(place()).toHaveTextContent('Condition 1–2 of 3');
  });
});

describe('C5 Split and Normal from the menus', () => {
  it('C5(a) the ⋯ menu of tile 2 (normal) offers Split: one save with kind split and the re-fitted rect; the tiles renumber in place; a split photo\'s menu offers Normal, not Split', async () => {
    await openWorkspace([workspace()]);
    await act(async () => { fireEvent.click(panelTile(2)); });
    openStageMenu();
    fireEvent.click(await menuItem('Split S'));
    await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
    const [photoId, values] = vi.mocked(setComposition).mock.calls[0];
    expect(photoId).toBe(2);
    expect(values).toMatchObject({ kind: 'split', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0 });
    near(values.rect.x, SPLIT.x, 1e-9);
    near(values.rect.y, SPLIT.y, 1e-9);
    near(values.rect.w, SPLIT.w, 1e-9);
    near(values.rect.h, SPLIT.h, 1e-9);
    const main = region('Main images');
    await waitFor(() => expect(numbers(main)).toEqual(['01–02', '03–04', '05–06', '07']));
    expect(panelTile(2).querySelector('.tile-split')).not.toBeNull();
    expect(document.querySelector('.stage-place')).toHaveTextContent('Main 3–4 of 7');

    fireEvent.contextMenu(panelTile(1));
    expect(await menuItem('Normal S')).not.toBeNull();
    expect(screen.queryByRole('menuitem', { name: 'Split S' })).toBeNull();
    await act(async () => { fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Escape' }); });
    // An unreviewed photo cannot be split (10D: Main and Condition photos).
    fireEvent.contextMenu(panelTile(4));
    expect((await menuItem('Split S')).getAttribute('aria-disabled')).toBe('true');
  });

  it('C5(b) with tiles 2 and 3 selected, Split from the right-click menu saves photo 2 only (3 is already split); both read split labels', async () => {
    await openWorkspace([workspace()]);
    await act(async () => { fireEvent.click(panelTile(2), { metaKey: true }); });
    await act(async () => { fireEvent.click(panelTile(3), { metaKey: true }); });
    expect(panelTile(2).dataset.selected).toBe('true');
    expect(panelTile(3).dataset.selected).toBe('true');
    fireEvent.contextMenu(panelTile(2));
    fireEvent.click(await menuItem('Split S'));
    await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
    expect(vi.mocked(setComposition).mock.calls[0][0]).toBe(2);
    const main = region('Main images');
    await waitFor(() => expect(numbers(main)).toEqual(['01–02', '03–04', '05–06', '07']));
    await act(async () => {});
    expect(setComposition).toHaveBeenCalledTimes(1);
  });

  it('C5(c) a condition photo splits the same way: setComposition(6, kind split); Condition reads 01–02, 03–04', async () => {
    await openWorkspace([workspace()]);
    fireEvent.contextMenu(panelTile(6));
    fireEvent.click(await menuItem('Split S'));
    await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
    expect(vi.mocked(setComposition).mock.calls[0][0]).toBe(6);
    expect(vi.mocked(setComposition).mock.calls[0][1].kind).toBe('split');
    const condition = region('Condition');
    await waitFor(() => expect(numbers(condition)).toEqual(['01–02', '03–04']));
    expect(numbers(region('Main images'))).toEqual(['01–02', '03', '04–05', '06']);
  });
});
