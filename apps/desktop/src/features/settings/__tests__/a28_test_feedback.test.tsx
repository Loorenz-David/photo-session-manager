import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { CameraStatus } from '@/services/camera';
import type { ConversationList, ConversationsSettings } from '@/services/conversations';
import type { Settings } from '@/services/settings';

// A28 (owner 2026-11-20): "on test connection nothing happens, also on the conversation drawer i only see the open
// settings connect pricing setting -> pricing"; "we should also fix the test camera connection which also doesn't work
// properly". Each test button now answers visibly every time; the panel says why it cannot connect.

vi.mock('@/services/camera', () => ({
  cameraSupportGet: vi.fn(), cameraSupportInstall: vi.fn(), cameraFtpPasswordGenerate: vi.fn(), cameraFtpPasswordGet: vi.fn(),
  cameraFtpAddress: vi.fn(), cameraSecretsSet: vi.fn(), cameraFingerprintConfirm: vi.fn(), cameraFingerprintReject: vi.fn(),
  cameraStatus: vi.fn(), cameraTestConnection: vi.fn(),
}));
vi.mock('@/services/conversations', () => ({
  conversationsSettingsGet: vi.fn(), conversationsSettingsSave: vi.fn(), conversationsKeySet: vi.fn(), conversationsTest: vi.fn(),
  conversationsList: vi.fn(), conversationsCount: vi.fn(),
}));
vi.mock('@/services/settings', () => ({ pickArchive: vi.fn(), getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), pickSessionFile: vi.fn(), listStyles: vi.fn() }));
vi.mock('@/services/events', () => ({
  onConversationsChanged: vi.fn().mockResolvedValue(() => undefined),
  onFingerprintPending: vi.fn().mockResolvedValue(() => undefined),
}));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { CameraSection } from '@/features/settings/CameraSection';
import { PricingSection } from '@/features/settings/PricingSection';
import { SettingsPage } from '@/features/settings/SettingsPage';
import { openSettingsSection } from '@/features/settings/SettingsNav';
import { ConversationsContext, useConversations } from '@/features/conversations/useConversations';
import { ConversationsButton } from '@/features/conversations/ConversationsButton';
import { ConversationsPanel } from '@/features/conversations/ConversationsPanel';
import { cameraStatus, cameraSupportGet, cameraTestConnection } from '@/services/camera';
import { conversationsList, conversationsSettingsGet, conversationsTest } from '@/services/conversations';
import { getSettings } from '@/services/settings';

const SHOP = 'shop-a.myshopify.com';
const DEFAULT_SERVER = 'https://pricing.beyoworkaroundtheclock.com';
const ports = { receiver_port: 2121, passive_port_start: 50000, passive_port_end: 50009 };
const shown: Settings = { camera: ports, document_id: null, effective_document_id: '/tmp/session', default_document_id: '/tmp/session', preview_recipe: 'PSM Preview', output_format: { long_edge: 2048, quality: 90 }, style_name: '', default_shape: '1:1', master_recipe: 'PSM Master', export_folder: '/tmp/out', preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null, integration: null } as Settings;

function pricing(connection: Partial<ConversationsSettings['connection']> = {}): ConversationsSettings {
  return {
    server: DEFAULT_SERVER, server_default: DEFAULT_SERVER, key_saved: true, sound: true, banner: true,
    connection: { state: 'ready', store_name: 'Beyo Vintage', shop: SHOP, visible_stores: [], last_ok_at: null, last_error: null, last_error_at: null, ...connection },
  };
}

function list(extra: Partial<ConversationList> = {}): ConversationList {
  return { state: 'ready', problem: null, rows: [], news_elsewhere: 0, ...extra };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(r => { resolve = r; });
  return { promise, resolve };
}

/** Local wall-clock time of `hh:mm:ss` today, as the tests' "now". */
function at(hh: number, mm: number, ss: number): Date {
  const now = new Date();
  now.setHours(hh, mm, ss, 0);
  return now;
}

beforeEach(() => {
  vi.mocked(cameraSupportGet).mockResolvedValue({ make: 'sony', version: '2.02.00', sidecar_path: '/x/sidecar' });
  vi.mocked(cameraStatus).mockResolvedValue({ kind: 'disconnected' });
  vi.mocked(cameraTestConnection).mockResolvedValue({ holder: null, programs: [], sentence: null });
  vi.mocked(conversationsSettingsGet).mockResolvedValue(pricing());
  vi.mocked(conversationsList).mockResolvedValue(list());
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.useRealTimers();
  window.location.hash = '';
});

// ---------- Settings → Pricing: Test connection ----------

it('A28(a) Test connection says "Checking…" while it runs, then the line and the time of that press, again on a second press with the same words', async () => {
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(at(14, 3, 7));
  const first = deferred<ConversationsSettings>();
  vi.mocked(conversationsTest).mockReturnValueOnce(first.promise);
  render(<PricingSection />);
  const button = await screen.findByRole('button', { name: 'Test connection' });
  fireEvent.click(button);
  expect(await screen.findByRole('button', { name: 'Checking…' })).toBeInTheDocument();
  expect(screen.queryByTestId('pricing-checked-at')).toBeNull();
  await act(async () => { first.resolve(pricing()); });
  expect(await screen.findByTestId('pricing-checked-at')).toHaveTextContent('Checked at 14:03:07');
  expect(screen.getByTestId('pricing-connection')).toHaveTextContent(`Connected to Beyo Vintage (${SHOP})`);

  vi.setSystemTime(at(14, 3, 12));
  vi.mocked(conversationsTest).mockResolvedValueOnce(pricing());
  fireEvent.click(screen.getByRole('button', { name: 'Test connection' }));
  await waitFor(() => expect(screen.getByTestId('pricing-checked-at')).toHaveTextContent('Checked at 14:03:12'));
  expect(screen.getByTestId('pricing-connection')).toHaveTextContent(`Connected to Beyo Vintage (${SHOP})`);
});

it('A28(b) a Test connection that cannot reach Pricing keeps its words and still says when it was checked', async () => {
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(at(9, 0, 5));
  vi.mocked(conversationsTest).mockRejectedValueOnce(Object.assign(new Error('Could not reach Pricing'), { code: 'provider' }));
  render(<PricingSection />);
  fireEvent.click(await screen.findByRole('button', { name: 'Test connection' }));
  expect(await screen.findByTestId('pricing-checked-at')).toHaveTextContent('Checked at 09:00:05');
  expect(screen.getByTestId('pricing-connection')).toHaveTextContent('Could not reach Pricing');
});

// ---------- the conversations panel when it cannot connect ----------

function Harness() {
  const conversations = useConversations(undefined);
  return (
    <ConversationsContext.Provider value={conversations}>
      <ConversationsButton />
      <ConversationsPanel />
    </ConversationsContext.Provider>
  );
}

async function openPanel() {
  render(<Harness />);
  fireEvent.click(screen.getByRole('button', { name: 'Conversations' }));
  return screen.findByRole('dialog', { name: 'Conversations' });
}

it('A28(c) a key tied to another store: the panel says so in the Settings words, and its button opens Settings on Pricing', async () => {
  vi.mocked(conversationsList).mockResolvedValue(list({ state: 'store_unmatched' }));
  vi.mocked(conversationsSettingsGet).mockResolvedValue(pricing({ state: 'store_unmatched', store_name: null, visible_stores: ['Dev store'] }));
  vi.mocked(getSettings).mockResolvedValue(shown);
  const panel = await openPanel();
  expect(await within(panel).findByText(`This Pricing key isn't tied to ${SHOP}. It can see: Dev store.`)).toBeInTheDocument();
  expect(within(panel).queryByText('Connect Pricing in Settings → Pricing')).toBeNull();
  fireEvent.click(within(panel).getByRole('button', { name: 'Open Settings → Pricing' }));
  expect(window.location.hash).toBe('#/settings');
  cleanup();
  render(<SettingsPage />);
  await waitFor(() => expect(document.querySelector('[data-section="pricing"]')).toHaveAttribute('data-active', 'true'));
  expect(document.querySelector('[data-section="capture-one"]')).toHaveAttribute('data-active', 'false');
});

it('A28(d) no Shopify connection: the panel says to connect Shopify first; no key at all still asks to connect Pricing', async () => {
  vi.mocked(conversationsList).mockResolvedValue(list({ state: 'no_shop' }));
  vi.mocked(conversationsSettingsGet).mockResolvedValue(pricing({ state: 'no_shop', store_name: null, shop: null }));
  let panel = await openPanel();
  expect(await within(panel).findByText('Connect Shopify first; conversations are matched to its products.')).toBeInTheDocument();
  cleanup();
  vi.mocked(conversationsList).mockResolvedValue(list({ state: 'no_key' }));
  vi.mocked(conversationsSettingsGet).mockResolvedValue(pricing({ state: 'no_key', store_name: null }));
  panel = await openPanel();
  await waitFor(() => expect(conversationsSettingsGet).toHaveBeenCalled());
  expect(await within(panel).findByText('Connect Pricing in Settings → Pricing')).toBeInTheDocument();
});

it('A28(e) a refused key: the problem keeps its words and offers the way to Settings → Pricing', async () => {
  vi.mocked(conversationsList).mockResolvedValue(list({ state: 'refused', problem: 'Pricing refused the key: check Settings → Pricing' }));
  const panel = await openPanel();
  expect(await within(panel).findByRole('alert')).toHaveTextContent('Pricing refused the key: check Settings → Pricing');
  expect(within(panel).getByRole('button', { name: 'Open Settings → Pricing' })).toBeInTheDocument();
});

it('A28(f) Open Settings → Pricing while Settings is on screen switches to the Pricing section', async () => {
  vi.mocked(getSettings).mockResolvedValue(shown);
  window.location.hash = '#/settings';
  render(<SettingsPage />);
  await waitFor(() => expect(document.querySelector('[data-section="capture-one"]')).toHaveAttribute('data-active', 'true'));
  act(() => openSettingsSection('pricing'));
  expect(document.querySelector('[data-section="pricing"]')).toHaveAttribute('data-active', 'true');
  expect(document.querySelector('[data-section="capture-one"]')).toHaveAttribute('data-active', 'false');
});

// ---------- Settings → Camera: Test camera connection ----------

async function checkCamera(status: CameraStatus) {
  vi.mocked(cameraStatus).mockResolvedValue(status);
  render(<CameraSection ports={ports} onPortsChange={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: 'Test camera connection' }));
  return screen.findByTestId('camera-check');
}

it.each<[CameraStatus, string]>([
  [{ kind: 'connected', name: 'ILCE-7RM5', link: 'usb' }, 'Connected: ILCE-7RM5 on the cable'],
  [{ kind: 'connected', name: 'ILCE-7RM5', link: 'wireless' }, 'Connected: ILCE-7RM5 on Wi-Fi'],
  [{ kind: 'connecting', link: 'wireless' }, 'Connecting on Wi-Fi…'],
  [{ kind: 'busy', reason: 'transfer' }, 'Receiving a photo'],
  [{ kind: 'disconnected' }, 'No camera found. Check that the camera is on and the cable is plugged in, or that Wi-Fi is set up.'],
])('A28(g) Test camera connection says the camera state first: %o', async (status, words) => {
  const check = await checkCamera(status);
  expect(check.querySelector('p')).toHaveTextContent(words);
});

it('A28(h) Test camera connection says "Checking…" while it runs and then the time of that press', async () => {
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(at(16, 45, 30));
  const pending = deferred<CameraStatus>();
  vi.mocked(cameraStatus).mockReturnValueOnce(pending.promise);
  render(<CameraSection ports={ports} onPortsChange={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: 'Test camera connection' }));
  expect(await screen.findByRole('button', { name: 'Checking…' })).toBeInTheDocument();
  await act(async () => { pending.resolve({ kind: 'connected', name: 'ILCE-7RM5', link: 'usb' }); });
  const check = await screen.findByTestId('camera-check');
  expect(within(check).getByText('Checked at 16:45:30')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Test camera connection' })).toBeInTheDocument();
});

it('A28(i) a camera held by another program: the needed action is said once, not repeated by the holder report', async () => {
  vi.mocked(cameraTestConnection).mockResolvedValue({ holder: 'Capture One', programs: ['Capture One'], sentence: 'Capture One is holding the camera. Quit it or press Take the cable back.' });
  const check = await checkCamera({ kind: 'needs_action', reason: 'held_by_other_program', holder: 'Capture One', message: 'Capture One is holding the camera. Quit it or press Take the cable back.', link: 'usb' });
  expect(within(check).getAllByText('Capture One is holding the camera. Quit it or press Take the cable back.')).toHaveLength(1);
  expect(within(check).queryByText('Capture One is holding the camera')).toBeNull();
});
