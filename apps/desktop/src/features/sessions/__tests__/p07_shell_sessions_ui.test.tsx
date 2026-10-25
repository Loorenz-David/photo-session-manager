import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ItemDto_Serialize, RowLabelResponse, SessionRowResponse_Serialize } from '@/services/bindings';
import type { SessionList } from '@/services/sessions';

vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  createSession: vi.fn(),
  activateSession: vi.fn(),
  deactivateSession: vi.fn(),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/items', () => ({ searchItems: vi.fn(), nextSku: vi.fn(), createItem: vi.fn() }));
vi.mock('@/services/camera', () => ({ cameraStatus: vi.fn(), cameraRetry: vi.fn() }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onCameraStatus: vi.fn(async () => () => undefined),
}));

import App from '@/App';
import { CameraDot } from '@/features/shell/CameraDot';
import { SessionsPage } from '@/features/sessions/SessionsPage';
import { StatusPill } from '@/features/sessions/StatusPill';
import { UnassignedCard } from '@/features/sessions/UnassignedCard';
import { cameraRetry, cameraStatus } from '@/services/camera';
import { createItem, nextSku, searchItems } from '@/services/items';
import { activateSession, createSession, deactivateSession, listSessions } from '@/services/sessions';

// cmdk and Radix need these browser APIs, which jsdom lacks.
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};
Element.prototype.scrollIntoView ??= function scrollIntoView() {};

function row(id: number, title: string, label: RowLabelResponse = { kind: 'in_progress', value: 0 }): SessionRowResponse_Serialize {
  return {
    id,
    sku: `SKU-${id}`,
    item_type: '',
    title,
    main_count: 2,
    condition_count: 1,
    unreviewed_count: 0,
    status: 'in_progress',
    label,
    progress: null,
    last_activity_at: Date.now() - 2 * 60 * 60 * 1000,
    cover_preview_path: null,
  };
}

function list(groups: SessionList['groups'], counts = { open: 3, published: 0, all: 3 }, unassigned = 0): SessionList {
  return { groups, counts, unassigned_count: unassigned };
}

function item(id: number, sku: string, title: string, origin = 'local'): ItemDto_Serialize {
  return { id, sku, folder_key: sku, barcode: null, item_type: '', title, origin, created_at: 0, updated_at: 0 };
}

beforeEach(() => {
  vi.mocked(listSessions).mockResolvedValue(list([]));
  vi.mocked(activateSession).mockResolvedValue();
  vi.mocked(deactivateSession).mockResolvedValue();
  vi.mocked(cameraStatus).mockResolvedValue({ kind: 'connected', name: 'Sony ZV-E10' });
  vi.mocked(nextSku).mockResolvedValue(null);
  vi.mocked(searchItems).mockResolvedValue([]);
  window.location.hash = '#/';
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

/** Opens the New session dialog from the Sessions page with the `N` key and types a title. */
async function startNewSession(title: string) {
  render(<App />);
  await act(async () => undefined);
  fireEvent.keyDown(window, { key: 'n' });
  const input = await screen.findByLabelText('Search items by title or SKU');
  fireEvent.change(input, { target: { value: title } });
}

describe('C1 sessions list grouping', () => {
  it('C1(a) renders the delivered groups in order with their rows', async () => {
    vi.mocked(listSessions).mockResolvedValue(
      list([
        { label: 'Today', rows: [row(1, 'Alpha chair'), row(2, 'Beta desk')] },
        { label: 'Yesterday', rows: [row(3, 'Gamma lamp')] },
      ]),
    );
    render(<SessionsPage navigate={() => undefined} />);

    const today = await screen.findByRole('region', { name: 'Today' });
    const yesterday = screen.getByRole('region', { name: 'Yesterday' });
    const headers = screen.getAllByRole('heading', { level: 2 }).map(heading => heading.textContent);
    expect(headers).toEqual(['Today 2', 'Yesterday 1']);
    expect(within(today).getByText('Alpha chair')).toBeInTheDocument();
    expect(within(today).getByText('Beta desk')).toBeInTheDocument();
    expect(within(today).queryByText('Gamma lamp')).toBeNull();
    expect(within(yesterday).getByText('Gamma lamp')).toBeInTheDocument();
    expect(within(yesterday).queryByText('Alpha chair')).toBeNull();
  });
});

describe('C2 status pill', () => {
  it('C2(a) Publishing(42) reads "Publishing 42%" with a progress bar at 42', () => {
    render(<StatusPill label={{ kind: 'publishing', value: 42 }} />);
    expect(screen.getByText('Publishing 42%')).toBeInTheDocument();
    expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '42');
  });

  it('C2(b) PublishFailed reads "Publish failed" with a Retry button', () => {
    render(<StatusPill label={{ kind: 'publish_failed' }} />);
    expect(screen.getByText('Publish failed')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Retry' })).toBeInTheDocument();
  });

  it('C2(c) DescriptionFailed reads "Description failed"', () => {
    const { container } = render(<StatusPill label={{ kind: 'description_failed' }} />);
    expect(container).toHaveTextContent(/^Description failed$/);
  });

  it('C2(d) Published reads "Published"', () => {
    const { container } = render(<StatusPill label={{ kind: 'published' }} />);
    expect(container).toHaveTextContent(/^Published$/);
  });

  it('C2(e) Ready reads "Ready · Publish" as a button', () => {
    render(<StatusPill label={{ kind: 'ready' }} />);
    expect(screen.getByRole('button', { name: 'Ready · Publish' })).toBeInTheDocument();
  });

  it('C2(f) InProgress(3) reads "In progress" and "3 to review"', () => {
    const { container } = render(<StatusPill label={{ kind: 'in_progress', value: 3 }} />);
    expect(screen.getByText('In progress')).toBeInTheDocument();
    expect(screen.getByText('3 to review')).toBeInTheDocument();
    expect(container).toHaveTextContent(/^In progress3 to review$/);
  });
});

describe('C3 filters and search', () => {
  it('C3(a) clicking Published lists with filter "published" and shows the delivered counts', async () => {
    vi.mocked(listSessions).mockResolvedValue(list([{ label: 'Today', rows: [row(1, 'Alpha chair')] }], { open: 7, published: 6, all: 13 }));
    render(<SessionsPage navigate={() => undefined} />);
    await screen.findByText('Alpha chair');

    fireEvent.click(screen.getByTestId('filter-published'));

    await waitFor(() => expect(listSessions).toHaveBeenLastCalledWith({ filter: 'published', query: null }));
    expect(screen.getByTestId('count-open')).toHaveTextContent('7');
    expect(screen.getByTestId('count-published')).toHaveTextContent('6');
    expect(screen.getByTestId('count-all')).toHaveTextContent('13');
  });

  it('C3(b) typing "oak" lists with query "oak" after the debounce', async () => {
    render(<SessionsPage navigate={() => undefined} />);
    await waitFor(() => expect(listSessions).toHaveBeenCalledWith({ filter: 'open', query: null }));

    fireEvent.change(screen.getByLabelText('Search sessions by title or SKU'), { target: { value: 'oak' } });

    expect(listSessions).not.toHaveBeenCalledWith({ filter: 'open', query: 'oak' });
    await waitFor(() => expect(listSessions).toHaveBeenLastCalledWith({ filter: 'open', query: 'oak' }));
  });
});

describe('C4 camera dot', () => {
  it('C4(a) Connected("Sony ZV-E10") shows a green dot titled with the camera name', async () => {
    render(<CameraDot />);
    const button = await screen.findByTitle(/Sony ZV-E10/);
    expect(button).toHaveAttribute('data-state', 'connected');
    expect(button).toHaveAttribute('title', 'Camera connected: Sony ZV-E10');
  });

  it('C4(b) Disconnected shows a red dot and help that names USB mode and Imaging Edge', async () => {
    vi.mocked(cameraStatus).mockResolvedValue({ kind: 'disconnected' });
    render(<CameraDot />);
    const button = await screen.findByRole('button', { name: /Camera disconnected/ });
    expect(button).toHaveAttribute('data-state', 'disconnected');

    fireEvent.mouseEnter(button.parentElement!);

    const help = screen.getByRole('tooltip');
    expect(help).toHaveTextContent(/USB mode/);
    expect(help).toHaveTextContent(/Imaging Edge/);
  });

  it('C4(c) clicking the dot calls camera_retry once', async () => {
    vi.mocked(cameraStatus).mockResolvedValue({ kind: 'disconnected' });
    vi.mocked(cameraRetry).mockResolvedValue({ kind: 'connected', name: 'Sony ZV-E10' });
    render(<CameraDot />);
    const button = await screen.findByRole('button', { name: /Camera disconnected/ });

    fireEvent.click(button);

    await waitFor(() => expect(button).toHaveAttribute('data-state', 'connected'));
    expect(cameraRetry).toHaveBeenCalledTimes(1);
  });
});

describe('C5 new session', () => {
  it('C5(a) no match for "Brass floor lamp" shows the Create row', async () => {
    await startNewSession('Brass floor lamp');

    expect(await screen.findByText('Create “Brass floor lamp”')).toBeInTheDocument();
    expect(searchItems).toHaveBeenCalledWith('Brass floor lamp');
  });

  it('C5(b) creating with SKU BFL-1 creates the item, then its session, then opens it', async () => {
    vi.mocked(nextSku).mockResolvedValue('VF-0421');
    vi.mocked(createItem).mockResolvedValue(item(17, 'BFL-1', 'Brass floor lamp'));
    vi.mocked(createSession).mockResolvedValue({ id: 42 } as Awaited<ReturnType<typeof createSession>>);
    await startNewSession('Brass floor lamp');

    fireEvent.click(await screen.findByText('Create “Brass floor lamp”'));
    const sku = await screen.findByLabelText('SKU');
    await waitFor(() => expect(sku).toHaveValue('VF-0421'));
    fireEvent.change(sku, { target: { value: 'BFL-1' } });
    const create = screen.getByRole('button', { name: 'Create session' });
    await waitFor(() => expect(create).toBeEnabled());
    fireEvent.click(create);

    await waitFor(() => expect(window.location.hash).toBe('#/session/42'));
    expect(createItem).toHaveBeenCalledTimes(1);
    expect(createItem).toHaveBeenCalledWith({ sku: 'BFL-1', title: 'Brass floor lamp' });
    expect(createSession).toHaveBeenCalledWith(17);
    expect(vi.mocked(createItem).mock.invocationCallOrder[0]).toBeLessThan(vi.mocked(createSession).mock.invocationCallOrder[0]);
    expect(await screen.findByText('Ready for the first photo')).toBeInTheDocument();
  });

  it('C5(f) a proposal the shop already holds is flagged and Create stays disabled', async () => {
    vi.mocked(nextSku).mockResolvedValue('VF-0421');
    vi.mocked(searchItems).mockImplementation(async query =>
      query === 'VF-0421' ? [item(90, 'VF-0421', 'Walnut cabinet', 'linked')] : [],
    );
    await startNewSession('Brass floor lamp');

    fireEvent.click(await screen.findByText('Create “Brass floor lamp”'));

    expect(await screen.findByText('VF-0421 is already in the shop')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Create session' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: 'Create session' }));
    expect(createItem).not.toHaveBeenCalled();
  });

  it('C5(d) picking an existing item opens the session session_create returns, creating no item', async () => {
    vi.mocked(searchItems).mockResolvedValue([item(5, 'OAK-1', 'Oak sideboard')]);
    vi.mocked(createSession).mockResolvedValue({ id: 9 } as Awaited<ReturnType<typeof createSession>>);
    await startNewSession('Oak');

    fireEvent.click(await screen.findByText('Oak sideboard'));

    await waitFor(() => expect(window.location.hash).toBe('#/session/9'));
    expect(createSession).toHaveBeenCalledTimes(1);
    expect(createSession).toHaveBeenCalledWith(5);
    expect(createItem).not.toHaveBeenCalled();
    expect(screen.queryByText(/Create “/)).toBeNull();
  });
});

describe('C6 unassigned card', () => {
  it('C6(a) is absent at 0 and shows the count at 3', () => {
    const { container, rerender } = render(<UnassignedCard count={0} onOpen={() => undefined} />);
    expect(container).toBeEmptyDOMElement();

    rerender(<UnassignedCard count={3} onOpen={() => undefined} />);

    expect(screen.getByTestId('unassigned-card')).toBeInTheDocument();
    expect(screen.getByTestId('unassigned-count')).toHaveTextContent(/^3$/);
  });
});
