// The harness registers the service mocks; it must be evaluated before this file's own service imports.
import { captureFrames, compose, composition, DEFAULT_RECT, image, layoutStage, openWorkspace, photo, pointer, savedValues, stage, tile, transformOf, view } from './harness';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { toast } from 'sonner';
import { setComposition, type Rect } from '@/services/compositions';
import { integrationKind, runExport } from '@/services/export';
import { publishConfirm, publishPreview } from '@/services/publish';
import { ServiceError } from '@/services/sessions';

vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => null), runExport: vi.fn(), openExportFolder: vi.fn(), onExportProgress: vi.fn(async () => () => undefined) }));
vi.mock('@/services/publish', () => ({ publishPreview: vi.fn(), publishConfirm: vi.fn(), publishRetry: vi.fn(), notificationsList: vi.fn(async () => []), onPublishProgress: vi.fn(async () => () => undefined) }));

const near = (actual: number, expected: number, tolerance = 1e-6) => expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance);
const rectNear = (actual: Rect, expected: Rect, tolerance = 1e-6) => {
  near(actual.x, expected.x, tolerance);
  near(actual.y, expected.y, tolerance);
  near(actual.w, expected.w, tolerance);
  near(actual.h, expected.h, tolerance);
};
const banner = () => document.querySelector('.compose-review');
const FRAME_TEXT = 'Capture One changed this photo’s frame — check the framing and press Return';
const GEOMETRY_TEXT = 'Changed in Capture One: straighten here instead';
/** The default square re-fitted to the turned (portrait) frame: the largest square at the same centre. */
const REFITTED: Rect = { x: 0, y: 0.16660156250000002, w: 1, h: 0.666796875 };
const REFUSED = () => new ServiceError({ code: 'needs_review', message: 'Re-frame first: Main 02', data: { photos: 'Main 02' } });

/** Photo 1 came back from Capture One turned: the observed frame is portrait, the stored fractions are the landscape square. */
const framed = () => view([
  photo(1, 'main', { position: 1, composition: composition(1, { state: 'needs_review', review_reason: 'frame', frame: { w: 1707, h: 2560 } }) }),
  photo(2, 'main', { position: 2 }),
]);
/** Photo 1 was straightened in Capture One: same frame size, geometry changed. */
const geometry = () => view([
  photo(1, 'main', { position: 1, composition: composition(1, { state: 'needs_review', review_reason: 'geometry' }) }),
  photo(2, 'main', { position: 2 }),
]);

async function editor(photos = framed()) {
  layoutStage();
  await openWorkspace([photos]);
  const frames = captureFrames();
  await compose();
  return frames;
}

describe('C7 the Re-frame confirmation', () => {
  it('C7(a) opening on a photo whose frame changed shows the banner and the stored rect re-fitted to the portrait frame, saving nothing', async () => {
    await editor();
    expect(banner()).toHaveTextContent(FRAME_TEXT);
    // The re-fitted square fills the 652 box: the portrait image is 652 × 977.81 centred at (500, 350).
    const placed = transformOf();
    near(placed.x, 174, 0.5);
    near(placed.y, 350 - 977.81 / 2, 0.5);
    near(parseFloat(image()!.style.width), 652, 0.5);
    near(parseFloat(image()!.style.height), 977.81, 0.5);
    expect(document.querySelector<HTMLElement>('.output-preview')!.style.aspectRatio).toBe('1707 / 1707');
    expect(setComposition).not.toHaveBeenCalled();
  });

  it('C7(b) Return confirms: one save with the re-fitted rect, and the ok answer removes the banner and the tile\'s Re-frame corner', async () => {
    await editor();
    expect(tile(1, document.querySelector('.compose-browser')!).querySelector('.tile-review')).toHaveTextContent('Re-frame');
    await act(async () => {
      fireEvent.keyDown(window, { key: 'Enter' });
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(vi.mocked(setComposition).mock.calls[0][0]).toBe(1);
    rectNear(savedValues(0).rect, REFITTED, 1e-9);
    expect(savedValues(0)).toMatchObject({ shape: '1:1', quarter_turns: 0, flip: 'none', straighten: 0 });
    await waitFor(() => expect(stage()).toBeNull());
    expect(banner()).toBeNull();
    expect(tile(1, document.querySelector('.photo-panel')!).querySelector('.tile-review')).toBeNull();
  });

  it('C7(c) a geometry change shows the straighten notice over the unchanged rect; Return still sends exactly one save that clears it', async () => {
    const frames = await editor(geometry());
    expect(banner()).toHaveTextContent(GEOMETRY_TEXT);
    frames.flush();
    const placed = transformOf();
    near(placed.x, 11.09, 0.5);
    near(placed.y, 24, 0.5);
    expect(setComposition).not.toHaveBeenCalled();
    await act(async () => {
      fireEvent.keyDown(window, { key: 'Enter' });
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(savedValues(0).rect).toEqual(DEFAULT_RECT);
    expect(savedValues(0)).toMatchObject({ shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0 });
    await waitFor(() => expect(stage()).toBeNull());
    expect(tile(1, document.querySelector('.photo-panel')!).querySelector('.tile-review')).toBeNull();
  });

  it('C7(d) any save confirms: a drag after (a) saves the re-fitted rect moved by the drag and the ok answer removes the banner', async () => {
    const frames = await editor();
    pointer('pointerdown', 500, 350);
    pointer('pointermove', 500, 300);
    frames.flush();
    await act(async () => {
      pointer('pointerup', 500, 300);
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    // The photo followed the pointer up by 50 px, so the frame sits 50 / 977.81 lower on it; w = 1 leaves no horizontal room.
    near(savedValues(0).rect.y, REFITTED.y + 50 / 977.81, 1e-6);
    expect(savedValues(0).rect.x).toBe(0);
    near(savedValues(0).rect.w, 1, 1e-9);
    near(savedValues(0).rect.h, REFITTED.h, 1e-9);
    expect(banner()).toBeNull();
    expect(stage()).not.toBeNull();
    await act(async () => {
      fireEvent.keyDown(window, { key: 'Enter' });
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
  });
});

describe('C8 export and publish refusals', () => {
  it('C8(a) a refused export surfaces the backend\'s naming of the photos as a toast', async () => {
    vi.mocked(integrationKind).mockResolvedValue(null);
    vi.mocked(runExport).mockRejectedValue(REFUSED());
    await openWorkspace([geometry()]);
    fireEvent.click(await screen.findByRole('button', { name: 'Export' }));
    await waitFor(() => expect(runExport).toHaveBeenCalledWith(5));
    await waitFor(() => expect(toast.error).toHaveBeenCalledWith(expect.stringContaining('Re-frame first: Main 02'), expect.anything()));
  });

  it('C8(b) a refused publish confirmation shows the same toast and leaves the popover open', async () => {
    vi.mocked(integrationKind).mockResolvedValue('shopify');
    vi.mocked(publishPreview).mockResolvedValue({ summary: 'Publish session 5: two selected images.', items: [{ session_id: 5, main: [1, 2], condition: [], main_thumbnails: ['/1.jpg', '/2.jpg'], condition_thumbnails: [], details_filled: 0, details_total: 0, unreviewed: 0, announced_removals: [], create_item: false }] });
    vi.mocked(publishConfirm).mockRejectedValue(REFUSED());
    await openWorkspace([geometry()]);
    fireEvent.click(await screen.findByRole('button', { name: 'Publish' }));
    const popover = await screen.findByRole('dialog', { name: 'Review publish' });
    fireEvent.click(await screen.findByRole('button', { name: 'Publish 2 images' }));
    await waitFor(() => expect(publishConfirm).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(toast.error).toHaveBeenCalledWith(expect.stringContaining('Re-frame first: Main 02')));
    expect(popover).toBeInTheDocument();
    expect(screen.getByRole('dialog', { name: 'Review publish' })).toBe(popover);
  });
});
