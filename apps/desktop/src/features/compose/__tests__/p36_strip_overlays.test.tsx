import { compose, layoutStage, openWorkspace, tile, twoPhotos } from './harness';
import { act, fireEvent, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

beforeEach(() => window.localStorage.removeItem('compose.overlay'));
const key = async (value: string) => act(async () => { fireEvent.keyDown(window, { key: value }); });
const overlay = () => document.querySelector('.compose-overlay-guides')!;

describe('P36 strip and overlays', () => {
  it('C1(a) keeps one strip beside the stage and switches the editing photo', async () => {
    layoutStage(); await openWorkspace([twoPhotos()]); await compose();
    fireEvent.pointerMove(document.querySelector('main.workspace')!, { clientX: window.innerWidth - 5 });
    const strip = document.querySelector('main.workspace > .folded-strip')!;
    expect([...strip.querySelectorAll('[data-photo-id]')].map(t => t.getAttribute('data-photo-id'))).toEqual(['1', '2']);
    expect(document.querySelector('aside.photo-panel')).toBeNull();
    expect(document.querySelector('.compose-browser')).toBeNull();
    expect(document.querySelector('.compose-browser[data-open]')).toBeNull();
    await act(async () => fireEvent.click(tile(2, strip)));
    expect(document.querySelector('.compose-stage')).not.toBeNull();
    expect(document.querySelector('.compose-image img')?.getAttribute('src')).toBe('asset:///previews/2/adjusted-1.jpg');
  });
  it('C1(b) F and the panel button fold and unfold, then Return restores the prior panel', async () => {
    layoutStage(); await openWorkspace([twoPhotos()]); await compose();
    await key('f');
    expect(document.querySelector('main.workspace > .photo-panel')).not.toBeNull();
    expect(document.querySelector('.folded-strip')).toBeNull();
    await act(async () => fireEvent.click(tile(1, document.querySelector('.photo-panel')!)));
    expect(document.querySelector('.compose-image img')?.getAttribute('src')).toBe('asset:///previews/1/adjusted-1.jpg');
    fireEvent.click(document.querySelector('.photo-panel [aria-label="Fold panel"]')!);
    expect(document.querySelector('main.workspace > .folded-strip')).not.toBeNull();
    await key('Enter');
    expect(document.querySelector('.compose-stage')).toBeNull();
    expect(document.querySelector('.photo-panel')).not.toBeNull();
    fireEvent.click(document.querySelector('.photo-panel [aria-label="Fold panel"]')!);
    await compose(); await key('f'); await key('Enter');
    expect(document.querySelector('.folded-strip')).not.toBeNull();
  });
  it('C2(b) cycles overlays, stores a choice, restores it, and repairs an invalid value', async () => {
    layoutStage(); await openWorkspace([twoPhotos()]); await compose();
    expect(overlay()).toHaveAttribute('data-overlay', 'thirds');
    expect(overlay().children).toHaveLength(4);
    for (const [name, count] of [['grid', 10], ['golden', 4], ['diagonals', 2], ['off', 0]] as const) {
      await key('o'); expect(overlay()).toHaveAttribute('data-overlay', name); expect(overlay().children).toHaveLength(count);
    }
    fireEvent.change(document.querySelector('#compose-overlay')!, { target: { value: 'golden' } });
    expect(window.localStorage.getItem('compose.overlay')).toBe('golden');
    await key('Enter'); await compose();
    expect(overlay()).toHaveAttribute('data-overlay', 'golden');
    await key('Enter');
    window.localStorage.setItem('compose.overlay', 'bogus');
    await compose();
    await waitFor(() => expect(overlay()).toHaveAttribute('data-overlay', 'thirds'));
  });
});
