import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { Definition } from '@/services/attributes';
import type { ResearchSettings } from '@/services/settings';

vi.mock('@/services/research', () => ({
  researchSecretsState: vi.fn(),
  setResearchSecrets: vi.fn(),
  addResearchSite: vi.fn(),
  removeResearchSite: vi.fn(),
  reorderResearchSites: vi.fn(),
}));
vi.mock('@/services/attributes', () => ({ listDefinitions: vi.fn() }));
vi.mock('@/services/settings', () => ({ saveResearchSettings: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { NOT_A_SHOP_FIELD, ResearchSection } from '@/features/settings/ResearchSection';
import { listDefinitions } from '@/services/attributes';
import { addResearchSite, researchSecretsState, setResearchSecrets } from '@/services/research';

const SHOP = 'beyo-design.myshopify.com';

function research(extra: Partial<ResearchSettings> = {}): ResearchSettings {
  return {
    auto: true, quiet_seconds: 1, source_wait_seconds: 120, attribution_keys: ['beyo.designer'], trusted_sites: [], max_pages: 10,
    stages: { observe: { model: '', effort: '', image_long_edge: 1024 } },
    max_cost_per_run: '0.50', max_cost_per_day: '15', prices: [], language: 'English', record_days: 30,
    call_timeout_seconds: { model: 180, other: 60 }, providers: { model: '', visual_search: '', page_reader: '' },
    ...extra,
  };
}
function def(key: string): Definition {
  return { integration: 'shopify', key, label: key, core_type: 'text', choices: [], validations: '{}', group: 'details', description: null, writable: true };
}

beforeEach(() => {
  vi.mocked(researchSecretsState).mockResolvedValue({ model: false, search: false, reader: false });
  vi.mocked(listDefinitions).mockResolvedValue([]);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const siteRows = () => within(screen.getByRole('list', { name: 'Trusted sites' })).getAllByRole('listitem');

it('a19_c5a_shop_domain_first_without_remove_then_entries_in_order', () => {
  render(<ResearchSection research={research({ trusted_sites: ['pamono.com', '1stdibs.com'] })} shopDomain={SHOP} />);
  const rows = siteRows();
  expect(rows.map(row => row.getAttribute('data-site'))).toEqual([SHOP, 'pamono.com', '1stdibs.com']);
  expect(rows[0]).toHaveTextContent('always trusted');
  expect(within(rows[0]).queryByRole('button', { name: /^Remove/ })).not.toBeInTheDocument();
  expect(within(rows[1]).getByRole('button', { name: 'Remove pamono.com' })).toBeInTheDocument();
  expect(within(rows[2]).getByRole('button', { name: 'Remove 1stdibs.com' })).toBeInTheDocument();
});

it('a19_c5b_refused_add_shows_the_refusal_and_keeps_the_list', async () => {
  const refusal = `${SHOP} is your shop's own site and is always trusted`;
  vi.mocked(addResearchSite).mockRejectedValue(Object.assign(new Error(refusal), { code: 'invalid_data' }));
  render(<ResearchSection research={research({ trusted_sites: ['pamono.com', '1stdibs.com'] })} shopDomain={SHOP} />);
  fireEvent.change(screen.getByRole('textbox', { name: 'Add a trusted site' }), { target: { value: `https://${SHOP}/products/x` } });
  fireEvent.click(screen.getByRole('button', { name: 'Add site' }));
  expect(await screen.findByRole('alert')).toHaveTextContent(refusal);
  expect(addResearchSite).toHaveBeenCalledWith(`https://${SHOP}/products/x`);
  expect(siteRows().map(row => row.getAttribute('data-site'))).toEqual([SHOP, 'pamono.com', '1stdibs.com']);
});

it('a19_c5c_keys_are_never_shown_and_read_set_after_saving', async () => {
  vi.mocked(researchSecretsState).mockResolvedValueOnce({ model: false, search: true, reader: false });
  vi.mocked(setResearchSecrets).mockResolvedValue(undefined);
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  const model = screen.getByLabelText('Model key');
  const search = screen.getByLabelText('Visual search key');
  await waitFor(() => expect(document.querySelector('[data-key-state="model"]')).toHaveTextContent('not set'));
  expect(document.querySelector('[data-key-state="search"]')).toHaveTextContent(/^set$/);
  expect(model).toHaveValue('');
  expect(search).toHaveValue('');
  expect(model).toHaveAttribute('type', 'password');
  vi.mocked(researchSecretsState).mockResolvedValue({ model: true, search: true, reader: false });
  fireEvent.change(model, { target: { value: 'sk-test-0001' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save keys' }));
  await waitFor(() => expect(document.querySelector('[data-key-state="model"]')).toHaveTextContent(/^set$/));
  expect(setResearchSecrets).toHaveBeenCalledTimes(1);
  expect(setResearchSecrets).toHaveBeenCalledWith({ model_key: 'sk-test-0001', search_key: null, reader_key: null });
  expect(model).toHaveValue('');
  expect(document.body.textContent).not.toContain('sk-test-0001');
});

it('a19_c5d_warning_only_under_an_attribution_key_the_shop_does_not_have', async () => {
  vi.mocked(listDefinitions).mockResolvedValue([def('product.title'), def('custom.designer')]);
  render(<ResearchSection research={research({ attribution_keys: ['custom.designer', 'custom.maker'] })} shopDomain={SHOP} />);
  const list = screen.getByRole('list', { name: 'Attribution keys' });
  await waitFor(() => expect(within(list).getAllByText(NOT_A_SHOP_FIELD)).toHaveLength(1));
  const [designer, maker] = within(list).getAllByRole('listitem');
  expect(designer).toHaveAttribute('data-attribution', 'custom.designer');
  expect(within(designer).queryByText(NOT_A_SHOP_FIELD)).not.toBeInTheDocument();
  expect(maker).toHaveAttribute('data-attribution', 'custom.maker');
  expect(within(maker).getByText(NOT_A_SHOP_FIELD)).toBeInTheDocument();
});
