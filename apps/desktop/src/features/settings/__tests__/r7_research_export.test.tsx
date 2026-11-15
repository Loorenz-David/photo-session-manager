import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ResearchSettings } from '@/services/settings';

vi.mock('@/services/research', () => ({
  researchSecretsState: vi.fn(),
  setResearchSecrets: vi.fn(),
  addResearchSite: vi.fn(),
  removeResearchSite: vi.fn(),
  reorderResearchSites: vi.fn(),
  exportResearch: vi.fn(),
}));
vi.mock('@/services/attributes', () => ({ listDefinitions: vi.fn() }));
vi.mock('@/services/items', () => ({ searchItems: vi.fn() }));
vi.mock('@/services/settings', () => ({ saveResearchSettings: vi.fn(), pickFolder: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { ResearchSection } from '@/features/settings/ResearchSection';
import { listDefinitions } from '@/services/attributes';
import { exportResearch, researchSecretsState } from '@/services/research';
import { pickFolder } from '@/services/settings';

function research(): ResearchSettings {
  return {
    source_wait_seconds: 120, attribution_keys: [], fill_keys: [], extra_shop_hosts: [], detailed_condition: { key: '', levels: [] }, trusted_sites: [], max_pages: 10,
    stages: {}, max_cost_per_run: '0.50', max_cost_per_day: '15', prices: [], language: 'English', record_days: 30,
    call_timeout_seconds: { model: 180, other: 60 }, providers: { model: '', visual_search: '', page_reader: '' },
  };
}

beforeEach(() => {
  vi.mocked(researchSecretsState).mockResolvedValue({ model: false, search: false, reader: false });
  vi.mocked(listDefinitions).mockResolvedValue([]);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('a21_c7a_export_research_with_a_range_and_a_folder_shows_the_returned_path', async () => {
  const zip = '/Users/me/Desktop/research-export-20261111-101500.zip';
  vi.mocked(pickFolder).mockResolvedValue('/Users/me/Desktop');
  vi.mocked(exportResearch).mockResolvedValue(zip);
  render(<ResearchSection research={research()} shopDomain={null} />);
  fireEvent.change(screen.getByLabelText('From'), { target: { value: '2026-11-10' } });
  fireEvent.change(screen.getByLabelText('To'), { target: { value: '2026-11-11' } });
  fireEvent.click(screen.getByRole('button', { name: 'Export research' }));
  await waitFor(() => expect(exportResearch).toHaveBeenCalledTimes(1));
  expect(exportResearch).toHaveBeenCalledWith({ kind: 'range', from: 20261110, to: 20261111 }, '/Users/me/Desktop');
  expect(await screen.findByText(`Exported to ${zip}`)).toBeInTheDocument();
});
