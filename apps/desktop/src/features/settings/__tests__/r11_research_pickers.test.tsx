import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { chooseValue } from '@/test/select';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { Definition } from '@/services/attributes';
import type { ResearchSettings } from '@/services/settings';

vi.mock('@/services/research', () => ({
  researchSecretsState: vi.fn(),
  setResearchSecrets: vi.fn(),
  addResearchSite: vi.fn(),
  removeResearchSite: vi.fn(),
  reorderResearchSites: vi.fn(),
  resetResearchSettings: vi.fn(),
  exportResearch: vi.fn(),
}));
vi.mock('@/services/attributes', () => ({ listDefinitions: vi.fn() }));
vi.mock('@/services/items', () => ({ searchItems: vi.fn() }));
vi.mock('@/services/settings', () => ({ saveResearchSettings: vi.fn(), pickFolder: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { NOT_A_SHOP_FIELD, ResearchSection } from '@/features/settings/ResearchSection';
import { listDefinitions } from '@/services/attributes';
import { researchSecretsState, resetResearchSettings } from '@/services/research';
import { saveResearchSettings } from '@/services/settings';

const SHOP = 'beyo-design.myshopify.com';
const NO_ATTRIBUTION = 'No designer or maker field is chosen: research will not suggest a designer or a maker.';
const LEVELS = ['N/A', 'None', 'Minor', 'Moderate', 'Major'];
const GRADES = ['Excellent', 'Very Good', 'Good', 'Worn'];

function research(extra: Partial<ResearchSettings> = {}): ResearchSettings {
  return {
    source_wait_seconds: 120, attribution_keys: [], fill_keys: [], extra_shop_hosts: [], detailed_condition: { key: '', levels: [] }, trusted_sites: [], max_pages: 10,
    stages: { observe: { model: '', effort: '', image_long_edge: 1024 } },
    max_cost_per_run: '0.50', max_cost_per_day: '15', prices: [], language: 'English', record_days: 30,
    call_timeout_seconds: { model: 180, other: 60 }, providers: { model: '', visual_search: '', page_reader: '' },
    ...extra,
  };
}
function def(key: string, label: string, extra: Partial<Definition> = {}): Definition {
  return { integration: 'shopify', key, label, core_type: 'text', choices: [], validations: '{}', group: 'details', description: null, writable: true, ...extra };
}
const choice = (key: string, label: string, choices: string[], group: Definition['group'] = 'details') => def(key, label, { core_type: 'choice', choices, group });
/** Five shop fields and one built-in Product field (never listed as a shop field). */
const FIVE = [def('product.title', 'Title', { integration: 'core', group: 'product' }), def('custom.wood', 'Wood'), def('custom.width', 'Width'),
  def('custom.era', 'Era'), def('custom.maker_x', 'Maker X'), def('custom.studio', 'Studio')];

beforeEach(() => {
  vi.mocked(researchSecretsState).mockResolvedValue({ model: false, search: false, reader: false });
  vi.mocked(listDefinitions).mockResolvedValue([]);
  vi.mocked(saveResearchSettings).mockImplementation(async value => value);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const group = (name: string) => screen.getByRole('group', { name });
const boxes = (name: string) => within(group(name)).queryAllByRole('checkbox');
const checkedKeys = (name: string) => within(group(name)).queryAllByRole('listitem')
  .filter(row => within(row).getByRole('checkbox').getAttribute('aria-checked') === 'true').map(row => row.getAttribute('data-field'));
const rowOf = (name: string, key: string) => within(group(name)).getAllByRole('listitem').find(row => row.getAttribute('data-field') === key)!;
const save = () => fireEvent.click(screen.getByRole('button', { name: 'Save research settings' }));
const saved = () => vi.mocked(saveResearchSettings).mock.calls[0][0];

it('a25_c28a_the_shop_fields_are_listed_and_the_saved_two_are_checked', async () => {
  vi.mocked(listDefinitions).mockResolvedValue(FIVE);
  render(<ResearchSection research={research({ attribution_keys: ['custom.era', 'custom.studio'] })} shopDomain={SHOP} />);
  await waitFor(() => expect(boxes('Designer and maker')).toHaveLength(5));
  const rows = within(group('Designer and maker')).getAllByRole('listitem');
  expect(rows.map(row => row.getAttribute('data-field'))).toEqual(['custom.wood', 'custom.width', 'custom.era', 'custom.maker_x', 'custom.studio']);
  expect(rows[0]).toHaveTextContent('Wood');
  expect(rows[0]).toHaveTextContent('custom.wood');
  expect(checkedKeys('Designer and maker')).toEqual(['custom.era', 'custom.studio']);
});

it('a25_c28b_designer_and_manufacturer_are_suggested_and_saved_only_on_save', async () => {
  vi.mocked(listDefinitions).mockResolvedValue([def('custom.designer', 'Designer'), def('custom.manufacturer', ' MANUFACTURER '),
    def('custom.wood', 'Wood'), def('custom.era', 'Era'), def('custom.width', 'Width')]);
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  await waitFor(() => expect(checkedKeys('Designer and maker')).toEqual(['custom.designer', 'custom.manufacturer']));
  expect(rowOf('Designer and maker', 'custom.designer')).toHaveTextContent('suggested — not saved yet');
  expect(rowOf('Designer and maker', 'custom.manufacturer')).toHaveTextContent('suggested — not saved yet');
  expect(rowOf('Designer and maker', 'custom.wood')).not.toHaveTextContent('suggested');
  expect(saveResearchSettings).not.toHaveBeenCalled();
  save();
  await waitFor(() => expect(saveResearchSettings).toHaveBeenCalledTimes(1));
  expect(saved().attribution_keys).toEqual(['custom.designer', 'custom.manufacturer']);
});

it('a25_c28c_nothing_chosen_says_research_will_not_suggest_a_designer', async () => {
  vi.mocked(listDefinitions).mockResolvedValue(FIVE);
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  await waitFor(() => expect(boxes('Designer and maker')).toHaveLength(5));
  expect(checkedKeys('Designer and maker')).toEqual([]);
  expect(within(group('Designer and maker')).getByText(NO_ATTRIBUTION)).toBeInTheDocument();
  cleanup();

  render(<ResearchSection research={research({ attribution_keys: ['custom.era'] })} shopDomain={SHOP} />);
  await waitFor(() => expect(checkedKeys('Designer and maker')).toEqual(['custom.era']));
  expect(within(group('Designer and maker')).queryByText(NO_ATTRIBUTION)).not.toBeInTheDocument();
  fireEvent.click(within(rowOf('Designer and maker', 'custom.era')).getByRole('checkbox'));
  expect(checkedKeys('Designer and maker')).toEqual([]);
  expect(within(group('Designer and maker')).getByText(NO_ATTRIBUTION)).toBeInTheDocument();
});

it('a25_c28d_an_unknown_key_is_marked_and_can_be_unchecked_and_nothing_is_preselected', async () => {
  vi.mocked(listDefinitions).mockResolvedValue([def('custom.maker', 'Maker'), def('custom.designer', 'Designer'), def('custom.wood', 'Wood')]);
  render(<ResearchSection research={research({ attribution_keys: ['custom.maker', 'old.designer'] })} shopDomain={SHOP} />);
  await waitFor(() => expect(boxes('Designer and maker')).toHaveLength(4));
  const unknown = rowOf('Designer and maker', 'old.designer');
  expect(unknown).toHaveTextContent(NOT_A_SHOP_FIELD);
  expect(rowOf('Designer and maker', 'custom.maker')).not.toHaveTextContent(NOT_A_SHOP_FIELD);
  expect(checkedKeys('Designer and maker')).toEqual(['custom.maker', 'old.designer']);
  expect(rowOf('Designer and maker', 'custom.designer')).not.toHaveTextContent('suggested');
  fireEvent.click(within(unknown).getByRole('checkbox'));
  expect(checkedKeys('Designer and maker')).toEqual(['custom.maker']);
  save();
  await waitFor(() => expect(saveResearchSettings).toHaveBeenCalledTimes(1));
  expect(saved().attribution_keys).toEqual(['custom.maker']);
});

it('a25_c28e_no_shop_linked_asks_to_connect_one_and_preselects_nothing', async () => {
  render(<ResearchSection research={research()} shopDomain={null} />);
  await waitFor(() => expect(listDefinitions).toHaveBeenCalled());
  expect(within(group('Designer and maker')).getByText('Connect a shop to choose fields')).toBeInTheDocument();
  expect(boxes('Designer and maker')).toHaveLength(0);
  save();
  await waitFor(() => expect(saveResearchSettings).toHaveBeenCalledTimes(1));
  expect(saved().attribution_keys).toEqual([]);
});

it('a27_c7b_fields_research_fills_start_unchecked_and_save_the_two_checked', async () => {
  vi.mocked(listDefinitions).mockResolvedValue(FIVE);
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  await waitFor(() => expect(boxes('Fields research fills')).toHaveLength(5));
  expect(checkedKeys('Fields research fills')).toEqual([]);
  fireEvent.click(within(rowOf('Fields research fills', 'custom.width')).getByRole('checkbox'));
  fireEvent.click(within(rowOf('Fields research fills', 'custom.era')).getByRole('checkbox'));
  save();
  await waitFor(() => expect(saveResearchSettings).toHaveBeenCalledTimes(1));
  expect(saved().fill_keys).toEqual(['custom.width', 'custom.era']);
  expect(screen.queryByRole('group', { name: 'Fields research leaves alone' })).not.toBeInTheDocument();
});

const hosts = () => within(screen.getByRole('list', { name: 'Extra shop address list' })).queryAllByRole('listitem').map(row => row.getAttribute('data-host'));
const addHost = (host: string) => {
  fireEvent.change(screen.getByRole('textbox', { name: 'Extra shop address' }), { target: { value: host } });
  fireEvent.click(screen.getByRole('button', { name: 'Add address' }));
};

it('a25_c30a_add_two_addresses_remove_the_first_and_save_the_second', async () => {
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  addHost('beyovintage.com');
  addHost('x.myshopify.com');
  expect(hosts()).toEqual(['beyovintage.com', 'x.myshopify.com']);
  fireEvent.click(screen.getByRole('button', { name: 'Remove extra address beyovintage.com' }));
  expect(hosts()).toEqual(['x.myshopify.com']);
  save();
  await waitFor(() => expect(saveResearchSettings).toHaveBeenCalledTimes(1));
  expect(saved().extra_shop_hosts).toEqual(['x.myshopify.com']);
});

it('a25_c30b_a_refused_save_shows_the_text_and_keeps_the_list', async () => {
  const refusal = 'extra shop address localhost: an address needs a dot';
  vi.mocked(saveResearchSettings).mockRejectedValue(Object.assign(new Error(refusal), { code: 'invalid_data' }));
  render(<ResearchSection research={research({ extra_shop_hosts: ['beyovintage.com'] })} shopDomain={SHOP} />);
  addHost('localhost');
  save();
  expect(await screen.findByRole('alert')).toHaveTextContent(refusal);
  expect(hosts()).toEqual(['beyovintage.com', 'localhost']);
});

it('a25_c31a_the_detailed_condition_table_saves_its_rows_in_order', async () => {
  vi.mocked(listDefinitions).mockResolvedValue([choice('custom.condition_scratch', 'Scratch', LEVELS, 'condition'),
    choice('custom.condition_veneer', 'Veneer', LEVELS, 'condition'), choice('custom.detailed_condition', 'Detailed condition', GRADES)]);
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  const field = screen.getByRole('combobox', { name: 'Detailed condition field' });
  await waitFor(() => expect(listDefinitions).toHaveBeenCalled());
  await chooseValue(field, 'custom.detailed_condition');
  const rows = within(screen.getByRole('list', { name: 'Detailed condition levels' })).getAllByRole('listitem');
  expect(rows.map(row => row.getAttribute('data-level'))).toEqual(LEVELS);
  const grade = (level: string) => screen.getByRole('combobox', { name: `Grade for ${level}` });
  await chooseValue(grade('None'), GRADES[1]);
  await chooseValue(grade('Minor'), GRADES[1]);
  await chooseValue(grade('Moderate'), GRADES[2]);
  await chooseValue(grade('N/A'), '__nothing__');
  save();
  await waitFor(() => expect(saveResearchSettings).toHaveBeenCalledTimes(1));
  expect(saved().detailed_condition).toEqual({ key: 'custom.detailed_condition', levels: [
    { level: 'N/A', choice: null }, { level: 'None', choice: 'Very Good' }, { level: 'Minor', choice: 'Very Good' }, { level: 'Moderate', choice: 'Good' },
  ] });
});

it('a25_c31b_no_field_chosen_says_not_set', async () => {
  vi.mocked(listDefinitions).mockResolvedValue([choice('custom.condition_scratch', 'Scratch', LEVELS, 'condition'), choice('custom.detailed_condition', 'Detailed condition', GRADES)]);
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  await waitFor(() => expect(listDefinitions).toHaveBeenCalled());
  expect(within(group('Detailed condition')).getByText('Not set: Detailed condition is filled like any other field.')).toBeInTheDocument();
  expect(screen.queryByRole('list', { name: 'Detailed condition levels' })).not.toBeInTheDocument();
});

it('a27_c13a_reset_keeps_the_fields_research_fills', async () => {
  const value = research({ stages: { observe: { model: 'gpt-6-luna', effort: 'low', image_long_edge: 1536 } },
    prices: [{ provider: 'openai', model: 'gpt-6-luna', unit: 'input', usd_per_million: '0.4', fallback_call_usd: null }] });
  vi.mocked(resetResearchSettings).mockResolvedValue(JSON.stringify(value));
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  fireEvent.click(screen.getByRole('button', { name: 'Reset research settings to defaults' }));
  const dialog = screen.getByRole('dialog', { name: 'Confirm research settings reset' });
  expect(within(within(dialog).getByRole('list', { name: 'Kept by the reset' })).getAllByRole('listitem').map(row => row.textContent)).toEqual([
    'Trusted sites', 'Designer and maker fields', 'Fields research fills', 'Extra shop addresses', 'The detailed-condition table', 'Language']);
  expect(dialog).toHaveTextContent('keys are untouched');
  expect(dialog).not.toHaveTextContent('Fields research leaves alone');
  fireEvent.click(within(dialog).getByRole('button', { name: 'Confirm reset' }));
  await waitFor(() => expect(screen.getByLabelText('observe model')).toHaveValue('gpt-6-luna'));
  expect(resetResearchSettings).toHaveBeenCalledTimes(1);
  expect(screen.getByLabelText('observe effort')).toHaveValue('low');
  expect(screen.getByLabelText('Price 1 model')).toHaveValue('gpt-6-luna');
  expect(screen.getByLabelText('Price 1 USD per million')).toHaveValue('0.4');
});

it('a25_c32b_cancelling_the_reset_calls_nothing_and_changes_nothing', () => {
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  fireEvent.click(screen.getByRole('button', { name: 'Reset research settings to defaults' }));
  fireEvent.click(within(screen.getByRole('dialog', { name: 'Confirm research settings reset' })).getByRole('button', { name: 'Cancel' }));
  expect(resetResearchSettings).not.toHaveBeenCalled();
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(screen.getByLabelText('observe model')).toHaveValue('');
  expect(screen.queryByLabelText('Price 1 model')).not.toBeInTheDocument();
});

it('a25_c33a_no_automatic_research_or_quiet_period_and_the_save_carries_neither', async () => {
  render(<ResearchSection research={research()} shopDomain={SHOP} />);
  expect(screen.queryByText(/Automatic research/i)).not.toBeInTheDocument();
  expect(screen.queryByText(/Quiet period/i)).not.toBeInTheDocument();
  expect(screen.queryByLabelText(/Automatic research/i)).not.toBeInTheDocument();
  expect(screen.queryByLabelText(/Quiet period/i)).not.toBeInTheDocument();
  save();
  await waitFor(() => expect(saveResearchSettings).toHaveBeenCalledTimes(1));
  expect(saved()).not.toHaveProperty('auto');
  expect(saved()).not.toHaveProperty('quiet_seconds');
});
