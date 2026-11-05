import { composition, openWorkspace, photo, tile, twoPhotos, view } from '@/features/compose/__tests__/harness';
import { act, cleanup, fireEvent, screen, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { applyComposition, setComposition } from '@/services/compositions';
import { getSessionView } from '@/services/photos';

vi.mock('@/services/attributes', async original => ({
  ...(await original<typeof import('@/services/attributes')>()),
  getForm: vi.fn(async () => ({ groups: [], fields: [] })),
}));

const waiting = "Waiting for Capture One's full-frame preview";
const menu = () => fireEvent.pointerDown(screen.getByRole('button', { name: 'Photo actions' }), { button: 0 });

it('C1(a) Compose button hints its key and opens the editor', async () => {
  await openWorkspace([twoPhotos()]);
  const button = screen.getByRole('button', { name: 'Compose' });
  expect(button.closest('.stage-bar .stage-tools')).not.toBeNull();
  expect(button).not.toHaveAttribute('aria-disabled');
  await act(async () => { button.focus(); });
  expect(await screen.findByRole('tooltip')).toHaveTextContent('Compose (R)');
  fireEvent.click(button);
  expect(document.querySelector('.compose-stage')).not.toBeNull();
});

it('C1(b) waiting Compose explains why for button, double-click, R and menu', async () => {
  const unframed = photo(1, 'main', { position: 1, composition: composition(1, { rect: null, frame: null }) });
  await openWorkspace([view([unframed])]);
  const button = screen.getByRole('button', { name: 'Compose' });
  expect(button).toHaveAttribute('aria-disabled', 'true');
  await act(async () => { button.focus(); });
  expect(await screen.findByRole('tooltip')).toHaveTextContent(waiting);
  fireEvent.click(button);
  await waitFor(() => expect(screen.getAllByText(waiting).length).toBeGreaterThan(0));
  fireEvent.doubleClick(document.querySelector('.stage-frame')!);
  fireEvent.keyDown(window, { key: 'r' });
  expect(document.querySelector('.compose-stage')).toBeNull();
  menu();
  const entry = await screen.findByRole('menuitem', { name: `Compose… ${waiting}` });
  expect(entry).toHaveAttribute('data-disabled');
  expect(entry.querySelector('kbd')).toBeNull();
  expect(document.querySelectorAll('[data-sonner-toast]').length).toBe(1);
});

it('C1(c) a photo needing review can still open Compose', async () => {
  const needsReview = photo(1, 'main', { position: 1, composition: composition(1, { state: 'needs_review', review_reason: 'geometry' }) });
  await openWorkspace([view([needsReview])]);
  const button = screen.getByRole('button', { name: 'Compose' });
  expect(button).not.toHaveAttribute('aria-disabled');
  fireEvent.click(button);
  expect(document.querySelector('.compose-stage')).not.toBeNull();
});

it('C4(a) an unreviewed grid photo splits and still fills one Main place', async () => {
  const splitRect = { x: 0.028503408013501264, y: 0.1464466094067262, w: 0.9429931839729975, h: 0.7071067811865476 };
  const photos = [
    photo(1, 'main', { position: 1, composition: composition(1, { kind: 'split', rect: splitRect }) }),
    photo(2, 'main', { position: 2 }),
    photo(3, 'main', { position: 3, composition: composition(3, { kind: 'split', rect: splitRect }) }),
    photo(4, 'unreviewed'),
    photo(5, 'condition', { position: 1, composition: composition(5, { kind: 'split', rect: splitRect }) }),
    photo(6, 'condition', { position: 2 }),
  ];
  await openWorkspace([view(photos)]);
  fireEvent.contextMenu(tile(4, document.querySelector('.photo-panel')!));
  const entry = await screen.findByRole('menuitem', { name: 'Split S' });
  expect(entry).not.toHaveAttribute('aria-disabled');
  expect(entry).not.toHaveAttribute('data-disabled');
  fireEvent.click(entry);
  await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
  const [id, values] = vi.mocked(setComposition).mock.calls[0];
  expect(id).toBe(4);
  expect(values.kind).toBe('split');
  expect(values.shape).toBe('1:1');
  expect(values.rect.x).toBeCloseTo(splitRect.x, 3);
  expect(values.rect.y).toBeCloseTo(splitRect.y, 3);
  expect(values.rect.w).toBeCloseTo(splitRect.w, 3);
  expect(values.rect.h).toBeCloseTo(splitRect.h, 3);
  await waitFor(() => expect(tile(4).querySelector('.composed-split')).not.toBeNull());
  expect([...document.querySelectorAll('.photo-panel [aria-label="Main images"] .tile-number')].map(n => n.textContent)).toEqual(['01–02', '03', '04–05', '06']);
});

it('C4(b) unreviewed photos split by S and the editor segment', async () => {
  await openWorkspace([view([photo(1, 'unreviewed')])]);
  fireEvent.keyDown(window, { key: 'r' });
  const segment = screen.getByRole('button', { name: 'Split' });
  expect(segment).not.toHaveAttribute('aria-disabled');
  fireEvent.keyDown(window, { key: 's' });
  await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
  const values = vi.mocked(setComposition).mock.calls[0][1];
  expect(values.kind).toBe('split');
  expect(values.shape).toBe('1:1');
  expect(values.rect.x).toBeCloseTo(0.0285034, 3);
  expect(values.rect.h).toBeCloseTo(0.7071068, 3);
});

it('C4(b) the Split segment also saves an unreviewed photo', async () => {
  await openWorkspace([view([photo(1, 'unreviewed')])]);
  fireEvent.keyDown(window, { key: 'r' });
  fireEvent.click(screen.getByRole('button', { name: 'Split' }));
  await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
  expect(vi.mocked(setComposition).mock.calls[0][1].kind).toBe('split');
  expect(vi.mocked(setComposition).mock.calls[0][1].rect.x).toBeCloseTo(0.0285034, 3);
});

it('C5(a) header and D open the product details form', async () => {
  await openWorkspace([twoPhotos()]);
  const details = screen.getByRole('button', { name: 'Details' });
  expect(details).not.toBeDisabled();
  await act(async () => { details.focus(); });
  expect(await screen.findByRole('tooltip')).toHaveTextContent('Details (D)');
  fireEvent.click(details);
  expect(await screen.findByRole('dialog', { name: 'Chair' })).toBeTruthy();
  fireEvent.click(screen.getByRole('button', { name: 'Close' }));
  await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Chair' })).toBeNull());
  fireEvent.keyDown(window, { key: 'd' });
  expect(await screen.findByRole('dialog', { name: 'Chair' })).toBeTruthy();
  fireEvent.click(screen.getByRole('button', { name: 'Close' }));
  await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Chair' })).toBeNull());
  expect(document.querySelector('.stage-column button[aria-label="Product details"]')).toBeNull();
});

it('C5(b) item card opens the same details form', async () => {
  await openWorkspace([twoPhotos()]);
  fireEvent.keyDown(window, { key: 'i' });
  fireEvent.click(screen.getByRole('button', { name: 'Edit details →' }));
  expect(await screen.findByRole('dialog', { name: 'Chair' })).toBeTruthy();
});

it('C6(a) Sessions toggles from the photo panel header', async () => {
  await openWorkspace([twoPhotos()]);
  expect(screen.queryByRole('button', { name: 'Show sessions panel' })).toBeNull();
  const sessions = screen.getByRole('button', { name: 'Sessions' });
  expect(sessions.closest('.panel-head .panel-tools')).not.toBeNull();
  expect(sessions.textContent).toBe('');
  expect(sessions).toHaveAttribute('aria-pressed', 'false');
  await act(async () => { sessions.focus(); });
  expect(await screen.findByRole('tooltip')).toHaveTextContent('Sessions');
  fireEvent.click(sessions);
  expect(screen.getByRole('complementary', { name: 'Sessions' })).toBeTruthy();
  expect(sessions).toHaveAttribute('aria-pressed', 'true');
  fireEvent.click(sessions);
  expect(screen.queryByRole('complementary', { name: 'Sessions' })).toBeNull();
});

it('C6(b) folded strip toggles Sessions and Unassigned opens them on entry', async () => {
  await openWorkspace([twoPhotos()]);
  fireEvent.keyDown(window, { key: 'f' });
  const strip = document.querySelector('.folded-strip')!;
  const stripButton = screen.getByRole('button', { name: 'Sessions' });
  expect(strip.contains(stripButton)).toBe(true);
  fireEvent.click(stripButton);
  expect(screen.getByRole('complementary', { name: 'Sessions' })).toBeTruthy();
  cleanup();
  await openWorkspace([view([photo(1, 'unreviewed')], 1)], 1);
  expect(screen.getByRole('button', { name: 'Sessions' })).toHaveAttribute('aria-pressed', 'true');
  expect(screen.getByRole('complementary', { name: 'Sessions' })).toBeTruthy();
});

it('C7(a) Apply clears Re-frame immediately without reloading', async () => {
  const first = photo(1, 'main', { position: 1 });
  const second = photo(2, 'main', { position: 2, composition: composition(2, { state: 'needs_review', review_reason: 'geometry' }) });
  await openWorkspace([view([first, second])]);
  fireEvent.keyDown(window, { key: 'r' });
  fireEvent.keyDown(window, { key: 'c', metaKey: true, altKey: true });
  fireEvent.keyDown(window, { key: 'Enter' });
  const target = tile(2, document.querySelector('.photo-panel')!);
  expect(target.querySelector('.tile-review')).not.toBeNull();
  fireEvent.click(target, { metaKey: true });
  vi.mocked(applyComposition).mockResolvedValue({ applied: [2], refused: [] });
  const loads = vi.mocked(getSessionView).mock.calls.length;
  fireEvent.contextMenu(target);
  fireEvent.click(await screen.findByRole('menuitem', { name: 'Apply composition to selected' }));
  await waitFor(() => expect(tile(2).querySelector('.tile-review')).toBeNull());
  expect(vi.mocked(getSessionView).mock.calls.length).toBe(loads);
});
