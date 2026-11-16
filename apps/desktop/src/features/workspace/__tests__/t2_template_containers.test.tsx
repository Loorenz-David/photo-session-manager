import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, renderHook, screen, waitFor, within } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import type { Photo } from '@/services/photos';
import type { WorkspaceEvent } from '@/services/events';
import type { TemplateView } from '@/services/templates';

vi.mock('@/services/templates', () => ({ getTemplateView: vi.fn() }));
vi.mock('@/services/events', () => ({ onWorkspaceEvents: vi.fn() }));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  previewUrl: (path: string) => `asset://${path}`,
}));

import { onWorkspaceEvents } from '@/services/events';
import { getTemplateView } from '@/services/templates';
import { PhotoPanel } from '../PhotoPanel';
import { useTemplateView } from '../useTemplateView';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

afterEach(() => { cleanup(); vi.clearAllMocks(); });

function photo(id: number, classification: Photo['classification'], position: number | null): Photo {
  return {
    id,
    session_id: 7,
    classification,
    position,
    captured_at: id * 10,
    original_preview_path: `/previews/${id}/original.jpg`,
    adjusted_preview_path: `/previews/${id}/adjusted-1.jpg`,
    thumbnail_path: null,
    composition: { photo_id: id, kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: null, frame: null, frame_source: null, origin: 'default', revision: 0, state: 'ok', review_reason: null, updated_at: 0 },
    orientation: 1,
    source: 'capture',
    integrity: 'ok',
    upload_state: null,
  };
}

const container = (final_no: number, shoot_no: number, presets = { split: false, shape: null as string | null, zoom: 0 },
  name: string | null = null, description: string | null = null) => ({
  id: 100 + final_no, final_no, shoot_no, example_file: `${final_no}.jpg`, example_path: `/templates/8/${final_no}.jpg`, presets, name, description,
});

/** The Chair-3 template: shooting order K2, K1, K3; K3 split + zoom 20. */
function chairView(slots: Array<[number, number | null]>, next_shot: number | null = null): TemplateView {
  return {
    templated: true,
    template_id: 8,
    containers: [container(1, 2), container(2, 1), container(3, 3, { split: true, shape: null, zoom: 20 })],
    photos: slots.map(([photo_id, slot]) => ({ photo_id, slot, by_hand: false, differs: false })),
    differs: 0,
    can_shift_back: false,
    next_shot,
  };
}

const untemplated: TemplateView = { templated: false, template_id: null, containers: [], photos: [], differs: 0, can_shift_back: false, next_shot: null };

function panel(photos: Photo[], templateView: TemplateView) {
  render(
    <PhotoPanel
      photos={photos}
      templateView={templateView}
      onFold={() => undefined}
      onImport={async () => undefined}
      sessionsOpen={false}
      onToggleSessions={() => undefined}
      activeId={null}
      selection={new Set()}
      onOpen={() => undefined}
      onToggleSelected={() => undefined}
      onMenu={() => undefined}
    />,
  );
  return screen.getByRole('region', { name: 'Main images' });
}

/** The Main section's cells in order: `P<id>` for a photo tile, `E<n>` for an empty container. */
function cells(main: HTMLElement): string[] {
  return Array.from(main.querySelectorAll('[data-photo-id], [data-container]')).map(cell =>
    cell.hasAttribute('data-photo-id') ? `P${cell.getAttribute('data-photo-id')}` : `E${cell.getAttribute('data-container')}`,
  );
}

it('t2_c8a_empty_containers_are_faded_numbered_placeholders', () => {
  const main = panel([photo(1, 'main', 1)], chairView([[1, 2]]));
  expect(cells(main)).toEqual(['E1', 'P1', 'E3']);
  const placeholders = main.querySelectorAll('[data-container]');
  for (const [i, n] of [[0, '1'], [1, '3']] as const) {
    expect(placeholders[i]).toHaveAttribute('data-container', n);
    expect(placeholders[i]).toHaveAttribute('data-empty');
    expect(placeholders[i].querySelector('.container-number')).toHaveTextContent(new RegExp(`^${n}$`));
  }
  expect(within(main).getByRole('heading').querySelector('span')).toHaveTextContent(/^1$/);
  expect(screen.getByText('1 main · 0 condition')).toBeInTheDocument();
});

it('t2_c8b_filled_containers_then_photos_without_a_container', () => {
  const photos = [photo(11, 'main', 1), photo(12, 'main', 2), photo(13, 'main', 3), photo(14, 'main', 4), photo(15, 'main', 5)];
  const main = panel(photos, chairView([[11, 3], [12, 1], [13, 2], [14, null], [15, null]]));
  expect(cells(main)).toEqual(['P12', 'P13', 'P11', 'P14', 'P15']);
  expect(main.querySelector('[data-empty]')).toBeNull();
});

it('t2_c8c_untemplated_session_looks_as_today', () => {
  const main = panel([photo(1, 'main', 1), photo(2, 'main', 2), photo(3, 'main', 3)], untemplated);
  expect(cells(main)).toEqual(['P1', 'P2', 'P3']);
  expect(screen.queryByRole('button', { name: 'Show template' })).toBeNull();
});

it('t2_c8d_info_button_opens_the_template', async () => {
  panel([photo(1, 'main', 1)], chairView([[1, 2]]));
  fireEvent.click(screen.getByRole('button', { name: 'Show template' }));
  const dialog = await screen.findByRole('dialog');
  const entries = Array.from(dialog.querySelectorAll('[data-container]'));
  expect(entries.map(entry => entry.querySelector('strong')?.textContent)).toEqual(['1', '2', '3']);
  const third = entries[2] as HTMLElement;
  expect(third.querySelector('[data-bw] img')).toBeInTheDocument();
  expect(within(third).getByText('Shot 3')).toBeInTheDocument();
  expect(within(third).getByText('Split · Zoom 20 %')).toBeInTheDocument();
});

it('guide_a_empty_containers_show_their_example_and_shot_number', () => {
  const main = panel([photo(1, 'main', 1)], chairView([[1, 2]]));
  const [k1, k3] = Array.from(main.querySelectorAll<HTMLElement>('[data-empty]'));
  expect(k1.querySelector('[data-bw] img')).toHaveAttribute('src', 'asset:///templates/8/1.jpg');
  expect(within(k1).getByText('Shot 2')).toBeInTheDocument();
  expect(within(k3).getByText('Shot 3')).toBeInTheDocument();
  expect(main.querySelector('[data-next]')).toBeNull();
});

it('guide_b_the_container_of_the_next_shot_is_outlined', () => {
  // Shot 1 (K2) is in; the next shot is shooting number 2, which goes to K1.
  const main = panel([photo(1, 'main', 1)], chairView([[1, 2]], 2));
  const outlined = Array.from(main.querySelectorAll('[data-next]'));
  expect(outlined).toHaveLength(1);
  expect(outlined[0]).toHaveAttribute('data-container', '1');
  expect(within(outlined[0] as HTMLElement).getByText('Next · Shot 2')).toBeInTheDocument();
  expect(screen.getByLabelText('Empty container 1, shot 2, next to shoot')).toBeInTheDocument();
});

it('guide_c_no_outline_once_every_container_has_had_its_shot', () => {
  const photos = [photo(11, 'main', 1), photo(12, 'main', 2)];
  const main = panel(photos, chairView([[11, 2], [12, 1]], null));
  expect(main.querySelector('[data-next]')).toBeNull();
});

it('guide_d_template_window_switches_to_shooting_order_and_marks_the_next_shot', async () => {
  panel([photo(1, 'main', 1)], chairView([[1, 2]], 2));
  fireEvent.click(screen.getByRole('button', { name: 'Show template' }));
  const dialog = await screen.findByRole('dialog');
  const numbers = () => Array.from(dialog.querySelectorAll('[data-container]')).map(entry => entry.querySelector('strong')?.textContent);
  const finals = () => Array.from(dialog.querySelectorAll('[data-container]')).map(entry => entry.getAttribute('data-container'));
  expect(within(dialog).getByRole('button', { name: 'Final order' })).toHaveAttribute('aria-pressed', 'true');
  expect(finals()).toEqual(['1', '2', '3']);
  fireEvent.click(within(dialog).getByRole('button', { name: 'Shooting order' }));
  expect(within(dialog).getByRole('button', { name: 'Shooting order' })).toHaveAttribute('aria-pressed', 'true');
  // Shooting order K2, K1, K3: shot numbers 1, 2, 3, each with its final image number.
  expect(numbers()).toEqual(['1', '2', '3']);
  expect(finals()).toEqual(['2', '1', '3']);
  expect(within(dialog).getByText('Image 2')).toBeInTheDocument();
  const next = dialog.querySelector('[data-next]') as HTMLElement;
  expect(next).toHaveAttribute('data-container', '1');
  expect(within(next).getByText('Next')).toBeInTheDocument();
});

it('label_a_empty_containers_show_their_name_and_the_description_behind_an_info_button', async () => {
  const view = chairView([[1, 2]], 2);
  view.containers[0] = container(1, 2, undefined, 'Front, low', 'Kneel at seat height, whole chair in frame.');
  const main = panel([photo(1, 'main', 1)], view);
  const k1 = main.querySelector('[data-container="1"]') as HTMLElement;
  expect(within(k1).getByText('Front, low')).toBeInTheDocument();
  expect(k1).toHaveAttribute('aria-label', 'Empty container 1 (Front, low), shot 2, next to shoot');
  expect(main.querySelector('[data-container="3"] .container-info')).toBeNull();
  fireEvent.click(within(k1).getByRole('button', { name: 'How to take shot 2' }));
  expect(await screen.findByText('Kneel at seat height, whole chair in frame.')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Show template' }));
  const dialog = await screen.findByRole('dialog');
  const entry = dialog.querySelector('[data-container="1"]') as HTMLElement;
  expect(within(entry).getByText('Front, low')).toBeInTheDocument();
  expect(within(entry).getByText('Kneel at seat height, whole chair in frame.')).toBeInTheDocument();
});

it('t2_c8e_view_reloads_on_events_of_its_session_only', async () => {
  let emit: (event: WorkspaceEvent) => void = () => undefined;
  vi.mocked(onWorkspaceEvents).mockImplementation(async handler => { emit = handler; return () => undefined; });
  vi.mocked(getTemplateView).mockResolvedValue(untemplated);
  renderHook(() => useTemplateView(7));
  await waitFor(() => expect(onWorkspaceEvents).toHaveBeenCalledTimes(1));
  await waitFor(() => expect(getTemplateView).toHaveBeenCalledTimes(1));
  await act(async () => { emit({ kind: 'photo_changed', photo_id: 1, session_id: 7 }); });
  await act(async () => { emit({ kind: 'photo_changed', photo_id: 2, session_id: 8 }); });
  expect(getTemplateView).toHaveBeenCalledTimes(2);
  expect(vi.mocked(getTemplateView).mock.calls).toEqual([[7], [7]]);
});
