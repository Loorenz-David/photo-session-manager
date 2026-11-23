import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ConversationsSettings } from '@/services/conversations';
import type { LabelsAccess, SessionLabel } from '@/services/labels';
import type { SessionList } from '@/services/sessions';

/**
 * L1·U4 (`plans/L1_labels_provider_and_marker.md` L1·C19): "Generate label" in a session's ⋯ menu, in L1 only where it is
 * off (09C §6; projection r0 D5, D6).
 */

vi.mock('@/services/labels', () => ({ labelsStatus: vi.fn() }));
vi.mock('@/services/conversations', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/conversations')>()),
  conversationsSettingsGet: vi.fn(),
  conversationsCounts: vi.fn(async () => ({ here: 0, elsewhere: 0 })),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  sessionFilesMisplaced: vi.fn(async () => 0),
}));
vi.mock('@/services/research', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/research')>()), researchOverview: vi.fn(async () => []) }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onLabelsChanged: vi.fn(async () => () => undefined),
  onConversationsChanged: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => null), onExportProgress: vi.fn(async () => () => undefined), openExportFolder: vi.fn(), runExport: vi.fn() }));
vi.mock('@/features/publish/usePublish', () => ({ usePublish: () => ({ failedCount: 0, label: null, notifications: [], refresh: async () => undefined }) }));
vi.mock('@/features/publish/NotificationsDrawer', () => ({ NotificationsDrawer: () => null }));
vi.mock('@/features/publish/PublishProgress', () => ({ PublishProgress: () => null }));
vi.mock('@/features/settings/StorageSection', () => ({ DiskWarning: () => null }));
vi.mock('@/features/history/HistoryButtons', () => ({ HistoryButtons: () => null }));
vi.mock('@/features/shell/CameraDot', () => ({ CameraDot: () => null }));
vi.mock('@/features/shell/ThemeToggle', () => ({ ThemeToggle: () => null }));
vi.mock('@/features/conversations/ConversationsButton', () => ({ ConversationsButton: () => null }));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }));

import { SessionActionsMenu } from '@/features/sessions/DeleteSession';
import { Header } from '@/features/shell/Header';
import { conversationsSettingsGet } from '@/services/conversations';
import { labelsStatus } from '@/services/labels';
import { listSessions } from '@/services/sessions';

if (!('PointerEvent' in window)) {
  class PointerEventPolyfill extends MouseEvent {
    readonly pointerId: number;
    readonly pointerType: string;
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init);
      this.pointerId = init.pointerId ?? 1;
      this.pointerType = init.pointerType ?? 'mouse';
    }
  }
  Object.assign(window, { PointerEvent: PointerEventPolyfill });
}
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const ALLOWED: LabelsAccess = { state: 'allowed', reason: null };
const PUBLISH_FIRST = 'Publish first so Pricing can see it';
const DENIED = "This key can't read labels. Ask a Pricing admin to add labels:read and labels:write.";

function label(productId: string | null): SessionLabel {
  return { session_id: 2, product_id: productId, marker: null };
}

function storeGone(): ConversationsSettings {
  return {
    server: 'https://pricing.example', server_default: 'https://pricing.example', key_saved: true, sound: true, banner: true,
    connection: { state: 'store_gone', store_name: 'Dev store', shop: 'shop-a.myshopify.com', visible_stores: [], last_ok_at: null, last_error: null, last_error_at: null },
    labels: 'off',
  };
}

async function open(trigger: HTMLElement) {
  fireEvent.pointerDown(trigger, { button: 0 });
  await screen.findByRole('menuitem', { name: 'Delete session…' });
}

async function menuOf(sessionLabel: SessionLabel, access: LabelsAccess) {
  render(<SessionActionsMenu sessionId={2} onDeleted={() => undefined} label={sessionLabel} labelsAccess={access} />);
  await open(screen.getByRole('button', { name: 'Session actions' }));
}

/** The label item (disabled) and its text, or null when there is none. */
function labelItem(): { disabled: boolean; text: string } | null {
  const item = screen.queryByRole('menuitem', { name: /Generate label/ });
  if (!item) return null;
  return { disabled: item.getAttribute('aria-disabled') === 'true', text: item.textContent ?? '' };
}

beforeEach(() => {
  vi.mocked(conversationsSettingsGet).mockResolvedValue(storeGone());
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('l1_c19a_a_session_without_a_shop_product_says_publish_first', async () => {
  await menuOf(label(null), ALLOWED);
  expect(labelItem()).toEqual({ disabled: true, text: `Generate label${PUBLISH_FIRST}` });
});

it('l1_c19b_off_and_denied_say_why', async () => {
  await menuOf(label('P1'), { state: 'off', reason: 'no_key' });
  expect(labelItem()).toEqual({ disabled: true, text: 'Generate labelConnect Pricing in Settings → Pricing' });
  cleanup();
  await menuOf(label('P1'), { state: 'denied', reason: null });
  expect(labelItem()).toEqual({ disabled: true, text: `Generate label${DENIED}` });
  cleanup();
  await menuOf(label('P1'), { state: 'off', reason: 'store_gone' });
  await waitFor(() => expect(labelItem()).toEqual({ disabled: true, text: 'Generate labelThis key is no longer tied to shop-a.myshopify.com' }));
});

// Replaced by L3 (L3·C17(a), projection r0 D6): with access `allowed` or `unknown` the item is now shown, enabled.
it('l1_c19c_allowed_and_unknown_show_the_enabled_label_item', async () => {
  for (const access of [ALLOWED, { state: 'unknown', reason: null } as LabelsAccess]) {
    await menuOf(label('P1'), access);
    expect(labelItem()).toEqual({ disabled: false, text: 'Generate label' });
    expect(screen.getByRole('menuitem', { name: 'Delete session…' })).toBeInTheDocument();
    cleanup();
  }
});

it('l1_c19d_the_session_page_menu_says_publish_first', async () => {
  vi.mocked(listSessions).mockResolvedValue({ groups: [{ label: 'Today', rows: [{ id: 2, sku: 'SKU-2', item_type: 'Chair', title: 'Chair', arranged: false, label: { kind: 'ready' } }] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 0 } as SessionList);
  vi.mocked(labelsStatus).mockResolvedValue({ access: ALLOWED, labels: [label(null)] });
  render(<Header sessionId={2} theme="light" onThemeChange={() => undefined} onNewSession={() => undefined} />);
  await waitFor(() => expect(labelsStatus).toHaveBeenCalled());
  await open(screen.getByRole('button', { name: 'Session actions' }));
  await waitFor(() => expect(labelItem()).toEqual({ disabled: true, text: `Generate label${PUBLISH_FIRST}` }));
});
