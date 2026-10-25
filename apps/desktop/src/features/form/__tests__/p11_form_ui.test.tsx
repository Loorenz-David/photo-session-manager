import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AttributeValue, Definition, Form, FormField, Group, PresetEntry } from '@/services/attributes';
import type { Item } from '@/services/items';
import type { Photo } from '@/services/photos';

vi.mock('@/services/attributes', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/attributes')>()),
  getForm: vi.fn(),
  listDefinitions: vi.fn(),
  setAttributeValue: vi.fn(),
  clearAttribute: vi.fn(),
  acceptAllSuggestions: vi.fn(),
  getPreset: vi.fn(),
  setPreset: vi.fn(),
  updateIdentity: vi.fn(),
}));

import { AiPill } from '@/features/form/AiPill';
import { FormDialog } from '@/features/form/FormDialog';
import { kindOf } from '@/features/form/kinds';
import { clearAttribute, getForm, getPreset, listDefinitions, setAttributeValue, setPreset } from '@/services/attributes';

// jsdom lacks these browser APIs, which cmdk, Radix and dnd-kit use.
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;
if (!('PointerEvent' in window)) {
  class PointerEventPolyfill extends MouseEvent {
    readonly pointerId: number;
    readonly isPrimary: boolean;
    readonly pointerType: string;
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init);
      this.pointerId = init.pointerId ?? 1;
      this.isPrimary = init.isPrimary ?? true;
      this.pointerType = init.pointerType ?? 'mouse';
    }
  }
  Object.assign(window, { PointerEvent: PointerEventPolyfill });
}

/** The rendered container of one generated field. */
async function fieldElement(key: string): Promise<HTMLElement> {
  return waitFor(() => {
    const element = document.querySelector(`[data-field="${key}"]`);
    expect(element).not.toBeNull();
    return element as HTMLElement;
  });
}

const ITEM_ID = 7;
const ITEM: Item = {
  id: ITEM_ID,
  sku: 'VF-0042',
  folder_key: 'VF-0042',
  barcode: null,
  item_type: 'Chair',
  title: 'Oak chair',
  origin: 'local',
  created_at: 0,
  updated_at: 0,
};

function def(key: string, core_type: string, choices: string[] = [], group: Group = 'details', label = key): Definition {
  return { integration: 'shopify', key, label, core_type, choices, validations: '{}', group, description: null, writable: core_type !== 'reference' };
}

function stored(key: string, value: unknown, extra: Partial<AttributeValue> = {}): AttributeValue {
  return {
    item_id: ITEM_ID,
    key,
    value: JSON.stringify(value),
    valid: true,
    provenance: 'human',
    ai_confidence: null,
    ai_status: null,
    ai_reason: null,
    ai_sources: null,
    updated_at: 1,
    ...extra,
  };
}

function field(definition: Definition, value: AttributeValue | null = null, selected: number[] | null = null): FormField {
  return { definition, value, is_default: false, is_severity: false, selected };
}

/** A form whose Details step holds `details` (counts as given, else derived). */
function form(groups: Partial<Record<Group, FormField[]>>, counts: Partial<Record<Group, [number, number]>> = {}): Form {
  return {
    item_id: ITEM_ID,
    groups: (['details', 'condition', 'product'] as Group[]).map(group => {
      const fields = groups[group] ?? [];
      const [filled, total] = counts[group] ?? [fields.filter(f => f.value?.valid).length, fields.length];
      return { group, fields, filled, total };
    }),
  };
}

function photo(id: number, classification: Photo['classification'], position: number): Photo {
  return {
    id,
    session_id: 3,
    classification,
    position,
    captured_at: id,
    original_preview_path: null,
    adjusted_preview_path: null,
    orientation: null,
    source: 'capture',
    integrity: 'ok',
  };
}

function renderForm(photos: Photo[] = []) {
  return render(<FormDialog open onOpenChange={() => undefined} item={ITEM} photos={photos} onItemChanged={() => undefined} />);
}

/** Lays out the Customize rows for dnd-kit, which measures with getBoundingClientRect (zero everywhere in jsdom). */
function layoutRows(keys: string[]) {
  vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function (this: Element) {
    const key = this.getAttribute('data-layout-key');
    const index = key === null ? -1 : keys.indexOf(key);
    const top = index === -1 ? 0 : index * 60;
    const height = index === -1 ? 0 : 50;
    const width = index === -1 ? 0 : 400;
    return { left: 0, top, width, height, right: width, bottom: top + height, x: 0, y: top, toJSON: () => ({}) } as DOMRect;
  });
}

beforeEach(() => {
  vi.mocked(setAttributeValue).mockImplementation(async (_item, key, value) => stored(key, value));
  vi.mocked(clearAttribute).mockResolvedValue(undefined);
  vi.mocked(setPreset).mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.clearAllMocks();
});

describe('P11 C1 kindOf (evidence design_handoff §Field kind map)', () => {
  const severity = ['Moderate', 'N/A', 'Major', 'None', 'Minor'];
  const rows: [string, Pick<Definition, 'core_type' | 'choices'>, string][] = [
    ['C1(a) text, no choices', { core_type: 'text', choices: [] }, 'text'],
    ['C1(b) choice with the severity set (any order)', { core_type: 'choice', choices: severity }, 'severity'],
    ['C1(c) choice, one option over 32 characters', { core_type: 'choice', choices: ['Good', 'Very Good - no defects, slight traces of use'] }, 'radio'],
    ['C1(d) choice, 5 short options', { core_type: 'choice', choices: ['1', '2', '3', '4', '5'] }, 'select'],
    ['C1(e) choice, 4 options joined over 48 characters', { core_type: 'choice', choices: ['Teak veneer', 'Rosewood veneer', 'Oak solid wood', 'Walnut veneer'] }, 'select'],
    ['C1(f) choice, 4 short options', { core_type: 'choice', choices: ['Round', 'Oval', 'Square', 'Other'] }, 'choice'],
    ['C1(g) tags', { core_type: 'tags', choices: [] }, 'tags'],
    ['C1(h) multi_choice, 5 options', { core_type: 'multi_choice', choices: ['a', 'b', 'c', 'd', 'e'] }, 'multiselect'],
    ['C1(i) multi_choice, 3 short options', { core_type: 'multi_choice', choices: ['Seat', 'Back', 'Arms'] }, 'multi'],
    ['C1(j) textarea', { core_type: 'textarea', choices: [] }, 'textarea'],
    ['C1(j) rich_text', { core_type: 'rich_text', choices: [] }, 'textarea'],
    ['C1(k) integer', { core_type: 'integer', choices: [] }, 'int'],
    ['C1(l) decimal', { core_type: 'decimal', choices: [] }, 'decimal'],
    ['C1(m) dimension', { core_type: 'dimension', choices: [] }, 'dimension'],
    ['C1(m) weight', { core_type: 'weight', choices: [] }, 'weight'],
    ['C1(n) boolean', { core_type: 'boolean', choices: [] }, 'boolean'],
    ['C1(o) date', { core_type: 'date', choices: [] }, 'date'],
    ['C1(p) color', { core_type: 'color', choices: [] }, 'color'],
    ['C1(q) rating', { core_type: 'rating', choices: [] }, 'rating'],
    ['C1(r) money', { core_type: 'money', choices: [] }, 'money'],
    ['C1(s) url', { core_type: 'url', choices: [] }, 'url'],
    ['C1(t) media_list', { core_type: 'media_list', choices: [] }, 'files'],
    ['C1(u) reference', { core_type: 'reference', choices: [] }, 'unsupported'],
  ];
  it.each(rows)('%s', (_row, definition, kind) => {
    expect(kindOf(definition)).toBe(kind);
  });
});

describe('P11 product details form', () => {
  it('C2(a) typing in a text field calls attribute_values_set once, 300 ms after the last keystroke, with the typed string', async () => {
    vi.mocked(getForm).mockResolvedValue(form({ details: [field(def('custom.maker', 'text', [], 'details', 'Maker'))] }));
    renderForm();
    const input = await screen.findByLabelText('Maker');
    vi.useFakeTimers();
    fireEvent.change(input, { target: { value: 'H' } });
    fireEvent.change(input, { target: { value: 'Hans' } });
    fireEvent.change(input, { target: { value: 'Hans Wegner' } });
    act(() => void vi.advanceTimersByTime(299));
    expect(setAttributeValue).not.toHaveBeenCalled();
    act(() => void vi.advanceTimersByTime(1));
    expect(setAttributeValue).toHaveBeenCalledTimes(1);
    expect(setAttributeValue).toHaveBeenCalledWith(ITEM_ID, 'custom.maker', 'Hans Wegner');
  });

  it('C2(b) an invalid stored value is shown verbatim with an "invalid" marker and is not written until the user edits', async () => {
    const finish = def('custom.finish', 'choice', ['Matte', 'Satin', 'Gloss'], 'details', 'Finish');
    vi.mocked(getForm).mockResolvedValue(form({ details: [field(finish, stored('custom.finish', 'Glossy!', { valid: false }))] }));
    renderForm();
    const marker = (await fieldElement('custom.finish')).querySelector('[data-invalid]') as HTMLElement;
    expect(marker).toHaveTextContent('Invalid');
    expect(within(marker).getByText('Glossy!')).toBeInTheDocument();
    await act(() => new Promise(resolve => setTimeout(resolve, 400)));
    expect(setAttributeValue).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('radio', { name: 'Satin' }));
    await waitFor(() => expect(setAttributeValue).toHaveBeenCalledWith(ITEM_ID, 'custom.finish', 'Satin'));
  });

  it('C3(a) the step tabs read "Details 3/8", "Condition 0/13", "Product 1/3"', async () => {
    vi.mocked(getForm).mockResolvedValue(form({}, { details: [3, 8], condition: [0, 13], product: [1, 3] }));
    renderForm();
    expect(await screen.findByRole('tab', { name: 'Details 3/8' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Condition 0/13' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Product 1/3' })).toBeInTheDocument();
  });

  it('C4(a) Customize: drag k2 above k1, remove k3, add k9 with default "Oak" → presets_set(type, step, [k2, k1, k9 = "Oak"])', async () => {
    const definitions = [def('k1', 'text', [], 'details', 'K one'), def('k2', 'text', [], 'details', 'K two'), def('k3', 'text', [], 'details', 'K three'), def('k9', 'text', [], 'details', 'Wood')];
    vi.mocked(getForm).mockResolvedValue(form({ details: definitions.slice(0, 3).map(d => field(d)) }));
    vi.mocked(listDefinitions).mockResolvedValue(definitions);
    vi.mocked(getPreset).mockImplementation(async (_type, group) => (group === 'details' ? [{ key: 'k1' }, { key: 'k2' }, { key: 'k3' }] : []));
    renderForm();
    fireEvent.click(await screen.findByRole('button', { name: 'Customize fields' }));
    const handle = await screen.findByRole('button', { name: 'Reorder K two' });
    await waitFor(() => expect(screen.getByRole('option', { name: /Wood/ })).toBeInTheDocument());

    layoutRows(['k1', 'k2', 'k3']);
    fireEvent.pointerDown(handle, { button: 0, isPrimary: true, clientX: 200, clientY: 85 });
    await act(async () => void fireEvent.pointerMove(document, { clientX: 200, clientY: 75 }));
    await act(async () => void fireEvent.pointerMove(document, { clientX: 200, clientY: 25 }));
    await act(async () => void fireEvent.pointerUp(document, { clientX: 200, clientY: 25 }));
    vi.mocked(Element.prototype.getBoundingClientRect).mockRestore();
    await waitFor(() => expect([...document.querySelectorAll('[data-layout-key]')].map(row => row.getAttribute('data-layout-key'))).toEqual(['k2', 'k1', 'k3']));
    await act(() => new Promise(resolve => setTimeout(resolve, 80)));

    fireEvent.click(screen.getByRole('button', { name: 'Remove K three' }));
    fireEvent.click(screen.getByRole('option', { name: /Wood/ }));
    const wood = await screen.findByRole('group', { name: 'Default value of Wood' });
    fireEvent.change(within(wood).getByRole('textbox'), { target: { value: 'Oak' } });
    fireEvent.click(screen.getByRole('button', { name: 'Done' }));

    await waitFor(() => expect(setPreset).toHaveBeenCalledTimes(1));
    const expected: PresetEntry[] = [{ key: 'k2' }, { key: 'k1' }, { key: 'k9', default: 'Oak' }];
    expect(setPreset).toHaveBeenCalledWith('Chair', 'details', expected);
  });

  it('C4(b) Customize on Product: title, description and tags rows have no remove control', async () => {
    const product = [def('product.title', 'text', [], 'product', 'Title'), def('product.description', 'rich_text', [], 'product', 'Description'), def('product.tags', 'tags', [], 'product', 'Tags'), def('custom.era', 'text', [], 'product', 'Era')];
    vi.mocked(getForm).mockResolvedValue(form({ product: product.map(d => field(d)) }));
    vi.mocked(listDefinitions).mockResolvedValue(product);
    vi.mocked(getPreset).mockResolvedValue(product.map(d => ({ key: d.key })));
    renderForm();
    fireEvent.click(await screen.findByRole('tab', { name: /^Product/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Customize fields' }));
    expect(await screen.findByRole('button', { name: 'Remove Era' })).toBeInTheDocument();
    for (const label of ['Title', 'Description', 'Tags']) {
      expect(screen.getByRole('button', { name: `Reorder ${label}` })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: `Remove ${label}` })).toBeNull();
    }
  });

  it('C5(a) AiPill shows 3 / 2 / 1 bars for high / medium / low; low reads "Check"', () => {
    const expected: [string, number, string][] = [
      ['high', 3, 'AI'],
      ['medium', 2, 'AI'],
      ['low', 1, 'Check'],
    ];
    for (const [confidence, bars, text] of expected) {
      const { container, unmount } = render(<AiPill confidence={confidence} />);
      expect(container.querySelectorAll('[data-bar][data-filled="true"]')).toHaveLength(bars);
      expect(container.querySelectorAll('[data-bar]')).toHaveLength(3);
      expect(container.querySelector('[data-ai-pill]')).toHaveTextContent(text);
      unmount();
    }
  });

  it('C5(b) popover: "Looks right" writes the same value as a human edit; "Clear" calls attribute_ai_clear', async () => {
    const scratch = def('custom.condition_scratch', 'choice', ['N/A', 'None', 'Minor', 'Moderate', 'Major'], 'details', 'Scratch');
    const era = def('custom.era', 'text', [], 'details', 'Era');
    const ai = { provenance: 'ai', ai_confidence: 'high', ai_status: 'suggested', ai_reason: 'Visible on photo 2', ai_sources: ['https://example.test/a'] };
    vi.mocked(getForm).mockResolvedValue(form({ details: [field(scratch, stored(scratch.key, 'Minor', ai)), field(era, stored(era.key, '1960s', ai))] }));
    renderForm();

    const scratchField = await fieldElement('custom.condition_scratch');
    fireEvent.click(within(scratchField).getByRole('button', { name: /AI suggestion/ }));
    fireEvent.click(await screen.findByRole('button', { name: 'Looks right' }));
    await waitFor(() => expect(setAttributeValue).toHaveBeenCalledWith(ITEM_ID, 'custom.condition_scratch', 'Minor'));
    expect(clearAttribute).not.toHaveBeenCalled();

    const eraField = await fieldElement('custom.era');
    fireEvent.click(within(eraField).getByRole('button', { name: /AI suggestion/ }));
    fireEvent.click(await screen.findByRole('button', { name: 'Clear' }));
    await waitFor(() => expect(clearAttribute).toHaveBeenCalledWith(ITEM_ID, 'custom.era'));
    expect(setAttributeValue).toHaveBeenCalledTimes(1);
  });

  it('C5(c) a field with provenance human renders no AI pill', async () => {
    const maker = def('custom.maker', 'text', [], 'details', 'Maker');
    const era = def('custom.era', 'text', [], 'details', 'Era');
    vi.mocked(getForm).mockResolvedValue(
      form({ details: [field(maker, stored(maker.key, 'Hans Wegner', { provenance: 'human' })), field(era, stored(era.key, '1960s', { provenance: 'ai', ai_confidence: 'medium', ai_status: 'suggested' }))] }),
    );
    renderForm();
    expect(await screen.findByDisplayValue('Hans Wegner')).toBeInTheDocument();
    expect(document.querySelectorAll('[data-field="custom.maker"] [data-ai-pill]')).toHaveLength(0);
    expect(document.querySelectorAll('[data-field="custom.era"] [data-ai-pill]')).toHaveLength(1);
  });

  it('C6(a) damage report: unticking p2 of [p1, p2, p3] writes {"excluded": [p2]}; p1 and p3 stay selected', async () => {
    const [p1, p2, p3] = [11, 12, 13];
    const report = def('custom.damage_report', 'media_list', [], 'condition', 'Damage report');
    vi.mocked(getForm).mockResolvedValue(form({ condition: [field(report, null, [p1, p2, p3])] }));
    renderForm([photo(20, 'main', 1), photo(p1, 'condition', 1), photo(p2, 'condition', 2), photo(p3, 'condition', 3)]);
    fireEvent.click(await screen.findByRole('tab', { name: /^Condition/ }));
    const picker = await screen.findByRole('group', { name: 'Damage report' });
    const boxes = within(picker).getAllByRole('checkbox');
    expect(boxes.map(box => Number(box.getAttribute('data-photo-id')))).toEqual([p1, p2, p3]);
    fireEvent.click(boxes[1]);
    await waitFor(() => expect(setAttributeValue).toHaveBeenCalledWith(ITEM_ID, 'custom.damage_report', { excluded: [p2] }));
    expect(setAttributeValue).toHaveBeenCalledTimes(1);
    expect(boxes.map(box => box.getAttribute('aria-checked'))).toEqual(['true', 'false', 'true']);
  });
});
