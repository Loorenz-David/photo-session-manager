import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TemplatesSection } from '../TemplatesSection';
import {
  addContainer, createTemplate, deleteTemplate, getTemplate, listTemplates, pickExampleFiles,
  reorderContainers, setPresets, setShootingOrder, type Template,
} from '@/services/templates';

vi.mock('@/services/templates', () => ({
  addContainer: vi.fn(), createTemplate: vi.fn(), deleteTemplate: vi.fn(), getTemplate: vi.fn(), listTemplates: vi.fn(),
  pickExampleFiles: vi.fn(), removeContainer: vi.fn(), reorderContainers: vi.fn(), setExample: vi.fn(), setPresets: vi.fn(), setShootingOrder: vi.fn(),
}));
vi.mock('@/services/items', () => ({ listItemTypes: vi.fn().mockResolvedValue([]) }));
vi.mock('@/services/sessions', () => ({ previewUrl: (path: string) => `asset:${path}` }));

const c = (id: number, final_no = id, shoot_no = id) => ({
  id, final_no, shoot_no, example_file: `${id}.jpg`, example_path: `/tmp/${id}.jpg`,
  presets: { split: false, shape: null, zoom: 0 },
});
const t = (containers = [c(1), c(2), c(3)]): Template => ({ id: 8, item_type: 'Dining chair', containers });

beforeEach(() => {
  vi.mocked(listTemplates).mockResolvedValue([]);
  vi.mocked(getTemplate).mockResolvedValue(t());
});
afterEach(() => { cleanup(); vi.clearAllMocks(); vi.restoreAllMocks(); });
async function show(template = t()) {
  vi.mocked(getTemplate).mockResolvedValue(template);
  render(<TemplatesSection />);
  fireEvent.change(screen.getByRole('combobox', { name: 'Item type' }), { target: { value: 'Dining chair' } });
  await screen.findByTestId(`template-card-${template.containers[0].id}`);
}

it('t1_c6a_creates_a_template_for_the_typed_type', async () => {
  vi.mocked(getTemplate).mockResolvedValue(null);
  vi.mocked(createTemplate).mockResolvedValue(t([]));
  render(<TemplatesSection />);
  fireEvent.change(screen.getByRole('combobox', { name: 'Item type' }), { target: { value: 'Dining chair' } });
  fireEvent.click(await screen.findByRole('button', { name: 'Create template' }));
  await waitFor(() => expect(createTemplate).toHaveBeenCalledExactlyOnceWith('Dining chair'));
  expect(screen.queryAllByTestId(/template-card-/)).toHaveLength(0);
  expect(screen.getByRole('button', { name: 'Add photos' })).toBeInTheDocument();
});

it('t1_c6b_shows_final_order_bw_examples_and_shooting_numbers', async () => {
  await show(t([c(1,1,3), c(2,2,1), c(3,3,2)]));
  const cards = screen.getAllByTestId(/template-card-/);
  expect(cards.map(card => card.getAttribute('data-testid'))).toEqual(['template-card-1','template-card-2','template-card-3']);
  expect(cards.map(card => within(card).getByText(/Container [123]/).textContent)).toEqual(['Container 1','Container 2','Container 3']);
  expect(cards.map(card => within(card).getByLabelText(/Shooting number/).textContent)).toEqual(['Shoot 3','Shoot 1','Shoot 2']);
  for (const card of cards) {
    expect(card.querySelector('[data-bw]')).toBeInTheDocument();
    expect(card.querySelector('img')).toHaveAttribute('src', expect.stringContaining('asset:'));
  }
});

it('t1_c6c_adds_chosen_photos_in_order_and_shows_last_result', async () => {
  vi.mocked(getTemplate).mockResolvedValue(t([]));
  vi.mocked(pickExampleFiles).mockResolvedValue(['/tmp/a.jpg', '/tmp/b.png']);
  vi.mocked(addContainer).mockResolvedValueOnce(t([c(1)])).mockResolvedValueOnce(t([c(1),c(2)]));
  render(<TemplatesSection />);
  fireEvent.change(screen.getByRole('combobox', { name: 'Item type' }), { target: { value: 'Dining chair' } });
  fireEvent.click(await screen.findByRole('button', { name: 'Add photos' }));
  await waitFor(() => expect(addContainer).toHaveBeenCalledTimes(2));
  expect(vi.mocked(addContainer).mock.calls).toEqual([[8, '/tmp/a.jpg'], [8, '/tmp/b.png']]);
  expect(await screen.findByTestId('template-card-2')).toBeInTheDocument();
  expect(screen.getAllByTestId(/template-card-/)).toHaveLength(2);
});

it('t1_c6d_saves_split_then_zoom_without_a_save_button', async () => {
  const base = t();
  const split = t([c(1),c(2),{ ...c(3), presets: { split: true, shape: null, zoom: 0 } }]);
  const zoom = t([c(1),c(2),{ ...c(3), presets: { split: true, shape: null, zoom: 20 } }]);
  vi.mocked(setPresets).mockResolvedValueOnce(split).mockResolvedValueOnce(zoom);
  await show(base);
  fireEvent.click(screen.getByRole('checkbox', { name: 'Split container 3' }));
  await waitFor(() => expect(setPresets).toHaveBeenCalledWith(3, { split: true, shape: null, zoom: 0 }));
  await waitFor(() => expect(screen.getByRole('checkbox', { name: 'Split container 3' })).toBeChecked());
  fireEvent.change(screen.getByRole('spinbutton', { name: 'Zoom container 3' }), { target: { value: '20' } });
  await waitFor(() => expect(setPresets).toHaveBeenNthCalledWith(2, 3, { split: true, shape: null, zoom: 20 }));
  expect(screen.queryByRole('button', { name: /Save/ })).not.toBeInTheDocument();
});

it('t1_c6e_taps_shooting_order_and_saves_after_each_tap', async () => {
  vi.mocked(setShootingOrder).mockResolvedValueOnce(t([c(1,1,2), c(2,2,1), c(3,3,3)]))
    .mockResolvedValueOnce(t([c(1,1,3), c(2,2,1), c(3,3,2)]));
  await show();
  fireEvent.click(screen.getByRole('button', { name: 'Set shooting order' }));
  fireEvent.click(screen.getByRole('button', { name: 'Tap container 2' }));
  await waitFor(() => expect(setShootingOrder).toHaveBeenNthCalledWith(1, 8, [2,1,3]));
  fireEvent.click(screen.getByRole('button', { name: 'Tap container 3' }));
  await waitFor(() => expect(setShootingOrder).toHaveBeenNthCalledWith(2, 8, [2,3,1]));
});

it('t1_c6f_move_earlier_reorders_the_final_containers', async () => {
  vi.mocked(reorderContainers).mockResolvedValue(t([c(1,1),c(3,2),c(2,3)]));
  await show();
  fireEvent.click(screen.getByRole('button', { name: 'Move earlier container 3' }));
  await waitFor(() => expect(reorderContainers).toHaveBeenCalledExactlyOnceWith(8, [1,3,2]));
});

it('t1_c6g_failed_write_shows_error_and_reloads_stored_presets', async () => {
  vi.mocked(setPresets).mockRejectedValue(new Error('disk full'));
  await show();
  fireEvent.click(screen.getByRole('checkbox', { name: 'Split container 3' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('disk full');
  await waitFor(() => expect(getTemplate).toHaveBeenCalledTimes(2));
  expect(screen.getByRole('checkbox', { name: 'Split container 3' })).not.toBeChecked();
});

it('t1_c6h_confirms_deletion_and_returns_to_create', async () => {
  vi.spyOn(window, 'confirm').mockReturnValue(true);
  vi.mocked(deleteTemplate).mockResolvedValue();
  await show();
  fireEvent.click(screen.getByRole('button', { name: 'Delete template' }));
  await waitFor(() => expect(deleteTemplate).toHaveBeenCalledExactlyOnceWith(8));
  expect(await screen.findByRole('button', { name: 'Create template' })).toBeInTheDocument();
});
