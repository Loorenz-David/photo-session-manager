import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { Definition } from '@/services/attributes';
import type { ResearchSettings } from '@/services/settings';
import type { WriterExample, WriterSettings } from '@/services/writer';
import { chooseValue, optionsOf } from '@/test/select';

vi.mock('@/services/writer', () => ({
  writerSettingsGet: vi.fn(),
  writerSettingsSave: vi.fn(),
  writerInstructionReset: vi.fn(),
  writerPreview: vi.fn(),
  writerSettingsExport: vi.fn(),
  writerSettingsImport: vi.fn(),
}));
vi.mock('@/services/attributes', () => ({ listDefinitions: vi.fn() }));
vi.mock('@/services/settings', () => ({}));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { SettingsNav } from '@/features/settings/SettingsNav';
import { WriterSection } from '@/features/settings/WriterSection';
import { listDefinitions } from '@/services/attributes';
import { toast } from 'sonner';
import { writerInstructionReset, writerPreview, writerSettingsExport, writerSettingsGet, writerSettingsImport, writerSettingsSave } from '@/services/writer';

const SHIPPED_EN = 'You write product texts for a small Swedish shop.';
const SHIPPED_SV = 'Du skriver produkttexter för en liten svensk butik.';
const EXAMPLE_D: WriterExample = {
  name: 'D', fields: 'fåtölj; teak', title: 'Dansk teakfåtölj', description: 'Fåtölj.', condition_report: 'Mycket gott.',
  seo_title: 'Dansk teakfåtölj, 1960-tal', seo_description: 'Dansk fåtölj i teak.',
};
const ZERO = { answered: 0, dismissed: 0 };

function defaults(extra: Partial<WriterSettings> = {}): WriterSettings {
  return {
    auto: true, language: 'English', model: 'gpt-6.1-sol', effort: 'medium',
    instructions: {
      English: { text: null, effective_instruction: SHIPPED_EN, is_default: true, versions: [] },
      Swedish: { text: null, effective_instruction: SHIPPED_SV, is_default: true, versions: [] },
    },
    examples: { English: [], Swedish: [EXAMPLE_D] },
    banned_extra: [],
    lengths: { title_max: 70, description_min_words: 60, description_max_words: 120, condition_max_words: 45, seo_title_max: 60, seo_description_min: 120, seo_description_max: 155 },
    visible_keys: null,
    safeguard: { field_vs_photos: 'ask', field_vs_field: 'note', field_vs_listings: 'note', missing_in_text: 'ask', text_vs_facts: 'ask' },
    max_questions: 3, shop_name: '', condition_report_key: 'custom.condition_report_description', damage_terms: {},
    grade_word: false, stability_sentence: false, title_formula: false, dimensions_line: false,
    counts: { field_vs_photos: ZERO, field_vs_field: ZERO, field_vs_listings: ZERO, missing_in_text: ZERO, text_vs_facts: ZERO },
    ...extra,
  };
}

const price = (model: string) => ({ provider: 'openai', model, unit: 'input_token', usd_per_million: '2', fallback_call_usd: '0.01' });
const RESEARCH: ResearchSettings = {
  source_wait_seconds: 120, attribution_keys: [], fill_keys: [], extra_shop_hosts: [], detailed_condition: { key: '', levels: [] }, trusted_sites: [], max_pages: 10,
  stages: {}, max_cost_per_run: '0.50', max_cost_per_day: '15', prices: [price('gpt-6.1-sol'), price('gpt-6.1-terra'), price('gpt-6-luna')],
  language: 'English', record_days: 30, call_timeout_seconds: { model: 180, other: 60 }, providers: { model: '', visual_search: '', page_reader: '' },
};

function definition(key: string, label: string, group: 'details' | 'condition', core_type: string): Definition {
  return { integration: 'shopify', key, label, core_type, choices: [], validations: '{}', group, description: null, writable: true };
}

beforeEach(() => {
  vi.mocked(listDefinitions).mockResolvedValue([]);
  vi.mocked(writerSettingsSave).mockImplementation(async value => value as WriterSettings);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

async function open(settings: WriterSettings = defaults()) {
  vi.mocked(writerSettingsGet).mockResolvedValue(settings);
  render(<WriterSection research={RESEARCH} active />);
  await screen.findByRole('button', { name: 'Save writer settings' });
}

async function saved(): Promise<WriterSettings> {
  fireEvent.click(screen.getByRole('button', { name: 'Save writer settings' }));
  await waitFor(() => expect(writerSettingsSave).toHaveBeenCalledTimes(1));
  return vi.mocked(writerSettingsSave).mock.calls[0][0];
}

const editor = () => screen.getByRole('textbox', { name: 'Instruction (English)' });

it('w4_c6a_writer_entry_and_the_defaults_with_the_priced_models', async () => {
  render(<SettingsNav active="writer" showResearch onSelect={() => undefined} />);
  const entries = within(screen.getByRole('navigation', { name: 'Settings sections' })).getAllByRole('button').map(b => b.textContent);
  expect(entries.slice(entries.indexOf('Research'), entries.indexOf('Research') + 2)).toEqual(['Research', 'Writer']);
  cleanup();
  await open();
  const model = screen.getByRole('combobox', { name: 'Model' });
  expect(model).toHaveTextContent('gpt-6.1-sol');
  expect(await optionsOf(model)).toEqual(['gpt-6.1-sol', 'gpt-6.1-terra', 'gpt-6-luna']);
  expect(screen.getByRole('textbox', { name: 'Effort' })).toHaveValue('medium');
  expect(screen.getByRole('combobox', { name: 'Writing language' })).toHaveTextContent('English');
  expect(screen.getByRole('checkbox', { name: 'Write texts automatically on Next to review' })).toBeChecked();
});

it('w4_c6b_swedish_and_automatic_writing_off_save_the_whole_slot', async () => {
  await open();
  await chooseValue(screen.getByRole('combobox', { name: 'Writing language' }), 'Swedish');
  fireEvent.click(screen.getByRole('checkbox', { name: 'Write texts automatically on Next to review' }));
  expect(await saved()).toEqual(defaults({ language: 'Swedish', auto: false }));
});

it('w4_c7a_an_edited_instruction_is_saved', async () => {
  await open();
  expect(editor()).toHaveValue(SHIPPED_EN);
  fireEvent.change(editor(), { target: { value: 'Write shorter descriptions.' } });
  const value = await saved();
  expect(value.instructions.English.text).toBe('Write shorter descriptions.');
  expect(value.instructions.Swedish).toEqual(defaults().instructions.Swedish);
});

it('w4_c7b_preview_shows_the_instructions_input_and_schema', async () => {
  vi.mocked(writerPreview).mockResolvedValue({
    instructions: 'Guide 2\n\nExample A', input: '{"texts":["title"],"shop_name":"Beyo Vintage"}', schema: '{"type":"object","properties":{"findings":{}}}',
  });
  await open();
  fireEvent.click(screen.getByRole('button', { name: 'Preview what is sent' }));
  const dialog = await screen.findByRole('dialog');
  expect(writerPreview).toHaveBeenCalledTimes(1);
  expect(writerPreview).toHaveBeenCalledWith('English');
  expect(dialog.querySelector('[data-preview="instructions"]')).toHaveTextContent('Guide 2 Example A');
  expect(dialog.querySelector('[data-preview="input"]')).toHaveTextContent('"shop_name": "Beyo Vintage"');
  expect(dialog.querySelector('[data-preview="schema"]')).toHaveTextContent('"findings": {}');
});

it('w4_c7c_use_this_puts_an_earlier_version_in_the_editor_without_saving', async () => {
  const base = defaults();
  await open(defaults({ instructions: { ...base.instructions,
    English: { text: 'Guide 2', effective_instruction: 'Guide 2', is_default: false, versions: [{ saved_at: 1_763_570_000_000, text: 'Guide 1' }] } } }));
  expect(editor()).toHaveValue('Guide 2');
  const versions = within(screen.getByRole('list', { name: 'Earlier versions' })).getAllByRole('listitem');
  expect(versions[0]).toHaveTextContent(new Date(1_763_570_000_000).toLocaleString());
  fireEvent.click(within(versions[0]).getByRole('button', { name: /Use the version/ }));
  expect(editor()).toHaveValue('Guide 1');
  expect(writerSettingsSave).not.toHaveBeenCalled();
});

it('w4_c7d_reset_to_default_shows_the_returned_default_text', async () => {
  const base = defaults();
  await open(defaults({ instructions: { ...base.instructions,
    English: { text: 'Guide 2', effective_instruction: 'Guide 2', is_default: false, versions: [] } } }));
  vi.mocked(writerInstructionReset).mockResolvedValue(defaults({ instructions: { ...base.instructions,
    English: { text: null, effective_instruction: SHIPPED_EN, is_default: true, versions: [{ saved_at: 1, text: 'Guide 2' }] } } }));
  fireEvent.click(screen.getByRole('button', { name: 'Reset to default' }));
  await waitFor(() => expect(editor()).toHaveValue(SHIPPED_EN));
  expect(writerInstructionReset).toHaveBeenCalledTimes(1);
  expect(writerInstructionReset).toHaveBeenCalledWith('English');
});

const FIELDS = [
  definition('custom.designer', 'Designer', 'details', 'text'),
  definition('custom.price', 'Price', 'details', 'money'),
  definition('custom.condition_scratch', 'Scratch', 'condition', 'choice'),
];

it('w4_c8a_without_a_saved_list_the_default_fields_are_checked_and_prices_not', async () => {
  vi.mocked(listDefinitions).mockResolvedValue(FIELDS);
  await open();
  await screen.findByRole('checkbox', { name: 'Designer (custom.designer)' });
  expect(screen.getByRole('checkbox', { name: 'Designer (custom.designer)' })).toBeChecked();
  expect(screen.getByRole('checkbox', { name: 'Scratch (custom.condition_scratch)' })).toBeChecked();
  expect(screen.getByRole('checkbox', { name: 'Price (custom.price)' })).not.toBeChecked();
});

it('w4_c8b_unchecking_a_field_saves_the_list_without_it', async () => {
  vi.mocked(listDefinitions).mockResolvedValue(FIELDS);
  await open();
  fireEvent.click(await screen.findByRole('checkbox', { name: 'Scratch (custom.condition_scratch)' }));
  expect((await saved()).visible_keys).toEqual(['custom.designer']);
});

it('w4_c9a_an_added_swedish_example_is_saved_after_the_existing_one', async () => {
  await open();
  fireEvent.click(screen.getByRole('button', { name: 'Add Swedish example' }));
  fireEvent.change(screen.getByRole('textbox', { name: 'Swedish example 2 title' }), { target: { value: 'Teakbord, 1960-tal' } });
  expect((await saved()).examples.Swedish).toEqual([EXAMPLE_D,
    { name: '', fields: '', title: 'Teakbord, 1960-tal', description: '', condition_report: '', seo_title: '', seo_description: '' }]);
});

it('w4_c10a_each_safeguard_type_shows_its_mode_and_counts', async () => {
  await open(defaults({ counts: { ...defaults().counts, field_vs_photos: { answered: 3, dismissed: 5 } } }));
  const row = document.querySelector('[data-safeguard="field_vs_photos"]') as HTMLElement;
  expect(within(row).getByRole('combobox', { name: 'Field vs photos mode' })).toHaveTextContent('ask');
  expect(within(row).getByText('answered 3 · dismissed 5')).toBeInTheDocument();
});

it('w4_c10b_mode_off_questions_and_shop_name_are_saved', async () => {
  await open();
  await chooseValue(screen.getByRole('combobox', { name: 'Field vs field mode' }), 'off');
  fireEvent.change(screen.getByRole('spinbutton', { name: 'Questions per write' }), { target: { value: '2' } });
  fireEvent.change(screen.getByRole('textbox', { name: 'Shop name for search titles' }), { target: { value: 'Beyo Vintage' } });
  const value = await saved();
  expect(value.safeguard.field_vs_field).toBe('off');
  expect(value.max_questions).toBe(2);
  expect(value.shop_name).toBe('Beyo Vintage');
});

it('a23_the_four_switches_start_off_and_are_saved', async () => {
  await open();
  const labels = ['Start the condition report with the grade word', 'Always end the condition report with a stability sentence',
    'Send the built-in title formula', 'End the description with a dimensions line'];
  for (const label of labels) expect(screen.getByRole('checkbox', { name: label })).not.toBeChecked();
  fireEvent.click(screen.getByRole('checkbox', { name: 'Send the built-in title formula' }));
  fireEvent.click(screen.getByRole('checkbox', { name: 'Start the condition report with the grade word' }));
  const value = await saved();
  expect([value.grade_word, value.stability_sentence, value.title_formula, value.dimensions_line]).toEqual([true, false, true, false]);
});

it('a23_export_and_import_buttons', async () => {
  vi.mocked(writerSettingsExport).mockResolvedValue(true);
  vi.mocked(writerSettingsImport).mockResolvedValue(defaults({ shop_name: 'Imported shop', banned_extra: ['sanded back'] }));
  await open();
  fireEvent.click(screen.getByRole('button', { name: 'Export…' }));
  await waitFor(() => expect(writerSettingsExport).toHaveBeenCalledTimes(1));
  await waitFor(() => expect(toast.success).toHaveBeenCalledWith('Writer settings exported'));
  fireEvent.click(screen.getByRole('button', { name: 'Import…' }));
  await waitFor(() => expect(screen.getByRole('textbox', { name: 'Shop name for search titles' })).toHaveValue('Imported shop'));
  expect(screen.getByRole('textbox', { name: /Words and phrases never to use/ })).toHaveValue('sanded back');
  expect(toast.success).toHaveBeenCalledWith('Writer settings imported and saved');
  expect(writerSettingsSave).not.toHaveBeenCalled();
});

it('a23_a_refused_import_shows_the_reason', async () => {
  vi.mocked(writerSettingsImport).mockRejectedValue(new Error('this is not a Session Manager writer settings file'));
  await open();
  fireEvent.click(screen.getByRole('button', { name: 'Import…' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('this is not a Session Manager writer settings file');
});
