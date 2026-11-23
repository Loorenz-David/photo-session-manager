import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import type { ConversationsSettings } from '@/services/conversations';
import type { LabelsAccessState } from '@/services/labels';

/** L1·U4 (`plans/L1_labels_provider_and_marker.md` L1·C16): Settings → Pricing's Labels line (09C §4). */

vi.mock('@/services/conversations', () => ({
  conversationsSettingsGet: vi.fn(),
  conversationsSettingsSave: vi.fn(),
  conversationsKeySet: vi.fn(),
  conversationsTest: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onConversationsChanged: vi.fn().mockResolvedValue(() => undefined) }));

import { PricingSection } from '@/features/settings/PricingSection';
import { conversationsSettingsGet } from '@/services/conversations';

function settings(labels: LabelsAccessState): ConversationsSettings {
  return {
    server: 'https://pricing.example', server_default: 'https://pricing.example', key_saved: true, sound: true, banner: true,
    connection: { state: labels === 'off' ? 'no_key' : 'ready', store_name: 'Dev store', shop: 'shop-a.myshopify.com', visible_stores: [], last_ok_at: null, last_error: null, last_error_at: null },
    labels,
  };
}

async function linesFor(labels: LabelsAccessState): Promise<string[]> {
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings(labels));
  render(<PricingSection />);
  await screen.findByText('Test connection');
  await waitFor(() => expect(conversationsSettingsGet).toHaveBeenCalled());
  return [...document.querySelectorAll('p')].map(p => p.textContent ?? '').filter(text => text.startsWith('Labels:'));
}

afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('l1_c16a_allowed', async () => {
  expect(await linesFor('allowed')).toEqual(['Labels: allowed']);
});

it('l1_c16b_denied', async () => {
  expect(await linesFor('denied')).toEqual(["Labels: this key can't read labels. Ask a Pricing admin to add labels:read and labels:write."]);
});

it('l1_c16c_unknown', async () => {
  expect(await linesFor('unknown')).toEqual(["Labels: couldn't check. Press Test connection."]);
});

it('l1_c16d_off_shows_no_labels_line', async () => {
  expect(await linesFor('off')).toEqual([]);
});
