import '@testing-library/jest-dom/vitest';
import { createElement, type ComponentProps } from 'react';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TemplatesSection } from '../TemplatesSection';
import { presetRect } from '../PresetPreview';
import {
  addContainer, createTemplate, deleteTemplate, getTemplate, listTemplates, pickExampleFiles, removeContainer,
  reorderContainers, setLabel, setPresets, setShootingOrder, type Template,
} from '@/services/templates';
import { chooseValue } from '@/test/select';

/** The editor's drag end, captured from the real DndContext so a drop can be played without pointer geometry. */
const drag = vi.hoisted(() => ({ end: null as null | ((event: unknown) => void) }));
vi.mock('@dnd-kit/core', async importOriginal => {
  const real = await importOriginal<typeof import('@dnd-kit/core')>();
  return {
    ...real,
    DndContext: (props: ComponentProps<typeof real.DndContext>) => {
      drag.end = props.onDragEnd as (event: unknown) => void;
      return createElement(real.DndContext, props);
    },
  };
});
vi.mock('@/services/templates', () => ({
  addContainer: vi.fn(), createTemplate: vi.fn(), deleteTemplate: vi.fn(), getTemplate: vi.fn(), listTemplates: vi.fn(),
  pickExampleFiles: vi.fn(), removeContainer: vi.fn(), reorderContainers: vi.fn(), setExample: vi.fn(), setPresets: vi.fn(),
  setShootingOrder: vi.fn(), setLabel: vi.fn(),
}));
vi.mock('@/services/items', () => ({ listItemTypes: vi.fn().mockResolvedValue([]) }));
vi.mock('@/services/sessions', () => ({ previewUrl: (path: string) => `asset:${path}` }));

const c = (id: number, final_no = id, shoot_no = id) => ({
  id, final_no, shoot_no, example_file: `${id}.jpg`, example_path: `/tmp/${id}.jpg`,
  presets: { split: false, shape: null as string | null, zoom: 0 }, name: null as string | null, description: null as string | null,
});
const t = (containers = [c(1), c(2), c(3)]): Template => ({ id: 8, item_type: 'Dining chair', containers });

beforeEach(() => {
  vi.mocked(listTemplates).mockResolvedValue([]);
  vi.mocked(getTemplate).mockResolvedValue(t());
});
afterEach(() => { cleanup(); vi.clearAllMocks(); vi.restoreAllMocks(); vi.useRealTimers(); });
async function open() {
  render(<TemplatesSection />);
  fireEvent.change(screen.getByRole('combobox', { name: 'Item type' }), { target: { value: 'Dining chair' } });
}
async function show(template = t()) {
  vi.mocked(getTemplate).mockResolvedValue(template);
  await open();
  await screen.findByTestId(`template-card-${template.containers[0].id}`);
}
const cardIds = () => screen.getAllByTestId(/template-card-/).map(card => Number(card.getAttribute('data-testid')!.split('-').pop()));
const card = (id: number) => screen.getByTestId(`template-card-${id}`);

it('t1_c6a_creates_a_template_for_the_typed_type', async () => {
  vi.mocked(getTemplate).mockResolvedValue(null);
  vi.mocked(createTemplate).mockResolvedValue(t([]));
  await open();
  fireEvent.click(await screen.findByRole('button', { name: 'Create template' }));
  await waitFor(() => expect(createTemplate).toHaveBeenCalledExactlyOnceWith('Dining chair'));
  expect(screen.queryAllByTestId(/template-card-/)).toHaveLength(0);
  expect(screen.getByRole('button', { name: /Add container/ })).toBeInTheDocument();
});

it('t1_c6b_shows_final_order_bw_examples_and_shooting_numbers', async () => {
  await show(t([c(1,1,3), c(2,2,1), c(3,3,2)]));
  expect(cardIds()).toEqual([1, 2, 3]);
  expect([1, 2, 3].map(id => within(card(id)).getByRole('textbox', { name: `Name of container ${id}` }))
    .map(name => name.getAttribute('placeholder'))).toEqual(['Container 1', 'Container 2', 'Container 3']);
  expect([1, 2, 3].map(id => within(card(id)).getByLabelText(/Shooting number/).textContent)).toEqual(['Shot 3', 'Shot 1', 'Shot 2']);
  for (const id of [1, 2, 3]) {
    expect(card(id).querySelector('[data-bw]')).toBeInTheDocument();
    expect(card(id).querySelector('img')).toHaveAttribute('src', 'asset:/tmp/' + id + '.jpg');
  }
  expect(screen.queryByRole('button', { name: /Move/ })).not.toBeInTheDocument();
});

it('t1_c6c_the_dashed_add_tile_adds_chosen_photos_in_order_or_an_empty_container', async () => {
  vi.mocked(getTemplate).mockResolvedValue(t([]));
  vi.mocked(pickExampleFiles).mockResolvedValue(['/tmp/a.jpg', '/tmp/b.png']);
  vi.mocked(addContainer).mockResolvedValueOnce(t([c(1)])).mockResolvedValueOnce(t([c(1),c(2)])).mockResolvedValueOnce(t([c(1),c(2),c(3)]));
  await open();
  fireEvent.click(await screen.findByRole('button', { name: /Add container/ }));
  await waitFor(() => expect(addContainer).toHaveBeenCalledTimes(2));
  expect(vi.mocked(addContainer).mock.calls).toEqual([[8, '/tmp/a.jpg'], [8, '/tmp/b.png']]);
  expect(await screen.findByTestId('template-card-2')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Add without a photo' }));
  await waitFor(() => expect(addContainer).toHaveBeenNthCalledWith(3, 8, null));
  expect(await screen.findByTestId('template-card-3')).toBeInTheDocument();
});

it('t1_c6d_saves_split_shape_and_zoom_without_a_save_button_and_previews_them', async () => {
  const split = t([c(1),c(2),{ ...c(3), presets: { split: true, shape: null, zoom: 0 } }]);
  const shaped = t([c(1),c(2),{ ...c(3), presets: { split: true, shape: '4:5', zoom: 0 } }]);
  const zoom = t([c(1),c(2),{ ...c(3), presets: { split: true, shape: '4:5', zoom: 20 } }]);
  vi.mocked(setPresets).mockResolvedValueOnce(split).mockResolvedValueOnce(shaped).mockResolvedValueOnce(zoom);
  await show();
  expect(card(3).querySelector('[data-side]')).toBeNull();
  fireEvent.click(screen.getByRole('checkbox', { name: 'Split container 3' }));
  await waitFor(() => expect(setPresets).toHaveBeenCalledWith(3, { split: true, shape: null, zoom: 0 }));
  await waitFor(() => expect(screen.getByRole('checkbox', { name: 'Split container 3' })).toBeChecked());
  // The split result is the two square halves, Left and Right, as the session shows it.
  expect(Array.from(card(3).querySelectorAll('[data-side]')).map(side => side.getAttribute('data-side'))).toEqual(['left', 'right']);
  expect(card(3).querySelector('.preset-crop')).toHaveAttribute('data-split');
  await chooseValue(within(card(3)).getByRole('combobox', { name: 'Shape container 3' }), '4:5');
  await waitFor(() => expect(setPresets).toHaveBeenNthCalledWith(2, 3, { split: true, shape: '4:5', zoom: 0 }));
  fireEvent.change(screen.getByRole('slider', { name: 'Zoom container 3' }), { target: { value: '20' } });
  expect(screen.getByLabelText('Zoom of container 3')).toHaveTextContent('20 % · 1.25×');
  await waitFor(() => expect(setPresets).toHaveBeenNthCalledWith(3, 3, { split: true, shape: '4:5', zoom: 20 }));
  expect(setPresets).toHaveBeenCalledTimes(3);
  expect(screen.queryByRole('button', { name: /Save/ })).not.toBeInTheDocument();
});

it('t1_c6d2_the_preview_crop_is_the_frame_the_presets_give_the_example', () => {
  const frame = { w: 3000, h: 2000 };
  expect(presetRect({ split: false, shape: null, zoom: 0 }, frame)).toEqual({ x: 0, y: 0, w: 1, h: 1 });
  const square = presetRect({ split: false, shape: '1:1', zoom: 20 }, frame);
  expect(square.w * frame.w).toBeCloseTo(1600);
  expect(square.h * frame.h).toBeCloseTo(1600);
  expect(square.x + square.w / 2).toBeCloseTo(0.5);
  const split = presetRect({ split: true, shape: null, zoom: 0 }, frame);
  expect((split.w * frame.w) / (split.h * frame.h)).toBeCloseTo(2);
});

it('t1_c6e_shooting_order_shows_cards_by_shot_and_a_drop_saves_the_new_shooting_order', async () => {
  vi.mocked(setShootingOrder).mockResolvedValue(t([c(1,1,2), c(2,2,3), c(3,3,1)]));
  await show(t([c(1,1,2), c(2,2,1), c(3,3,3)]));
  fireEvent.click(screen.getByRole('button', { name: 'Shooting order' }));
  expect(screen.getByRole('button', { name: 'Shooting order' })).toHaveAttribute('aria-pressed', 'true');
  expect(cardIds()).toEqual([2, 1, 3]);
  expect(within(card(2)).getByLabelText('Shot 1')).toBeInTheDocument();
  expect(within(card(2)).getByLabelText('Final image 2')).toBeInTheDocument();
  // Container 3 dragged onto the first place of the route.
  act(() => drag.end!({ active: { id: 3 }, over: { id: 2 } }));
  await waitFor(() => expect(setShootingOrder).toHaveBeenCalledExactlyOnceWith(8, [3, 2, 1]));
  expect(reorderContainers).not.toHaveBeenCalled();
  await waitFor(() => expect(cardIds()).toEqual([3, 1, 2]));
});

it('t1_c6f_a_drop_in_final_order_reorders_the_final_containers', async () => {
  vi.mocked(reorderContainers).mockResolvedValue(t([c(1,1),c(3,2),c(2,3)]));
  await show();
  act(() => drag.end!({ active: { id: 3 }, over: { id: 2 } }));
  await waitFor(() => expect(reorderContainers).toHaveBeenCalledExactlyOnceWith(8, [1, 3, 2]));
  expect(setShootingOrder).not.toHaveBeenCalled();
  act(() => drag.end!({ active: { id: 1 }, over: null }));
  expect(reorderContainers).toHaveBeenCalledTimes(1);
});

it('t1_c6g_failed_write_shows_error_and_reloads_stored_presets', async () => {
  vi.mocked(setPresets).mockRejectedValue(new Error('disk full'));
  const before = t([c(1), c(2), { ...c(3), presets: { split: true, shape: null, zoom: 0 } }]);
  const stored = t();
  vi.mocked(getTemplate).mockResolvedValueOnce(before).mockResolvedValueOnce(stored);
  await open();
  await waitFor(() => expect(screen.getByRole('checkbox', { name: 'Split container 3' })).toBeChecked());
  fireEvent.click(screen.getByRole('checkbox', { name: 'Split container 3' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('disk full');
  await waitFor(() => expect(getTemplate).toHaveBeenCalledTimes(2));
  expect(screen.getByRole('checkbox', { name: 'Split container 3' })).not.toBeChecked();
});

it('t1_c6h_confirms_deletion_and_returns_to_create', async () => {
  vi.mocked(deleteTemplate).mockResolvedValue();
  await show();
  fireEvent.click(screen.getByRole('button', { name: 'Delete template' }));
  expect(screen.getByRole('dialog')).toHaveTextContent('Delete the template for Dining chair?');
  fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
  expect(deleteTemplate).not.toHaveBeenCalled();
  expect(screen.getByRole('button', { name: /Add container/ })).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Delete template' }));
  fireEvent.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Delete template' }));
  await waitFor(() => expect(deleteTemplate).toHaveBeenCalledExactlyOnceWith(8));
  expect(await screen.findByRole('button', { name: 'Create template' })).toBeInTheDocument();
});

it('t1_c6i_names_and_describes_a_container', async () => {
  const named = t([c(1), { ...c(2), name: 'Front, low' }, c(3)]);
  const described = t([c(1), { ...c(2), name: 'Front, low', description: 'Kneel at seat height.' }, c(3)]);
  vi.mocked(setLabel).mockResolvedValueOnce(named).mockResolvedValueOnce(described);
  await show();
  const name = within(card(2)).getByRole('textbox', { name: 'Name of container 2' });
  fireEvent.change(name, { target: { value: 'Front, low' } });
  fireEvent.blur(name);
  await waitFor(() => expect(setLabel).toHaveBeenCalledExactlyOnceWith(2, 'Front, low', ''));
  await waitFor(() => expect(within(card(2)).getByRole('button', { name: 'Description, Front, low' })).toBeInTheDocument());
  fireEvent.click(within(card(2)).getByRole('button', { name: 'Description, Front, low' }));
  fireEvent.change(await screen.findByRole('textbox', { name: 'Description of Front, low' }), { target: { value: 'Kneel at seat height.' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save description' }));
  await waitFor(() => expect(setLabel).toHaveBeenNthCalledWith(2, 2, 'Front, low', 'Kneel at seat height.'));
  // An unchanged name is not written again.
  fireEvent.blur(within(card(2)).getByRole('textbox', { name: 'Name of container 2' }));
  expect(setLabel).toHaveBeenCalledTimes(2);
});

it('t1_c6j_the_corner_x_removes_a_container_after_a_confirmation', async () => {
  vi.mocked(removeContainer).mockResolvedValue(t([c(1), c(3, 2, 2)]));
  await show();
  fireEvent.click(within(card(2)).getByRole('button', { name: 'Remove Container 2' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Keep' }));
  expect(removeContainer).not.toHaveBeenCalled();
  fireEvent.click(within(card(2)).getByRole('button', { name: 'Remove Container 2' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Remove' }));
  await waitFor(() => expect(removeContainer).toHaveBeenCalledExactlyOnceWith(2));
  await waitFor(() => expect(cardIds()).toEqual([1, 3]));
});
