import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { ActionButton } from '@/components/ui/action-button';
import { SkeletonRows } from '@/components/pending/SkeletonRows';
import { usePageData } from '@/lib/pending';
import { SessionsPage } from '@/features/sessions/SessionsPage';
import { WorkspacePage } from '@/features/workspace/WorkspacePage';
import { HealthPage } from '@/features/health/HealthPage';
import { SettingsPage } from '@/features/settings/SettingsPage';
import { listSessions } from '@/services/sessions';
import { getSessionView } from '@/services/photos';
import { getHealth, getAppPaths } from '@/services/health';
import { getSettings } from '@/services/settings';
import type { SessionList } from '@/services/sessions';
import type { SessionView } from '@/services/photos';
import type { Settings } from '@/services/settings';

vi.mock('@/services/sessions', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/sessions')>()), listSessions: vi.fn() }));
vi.mock('@/services/photos', () => ({ getSessionView: vi.fn() }));
vi.mock('@/services/health', () => ({ getHealth: vi.fn(), getAppPaths: vi.fn() }));
vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), pickFolder: vi.fn(), saveSettings: vi.fn() }));
vi.mock('@/services/events', () => ({ onSessionsChanged: vi.fn(async () => () => undefined), onWorkspaceEvents: vi.fn(async () => () => undefined), onFileDrop: vi.fn(async () => () => undefined) }));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), dismiss: vi.fn() } }));

globalThis.ResizeObserver ??= class { observe() {} unobserve() {} disconnect() {} };
Element.prototype.scrollIntoView ??= function scrollIntoView() {};

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('C1(a) a pending action disables immediately and blocks a second submit', async () => {
  const pending = deferred<void>();
  const fn = vi.fn(() => pending.promise);
  render(<ActionButton action={fn}>Save</ActionButton>);
  const button = screen.getByRole('button', { name: 'Save' });
  fireEvent.click(button);
  expect(button).toHaveAttribute('aria-busy', 'true');
  expect(button).toBeDisabled();
  fireEvent.click(button);
  expect(fn).toHaveBeenCalledTimes(1);
  await act(async () => pending.resolve());
  expect(button).not.toHaveAttribute('aria-busy');
  expect(button).toBeEnabled();
});

it('C1(b) a rejected action reports its error and enables the button again', async () => {
  const pending = deferred<void>();
  const onError = vi.fn();
  render(<ActionButton action={() => pending.promise} onError={onError}>Save</ActionButton>);
  const button = screen.getByRole('button', { name: 'Save' });
  fireEvent.click(button);
  await act(async () => pending.reject(new Error('nope')));
  expect(onError).toHaveBeenCalledWith(expect.objectContaining({ message: 'nope' }));
  expect(button).toBeEnabled();
  expect(button).not.toHaveAttribute('aria-busy');
});

it('C2(a) page data shows skeletons only before its first result and keeps content on reload', async () => {
  const first = deferred<{ n: number }>();
  const second = deferred<{ n: number }>();
  const loader = vi.fn().mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
  function Page() {
    const { data, loading, reload } = usePageData<{ n: number }>(loader);
    return <section aria-label="Data" aria-busy={loading ? 'true' : undefined}>{loading ? <SkeletonRows n={2} /> : <><p>Value {data?.n}</p><button onClick={() => void reload()}>Reload</button></>}</section>;
  }
  render(<Page />);
  const page = screen.getByRole('region', { name: 'Data' });
  expect(page).toHaveAttribute('aria-busy', 'true');
  expect(page.querySelectorAll('[data-slot="skeleton"]')).toHaveLength(2);
  await act(async () => first.resolve({ n: 1 }));
  expect(screen.getByText('Value 1')).toBeInTheDocument();
  expect(page.querySelectorAll('[data-slot="skeleton"]')).toHaveLength(0);
  fireEvent.click(screen.getByRole('button', { name: 'Reload' }));
  expect(screen.getByText('Value 1')).toBeInTheDocument();
  expect(page.querySelectorAll('[data-slot="skeleton"]')).toHaveLength(0);
  await act(async () => second.resolve({ n: 2 }));
  expect(screen.getByText('Value 2')).toBeInTheDocument();
});

it('C2(b) sessions list shows rows only after the first list result', async () => {
  const pending = deferred<SessionList>();
  vi.mocked(listSessions).mockReturnValue(pending.promise);
  render(<SessionsPage navigate={() => undefined} />);
  expect(document.querySelectorAll('[data-slot="skeleton"]').length).toBeGreaterThanOrEqual(3);
  expect(screen.queryByText('No sessions yet')).not.toBeInTheDocument();
  expect(document.querySelectorAll('[data-testid^="session-row-"]')).toHaveLength(0);
  const row = (id: number, title: string) => ({ id, sku: `SKU-${id}`, item_type: '', title, main_count: 0, condition_count: 0, unreviewed_count: 0, status: 'in_progress', label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), cover_preview_path: null, publish: null });
  await act(async () => pending.resolve({ groups: [{ label: 'Today', rows: [row(2, 'Oak chair'), row(3, 'Walnut desk')] }], counts: { open: 2, published: 0, all: 2 }, unassigned_count: 0 } as SessionList));
  expect(document.querySelectorAll('[data-testid^="session-row-"]')).toHaveLength(2);
  expect(screen.getByText('Oak chair')).toBeInTheDocument();
  expect(screen.getByText('Walnut desk')).toBeInTheDocument();
  expect(document.querySelectorAll('[data-slot="skeleton"]')).toHaveLength(0);
});

it('C2(c) workspace shows a stage and eight photo placeholders before two photos arrive', async () => {
  const pending = deferred<SessionView>();
  vi.mocked(getSessionView).mockReturnValue(pending.promise);
  render(<WorkspacePage sessionId={5} assignOpen={false} onAssignOpenChange={() => undefined} navigate={() => undefined} />);
  const panel = screen.getByLabelText('Photos');
  expect(panel.querySelectorAll('[data-slot="skeleton"]')).toHaveLength(8);
  expect(document.querySelector('.stage-column [data-slot="skeleton"]')).toBeInTheDocument();
  const photo = (id: number) => ({ id, session_id: 5, classification: 'main', position: id, captured_at: id, original_preview_path: null, adjusted_preview_path: null, orientation: 1, source: 'capture', integrity: 'ok', upload_state: null });
  await act(async () => pending.resolve({ session: { id: 5, item_id: null, published_at: null, published_seq: 0, change_seq: 0, dirty: false, last_activity_at: 0, created_at: 0, photo_count: 2, unreviewed_count: 0, main_count: 2, condition_count: 0, publish: null }, item: null, photos: [photo(1), photo(2)] } as SessionView));
  expect(within(screen.getByLabelText('Photos')).getAllByRole('button', { name: /^Photo 0/ })).toHaveLength(2);
  expect(document.querySelectorAll('[data-slot="skeleton"]')).toHaveLength(0);
});

it('C2(d) health and settings show placeholders, then their loaded content', async () => {
  const health = deferred<Awaited<ReturnType<typeof getHealth>>>();
  const paths = deferred<Awaited<ReturnType<typeof getAppPaths>>>();
  const settings = deferred<Settings>();
  vi.mocked(getHealth).mockReturnValue(health.promise);
  vi.mocked(getAppPaths).mockReturnValue(paths.promise);
  vi.mocked(getSettings).mockReturnValue(settings.promise);
  const shown = render(<><HealthPage /><SettingsPage /></>);
  expect(shown.container.querySelectorAll('[data-slot="skeleton"]').length).toBeGreaterThanOrEqual(2);
  expect(screen.queryByText('Loading…')).not.toBeInTheDocument();
  await act(async () => {
    health.resolve({ items: [{ code: 'nudge_listening', ok: true, detail: 'Listening' }] });
    paths.resolve({ data_dir: '/tmp/psm', database: '/tmp/psm/session_manager.db', settings: '/tmp/psm/settings.json' });
    settings.resolve({ document_id: '/tmp/session', preview_recipe: 'PSM Preview', web_recipe: 'PSM Web', style_name: 'PSM House', export_folder: '/tmp/export', preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null, integration: null });
  });
  expect(screen.getByText('Listening')).toBeInTheDocument();
  expect(screen.getByLabelText('Preview recipe')).toHaveValue('PSM Preview');
  expect(shown.container.querySelectorAll('[data-slot="skeleton"]')).toHaveLength(0);
});
