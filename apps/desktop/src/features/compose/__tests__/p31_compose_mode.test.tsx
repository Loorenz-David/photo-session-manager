// The harness registers the service mocks; it must be evaluated before this file's own service imports.
import { compose, image, layoutStage, openWorkspace, photo, stage, tile, transformOf, twoPhotos, view } from './harness';
import { cleanup, fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { setComposition } from '@/services/compositions';
import { classifyPhoto, deletePhoto } from '@/services/photos';

/** The editor is open on the stage: the frame, the image placed per the view maths, Done, and nothing of the review stage. */
function expectEditorOpen() {
  expect(stage()).not.toBeNull();
  const placed = transformOf();
  expect(Math.abs(placed.x - 11.09)).toBeLessThanOrEqual(0.5);
  expect(Math.abs(placed.y - 24)).toBeLessThanOrEqual(0.5);
  expect(placed.deg).toBe(0);
  expect(document.querySelector('.compose-overlay')).not.toBeNull();
  expect(document.querySelector('.compose-done')).not.toBeNull();
  expect(document.querySelector('.swipe-layer')).toBeNull();
  expect(document.querySelector('.toggle-pill')).toBeNull();
  expect(document.querySelector('.review-buttons')).toBeNull();
  expect(document.querySelector('main.workspace > .folded-strip')).not.toBeNull();
  expect(document.querySelector('main.workspace aside.photo-panel')).toBeNull();
}

describe('C2 entering and leaving Compose', () => {
  it('C2(a) R, a double-click on the stage photo, Compose… in the ⋯ menu and in the right-click menu each open the editor', async () => {
    layoutStage();
    await openWorkspace([twoPhotos()]);
    await compose();
    expectEditorOpen();
    cleanup();

    await openWorkspace([twoPhotos()]);
    fireEvent.doubleClick(document.querySelector('.stage-frame')!);
    await waitFor(() => expect(stage()).not.toBeNull());
    expectEditorOpen();
    cleanup();

    await openWorkspace([twoPhotos()]);
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Photo actions' }), { button: 0, ctrlKey: false });
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Compose… R' }));
    await waitFor(() => expect(stage()).not.toBeNull());
    expectEditorOpen();
    cleanup();

    await openWorkspace([twoPhotos()]);
    fireEvent.contextMenu(tile(1));
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Compose… R' }));
    await waitFor(() => expect(stage()).not.toBeNull());
    expectEditorOpen();
    expect(image()?.querySelector('img')?.getAttribute('src')).toBe('asset:///previews/1/adjusted-1.jpg');
  });

  it('C2(b) Return and Done leave the editor with the same photo active and nothing saved', async () => {
    layoutStage();
    await openWorkspace([twoPhotos()]);
    await compose();
    fireEvent.keyDown(window, { key: 'Enter' });
    await waitFor(() => expect(stage()).toBeNull());
    expect(document.querySelector('.swipe-layer')).not.toBeNull();
    expect(tile(1, document.querySelector('.photo-panel')!).getAttribute('data-active')).toBe('true');
    expect(document.querySelector('.stage-place')).toHaveTextContent('Main 1 of 2');
    expect(setComposition).not.toHaveBeenCalled();

    await compose();
    fireEvent.click(document.querySelector('.compose-done')!);
    await waitFor(() => expect(stage()).toBeNull());
    expect(document.querySelector('.swipe-layer')).not.toBeNull();
    expect(tile(1, document.querySelector('.photo-panel')!).getAttribute('data-active')).toBe('true');
    expect(setComposition).not.toHaveBeenCalled();
  });

  it('C2(c) the workspace keys A, C, ⌫, Space, F, I, D do nothing while composing', async () => {
    layoutStage();
    await openWorkspace([twoPhotos()]);
    await compose();
    for (const key of ['a', 'c', 'Backspace', ' ', 'f', 'i', 'd']) fireEvent.keyDown(window, { key });
    expect(classifyPhoto).not.toHaveBeenCalled();
    expect(deletePhoto).not.toHaveBeenCalled();
    expect(document.querySelector('.toggle-pill')).toBeNull();
    expect(document.querySelector('.folded-strip')).toBeNull();
    expect(document.querySelector('.item-card')).toBeNull();
    expect(document.querySelector('[role="dialog"]')).toBeNull();
    expect(stage()).not.toBeNull();
    fireEvent.keyDown(window, { key: 'Enter' });
    await waitFor(() => expect(stage()).toBeNull());
    expect(document.querySelector('.toggle-pill')).toHaveTextContent('Adjusted');
    expect(document.querySelector('.folded-strip')).toBeNull();
    expect(document.querySelector('.item-card')).toBeNull();
  });

  it('C2(d) a photo without an adjusted preview or without a rect cannot be composed: R does nothing, Compose… is disabled', async () => {
    layoutStage();
    await openWorkspace([view([photo(1, 'main', { position: 1, adjusted_preview_path: null })])]);
    fireEvent.keyDown(window, { key: 'r' });
    expect(stage()).toBeNull();
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Photo actions' }), { button: 0, ctrlKey: false });
    expect((await screen.findByRole('menuitem', { name: /Compose… Waiting for Capture One/ })).getAttribute('aria-disabled')).toBe('true');
    cleanup();

    const unframed = photo(1, 'main', { position: 1 });
    unframed.composition = { ...unframed.composition, rect: null, frame: null };
    await openWorkspace([view([unframed])]);
    fireEvent.keyDown(window, { key: 'r' });
    expect(stage()).toBeNull();
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Photo actions' }), { button: 0, ctrlKey: false });
    expect((await screen.findByRole('menuitem', { name: /Compose… Waiting for Capture One/ })).getAttribute('aria-disabled')).toBe('true');
  });

  it('C2(e) the first-time line shows once and the key is set', async () => {
    window.localStorage.clear();
    layoutStage();
    await openWorkspace([twoPhotos()]);
    await compose();
    expect(screen.getByText('Crops made in Capture One are not used.')).toBeInTheDocument();
    expect(window.localStorage.getItem('compose.hintShown')).toBe('1');
    fireEvent.keyDown(window, { key: 'Enter' });
    await waitFor(() => expect(stage()).toBeNull());
    await compose();
    expect(screen.queryByText('Crops made in Capture One are not used.')).toBeNull();
    expect(window.localStorage.getItem('compose.hintShown')).toBe('1');
    expect(vi.mocked(setComposition)).not.toHaveBeenCalled();
  });
});
