import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { Definition } from '@/services/attributes';
import type { ResearchSettings } from '@/services/settings';

vi.mock('@/services/research', () => ({
  researchSecretsState: vi.fn(), setResearchSecrets: vi.fn(), addResearchSite: vi.fn(),
  removeResearchSite: vi.fn(), reorderResearchSites: vi.fn(), resetResearchSettings: vi.fn(), exportResearch: vi.fn(),
}));
vi.mock('@/services/attributes', () => ({ listDefinitions: vi.fn() }));
vi.mock('@/services/items', () => ({ searchItems: vi.fn() }));
vi.mock('@/services/settings', () => ({ saveResearchSettings: vi.fn(), pickFolder: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { ResearchSection } from '@/features/settings/ResearchSection';
import { listDefinitions } from '@/services/attributes';
import { researchSecretsState } from '@/services/research';
import { saveResearchSettings } from '@/services/settings';

const SHOP = 'shop.example';
const setting = (extra: Partial<ResearchSettings> = {}): ResearchSettings => ({
  source_wait_seconds: 120, attribution_keys: [], fill_keys: [], extra_shop_hosts: [], detailed_condition: { key: '', levels: [] },
  trusted_sites: [], max_pages: 10, stages: { observe: { model: '', effort: '', image_long_edge: 1024 } },
  max_cost_per_run: '0.50', max_cost_per_day: '15', prices: [], language: 'English', record_days: 30,
  call_timeout_seconds: { model: 180, other: 60 }, providers: { model: '', visual_search: '', page_reader: '' }, ...extra,
});
const def = (key: string, label: string, extra: Partial<Definition> = {}): Definition => ({
  integration: 'shopify', key, label, core_type: 'text', choices: [], validations: '{}', group: 'details',
  description: null, writable: true, ...extra,
});
const fields = [
  def('custom.text', 'Text'), def('custom.choice', 'Choice', { core_type: 'choice', choices: ['A', 'B'] }),
  def('custom.media', 'Media', { core_type: 'media_list' }), def('custom.reference', 'Reference', { core_type: 'reference' }),
  def('custom.rich', 'Rich', { core_type: 'rich_text' }), def('custom.money', 'Money', { core_type: 'money' }),
  def('custom.rating', 'Rating', { core_type: 'rating' }), def('custom.locked', 'Locked', { writable: false }),
];
const group = (name: string) => screen.getByRole('group', { name });
const rows = (name: string) => within(group(name)).queryAllByRole('listitem');
const row = (name: string, key: string) => rows(name).find(element => element.getAttribute('data-field') === key)!;
const toggle = (name: string, key: string) => fireEvent.click(within(row(name, key)).getByRole('checkbox'));
const save = () => fireEvent.click(screen.getByRole('button', { name: 'Save research settings' }));

beforeEach(() => {
  vi.mocked(researchSecretsState).mockResolvedValue({ model: false, search: false, reader: false });
  vi.mocked(listDefinitions).mockResolvedValue(fields);
  vi.mocked(saveResearchSettings).mockImplementation(async value => value);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('a27_c10a_only_fillable_fields_are_offered', async () => {
  render(<ResearchSection research={setting()} shopDomain={SHOP} />);
  await waitFor(() => expect(rows('Fields research fills')).toHaveLength(2));
  expect(rows('Fields research fills').map(element => element.getAttribute('data-field'))).toEqual(['custom.text', 'custom.choice']);
  expect(rows('Designer and maker')).toHaveLength(8);
});

it('a27_c10b_a_saved_unfillable_key_can_be_removed', async () => {
  render(<ResearchSection research={setting({ fill_keys: ['custom.rating'] })} shopDomain={SHOP} />);
  await waitFor(() => expect(rows('Fields research fills')).toHaveLength(3));
  expect(row('Fields research fills', 'custom.rating')).toHaveTextContent('Research never fills this kind of field');
  expect(within(row('Fields research fills', 'custom.rating')).getByRole('checkbox')).toBeChecked();
  toggle('Fields research fills', 'custom.rating');
  save();
  await waitFor(() => expect(saveResearchSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(saveResearchSettings).mock.calls[0][0].fill_keys).toEqual([]);
});

it('a27_c11a_empty_list_explains_why_research_fills_nothing', async () => {
  render(<ResearchSection research={setting()} shopDomain={SHOP} />);
  await waitFor(() => expect(rows('Fields research fills')).toHaveLength(2));
  expect(within(group('Fields research fills')).getByText('No field is chosen: research will not fill anything.')).toBeInTheDocument();
  toggle('Fields research fills', 'custom.text');
  expect(within(group('Fields research fills')).queryByText('No field is chosen: research will not fill anything.')).not.toBeInTheDocument();
  expect(group('Fields research fills')).toHaveTextContent('Research suggests values only for the fields ticked here. A new field in your shop stays empty until you tick it.');
});

it('a27_c12a_unchosen_designer_is_warned_by_label', async () => {
  vi.mocked(listDefinitions).mockResolvedValue([def('custom.designer', 'Designer'), def('custom.wood', 'Wood')]);
  render(<ResearchSection research={setting({ attribution_keys: ['custom.designer'], fill_keys: ['custom.wood'] })} shopDomain={SHOP} />);
  await waitFor(() => expect(rows('Fields research fills')).toHaveLength(2));
  expect(within(group('Designer and maker')).getByText('Not among the fields research fills, so research will not suggest it: Designer')).toBeInTheDocument();
});

it('a27_c12b_choosing_designer_removes_its_warning', async () => {
  vi.mocked(listDefinitions).mockResolvedValue([def('custom.designer', 'Designer'), def('custom.wood', 'Wood')]);
  render(<ResearchSection research={setting({ attribution_keys: ['custom.designer'], fill_keys: ['custom.wood'] })} shopDomain={SHOP} />);
  await waitFor(() => expect(rows('Fields research fills')).toHaveLength(2));
  toggle('Fields research fills', 'custom.designer');
  expect(within(group('Designer and maker')).queryByText('Not among the fields research fills, so research will not suggest it: Designer')).not.toBeInTheDocument();
});

it('a27_c12c_choosing_the_grade_removes_its_warning', async () => {
  vi.mocked(listDefinitions).mockResolvedValue([def('custom.detailed_condition', 'Detailed condition', { core_type: 'choice', choices: ['A', 'B'] })]);
  render(<ResearchSection research={setting({ detailed_condition: { key: 'custom.detailed_condition', levels: [] } })} shopDomain={SHOP} />);
  await waitFor(() => expect(rows('Fields research fills')).toHaveLength(1));
  expect(within(group('Detailed condition')).getByText('Not among the fields research fills, so the detailed condition will not be worked out.')).toBeInTheDocument();
  toggle('Fields research fills', 'custom.detailed_condition');
  expect(within(group('Detailed condition')).queryByText('Not among the fields research fills, so the detailed condition will not be worked out.')).not.toBeInTheDocument();
  expect(group('Detailed condition')).toHaveTextContent('Levels with no value do not hold the grade back.');
});
