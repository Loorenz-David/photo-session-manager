// The harness registers the service mocks; it must be evaluated before this file's own service imports.
import { composition, compose, delivered, layoutStage, openWorkspace, photo, SESSION, stage, tile, view } from './harness';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { commands } from '@/services/bindings';
import { setComposition } from '@/services/compositions';

const NOTE = "Camera preview — Capture One's version is on its way";
/** The row's rect (0.1, 0.1, ·, 0.6) in 4:5 on a 3:2 frame. */
const FOUR_FIVE = { x: 0.1, y: 0.1, w: 0.32, h: 0.6 };
const editorImage = () => stage()!.querySelector('.compose-image img');

describe('P58 compose on the camera preview', () => {
  it('C1(a) opens on the camera original turned by its orientation and shows the line', async () => {
    layoutStage();
    const p = photo(1, 'main', { position: 1, adjusted_preview_path: null, thumbnail_path: null, orientation: 6,
      composition: composition(1, { frame: { w: 4160, h: 6240 } }) });
    await openWorkspace([view([p])]);
    await compose();
    expect(editorImage()).toHaveAttribute('src', 'asset:///previews/1/original.jpg');
    expect(editorImage()?.getAttribute('style')).toContain('rotate(0deg) scaleX(1) rotate(90deg)');
    // The landscape JPEG is laid out with the portrait frame's sides exchanged before it is turned upright.
    expect(editorImage()).toHaveAttribute('data-swapped', 'true');
    expect(screen.getByText(NOTE)).toBeVisible();
  });

  it('C1(b) a camera original that does not match keeps the waiting reason', async () => {
    layoutStage();
    const p = photo(1, 'main', { position: 1, adjusted_preview_path: null, composition: composition(1, { frame: null, rect: null }) });
    await openWorkspace([view([p])]);
    fireEvent.keyDown(window, { key: 'r' });
    expect(stage()).toBeNull();
    expect(screen.getByRole('button', { name: 'Compose' })).toHaveAttribute('aria-disabled', 'true');
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Photo actions' }), { button: 0 });
    expect(await screen.findByText("Waiting for Capture One's full-frame preview")).toBeVisible();
  });

  it('C1(c) swaps to the adjusted picture in place, keeps the framing and drops the line', async () => {
    layoutStage();
    const camera = photo(1, 'main', { position: 1, adjusted_preview_path: null, thumbnail_path: null,
      composition: composition(1, { shape: '4:5', rect: FOUR_FIVE, frame: { w: 6240, h: 4160 }, origin: 'user', revision: 1 }) });
    const adjusted = { ...camera, adjusted_preview_path: '/previews/1/adjusted-1.jpg',
      composition: composition(1, { shape: '4:5', rect: FOUR_FIVE, frame: { w: 300, h: 200 }, origin: 'user', revision: 1 }) };
    await openWorkspace([view([camera]), view([adjusted])]);
    await compose();
    expect(screen.getByText(NOTE)).toBeVisible();
    await act(async () => delivered.workspace.forEach(send => send({ kind: 'photo_changed', photo_id: 1, session_id: SESSION })));
    await waitFor(() => expect(editorImage()).toHaveAttribute('src', 'asset:///previews/1/adjusted-1.jpg'));
    expect(stage()).not.toBeNull();
    expect(editorImage()).not.toHaveAttribute('data-swapped');
    expect(screen.queryByText(NOTE)).toBeNull();
    expect(vi.mocked(setComposition)).not.toHaveBeenCalled();
  });

  it('C1(d) stage and tile show the composed camera original before Capture One renders', async () => {
    layoutStage();
    const p = photo(1, 'main', { position: 1, adjusted_preview_path: null, thumbnail_path: null,
      composition: composition(1, { frame: { w: 6240, h: 4160 }, rect: FOUR_FIVE, shape: '4:5' }) });
    await openWorkspace([view([p])]);
    expect(document.querySelector('.stage-composed')).toHaveStyle({ aspectRatio: '1996.8 / 2496' });
    expect(document.querySelector('.stage-composed .stage-original')).toHaveAttribute('src', 'asset:///previews/1/original.jpg');
    expect(tile(1).querySelector('.tile-composed .thumb-image')).toHaveAttribute('src', 'asset:///previews/1/original.jpg');
  });

  it('C3(a) reports the photo on the stage as the one to render first', async () => {
    const focus = vi.spyOn(commands, 'photosPreviewFocus').mockResolvedValue({ status: 'ok', data: null });
    await openWorkspace([view([photo(1, 'main', { position: 1 })])]);
    await waitFor(() => expect(focus).toHaveBeenLastCalledWith(1));
  });
});
