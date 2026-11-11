import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { EditMenuEvent, HistoryChanged } from '@/services/history';
import type { Form } from '@/services/attributes';

const delivered = vi.hoisted(() => ({
  history: [] as ((event: HistoryChanged) => void)[],
  menu: [] as ((event: EditMenuEvent) => void)[],
}));
vi.mock('@/services/history', () => ({
  historyGet: vi.fn(), historyUndo: vi.fn(), historyRedo: vi.fn(), editMenuSet: vi.fn(async () => undefined),
  onHistoryChanged: vi.fn(async (handler: (event: HistoryChanged) => void) => { delivered.history.push(handler); return () => undefined; }),
  onEditMenu: vi.fn(async (handler: (event: EditMenuEvent) => void) => { delivered.menu.push(handler); return () => undefined; }),
}));
vi.mock('@/services/attributes', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/attributes')>()),
  getForm: vi.fn(),
}));

import { compose, openWorkspace, photo, SESSION, tile, view } from '@/features/compose/__tests__/harness';
import { applyComposition, setComposition } from '@/services/compositions';
import { historyGet, historyRedo, historyUndo, editMenuSet } from '@/services/history';
import { ServiceError } from '@/services/sessions';
import { getForm } from '@/services/attributes';

const p = () => view([photo(1, 'main', { position: 1 })]);
const undone = { label: 'Approve 03', direction: 'undone' as const, sessions: [], active_photo_id: null };
const redone = { ...undone, direction: 'redone' as const };
const press = (key: string, options: KeyboardEventInit = {}) => fireEvent.keyDown(window, { key, ...options });
const change = (session_id: number, undo: string | null, redo: string | null) => act(() => delivered.history.forEach(send => send({ session_id, undo, redo })));
const button = (name: 'Undo' | 'Redo' | 'Notifications') => screen.getByRole('button', { name });
const panelTile = (id: number) => tile(id, document.querySelector('.photo-panel')!);

beforeEach(() => {
  delivered.history.length = 0;
  delivered.menu.length = 0;
  vi.mocked(historyGet).mockResolvedValue({ undo: 'Approve 03', redo: null });
  vi.mocked(historyUndo).mockResolvedValue(undone);
  vi.mocked(historyRedo).mockResolvedValue(redone);
  vi.mocked(applyComposition).mockResolvedValue({ applied: [], refused: [] });
  const title: Form = { item_id: 50, item_type_from_integration: false, groups: [
    { group: 'details', fields: [], filled: 0, total: 0 },
    { group: 'condition', fields: [], filled: 0, total: 0 },
    { group: 'product', fields: [{ definition: { integration: 'shopify', key: 'product.title', label: 'Title', core_type: 'text', choices: [], validations: '{}', group: 'product', description: null, writable: true }, value: null, is_default: false, is_severity: false, selected: null, default: null, saved: [] }], filled: 0, total: 1 },
  ] };
  vi.mocked(getForm).mockResolvedValue(title);
});

describe('C1 history keys', () => {
  it('C1(a) workspace ⌘Z and ⌘⇧Z call the open session history', async () => {
    await openWorkspace([p()]);
    press('z', { metaKey: true });
    press('z', { metaKey: true, shiftKey: true });
    await waitFor(() => expect(historyRedo).toHaveBeenCalledOnce());
    expect(historyUndo).toHaveBeenCalledExactlyOnceWith(SESSION);
    expect(historyRedo).toHaveBeenCalledExactlyOnceWith(SESSION);
    const actualPhotos = await vi.importActual<typeof import('@/services/photos')>('@/services/photos');
    expect('undoDelete' in actualPhotos).toBe(false);
  });
  it('C1(b) ⌘Z and ⌘⇧Z in the details Title field belong to the field', async () => {
    await openWorkspace([p()]);
    press('d');
    const product = await screen.findByRole('tab', { name: /Product/ });
    fireEvent.click(product);
    const input = await screen.findByLabelText('Title');
    input.focus();
    fireEvent.keyDown(input, { key: 'z', metaKey: true });
    fireEvent.keyDown(input, { key: 'z', metaKey: true, shiftKey: true });
    expect(historyUndo).not.toHaveBeenCalled();
    expect(historyRedo).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });
  it('C1(c) ⌘Z works while composing and plain Z stays with the loupe', async () => {
    await openWorkspace([p()]);
    await compose();
    press('z', { metaKey: true });
    press('z');
    await waitFor(() => expect(historyUndo).toHaveBeenCalledExactlyOnceWith(SESSION));
  });
});

describe('C2 notices', () => {
  it('C2(a) names the undone and redone step exactly', async () => {
    await openWorkspace([p()]);
    press('z', { metaKey: true });
    expect(await screen.findByText('Undone: Approve 03 · Redo ⌘⇧Z')).toBeInTheDocument();
    press('z', { metaKey: true, shiftKey: true });
    expect(await screen.findByText('Redone: Approve 03 · Undo ⌘Z')).toBeInTheDocument();
    expect(screen.queryByText('Undone: Approve 03 · Redo ⌘⇧Z')).toBeNull();
  });
  it('C2(b) shows the backend refusal and leaves the active photo', async () => {
    vi.mocked(historyUndo).mockRejectedValue(new ServiceError({ code: 'step_cannot_apply', message: "Can't undo Move to Lamp 2: that session is published", data: { label: 'Move to Lamp 2', reason: 'that session is published' } }));
    await openWorkspace([p()]);
    const before = document.querySelector('.photo-panel [aria-current="true"]')?.getAttribute('data-photo-id');
    press('z', { metaKey: true });
    expect(await screen.findByText("Can't undo Move to Lamp 2: that session is published")).toBeInTheDocument();
    expect(document.querySelector('.photo-panel [aria-current="true"]')?.getAttribute('data-photo-id')).toBe(before);
  });
  it('C2(c) gives fixed notices for empty undo and redo', async () => {
    vi.mocked(historyUndo).mockRejectedValue(new ServiceError({ code: 'nothing_to_undo', message: 'no', data: {} }));
    vi.mocked(historyRedo).mockRejectedValue(new ServiceError({ code: 'nothing_to_redo', message: 'no', data: {} }));
    await openWorkspace([p()]);
    press('z', { metaKey: true });
    expect(await screen.findByText('Nothing to undo.')).toBeInTheDocument();
    press('z', { metaKey: true, shiftKey: true });
    expect(await screen.findByText('Nothing to redo.')).toBeInTheDocument();
  });
});

describe('C3 header', () => {
  it('C3(a) renders ordered icon buttons and their hover cards only in a session', async () => {
    await openWorkspace([p()]);
    await waitFor(() => expect(button('Redo')).toHaveAttribute('aria-disabled', 'true'));
    const undoButton = button('Undo');
    const redoButton = button('Redo');
    expect(undoButton.querySelector('svg')).not.toBeNull();
    expect(redoButton.querySelector('svg')).not.toBeNull();
    expect(undoButton.textContent).toBe('');
    expect(redoButton.textContent).toBe('');
    expect(undoButton.compareDocumentPosition(redoButton) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(redoButton.compareDocumentPosition(button('Notifications')) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(undoButton).not.toHaveAttribute('aria-disabled');
    undoButton.focus();
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Undo · ⌘Z — Approve 03');
    redoButton.focus();
    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent('Redo · ⌘⇧Z — Nothing to redo'));
    fireEvent.click(screen.getByRole('button', { name: 'Back to Sessions' }));
    await waitFor(() => { expect(screen.queryByRole('button', { name: 'Undo' })).toBeNull(); expect(screen.queryByRole('button', { name: 'Redo' })).toBeNull(); });
  });
  it('C3(b) same-session event changes both labels; another session does not', async () => {
    await openWorkspace([p()]);
    await change(SESSION, null, 'Approve 03');
    expect(button('Undo')).toHaveAttribute('aria-disabled', 'true');
    expect(button('Redo')).not.toHaveAttribute('aria-disabled');
    button('Undo').focus();
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Undo · ⌘Z — Nothing to undo');
    button('Redo').focus();
    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent('Redo · ⌘⇧Z — Approve 03'));
    await change(99, 'Other', null);
    expect(button('Undo')).toHaveAttribute('aria-disabled', 'true');
    expect(button('Redo')).not.toHaveAttribute('aria-disabled');
    expect(screen.getByRole('tooltip')).toHaveTextContent('Redo · ⌘⇧Z — Approve 03');
  });
  it('C3(c) disabled icon never acts', async () => {
    await openWorkspace([p()]);
    await waitFor(() => expect(button('Undo')).not.toHaveAttribute('aria-disabled'));
    fireEvent.click(button('Undo'));
    fireEvent.click(button('Redo'));
    await waitFor(() => expect(historyUndo).toHaveBeenCalledExactlyOnceWith(SESSION));
    expect(historyRedo).not.toHaveBeenCalled();
  });
});

describe('C6 grid Split', () => {
  const three = () => view([photo(1, 'main', { position: 1 }), photo(2, 'main', { position: 2 }), photo(3, 'main', { position: 3 })]);
  it('C6(a) two selected photos go through one Apply and both tiles split', async () => {
    vi.mocked(applyComposition).mockResolvedValue({ applied: [2, 3], refused: [] });
    await openWorkspace([three()]);
    fireEvent.click(panelTile(2), { metaKey: true });
    fireEvent.click(panelTile(3), { metaKey: true });
    fireEvent.contextMenu(panelTile(2));
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Split S' }));
    await waitFor(() => expect(applyComposition).toHaveBeenCalledOnce());
    const items = vi.mocked(applyComposition).mock.calls[0][0];
    expect(items.map(item => item.photo_id)).toEqual([2, 3]);
    const width = Math.sqrt(1707 * 1707 * 2);
    const expected = { x: 0.5 - width / 2560 / 2, y: 0.5 - width / 2 / 1707 / 2, w: width / 2560, h: width / 2 / 1707 };
    for (const item of items) {
      expect(item.values.kind).toBe('split');
      expect(item.values.rect.x).toBeCloseTo(expected.x, 6);
      expect(item.values.rect.y).toBeCloseTo(expected.y, 6);
      expect(item.values.rect.w).toBeCloseTo(expected.w, 6);
      expect(item.values.rect.h).toBeCloseTo(expected.h, 6);
    }
    expect(setComposition).not.toHaveBeenCalled();
    await waitFor(() => { expect(panelTile(2).querySelector('.composed-split')).not.toBeNull(); expect(panelTile(3).querySelector('.composed-split')).not.toBeNull(); });
  });
  it('C6(b) one photo keeps the single set path', async () => {
    await openWorkspace([three()]);
    fireEvent.contextMenu(panelTile(2));
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Split S' }));
    await waitFor(() => expect(setComposition).toHaveBeenCalledOnce());
    expect(vi.mocked(setComposition).mock.calls[0][0]).toBe(2);
    expect(applyComposition).not.toHaveBeenCalled();
  });
});

describe('C7 Edit menu bridge', () => {
  it('C7(a) follows loaded labels and this session’s events only', async () => {
    await openWorkspace([p()]);
    await waitFor(() => expect(editMenuSet).toHaveBeenCalledWith('Approve 03', null));
    await change(SESSION, null, 'Approve 03');
    await waitFor(() => expect(editMenuSet).toHaveBeenLastCalledWith(null, 'Approve 03'));
    const calls = vi.mocked(editMenuSet).mock.calls.length;
    await change(99, 'Other', null);
    expect(editMenuSet).toHaveBeenCalledTimes(calls);
  });
  it('C7(b) menu Undo then Redo share the command path', async () => {
    await openWorkspace([p()]);
    await act(() => delivered.menu.forEach(send => send({ action: 'undo' })));
    await act(() => delivered.menu.forEach(send => send({ action: 'redo' })));
    await waitFor(() => expect(historyRedo).toHaveBeenCalledOnce());
    expect(historyUndo).toHaveBeenCalledExactlyOnceWith(SESSION);
    expect(historyRedo).toHaveBeenCalledExactlyOnceWith(SESSION);
    expect(vi.mocked(historyUndo).mock.invocationCallOrder[0]).toBeLessThan(vi.mocked(historyRedo).mock.invocationCallOrder[0]);
  });
});
