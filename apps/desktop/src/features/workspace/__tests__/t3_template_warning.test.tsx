import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import type { Photo } from '@/services/photos';
import type { TemplateView } from '@/services/templates';

vi.mock('@/services/templates', () => ({ templateBack: vi.fn(async () => ({ sessions: [], active_photo_id: null })) }));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  previewUrl: (path: string) => `asset://${path}`,
}));

import { templateBack } from '@/services/templates';
import { PhotoPanel } from '../PhotoPanel';
import { dropAction } from '../templateDrop';

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

const container = (final_no: number, shoot_no: number) => ({
  id: 100 + final_no, final_no, shoot_no, example_file: null, example_path: null, presets: { split: false, shape: null, zoom: 0 },
});

/** Three containers; `entries` are [photo id, slot, differs]. */
function view(entries: Array<[number, number | null, boolean]>): TemplateView {
  return {
    templated: true,
    template_id: 8,
    containers: [container(1, 2), container(2, 1), container(3, 3)],
    photos: entries.map(([photo_id, slot, differs]) => ({ photo_id, slot, by_hand: differs, differs })),
    differs: entries.filter(([, , differs]) => differs).length,
    can_shift_back: false,
    next_shot: null,
  };
}

function panel(photos: Photo[], templateView: TemplateView) {
  render(
    <PhotoPanel
      photos={photos}
      templateView={templateView}
      sessionId={7}
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
}

it('T3·C4(a) the warning counts the differing photos and Back to template calls the command once', async () => {
  panel([photo(11, 'main', 1), photo(12, 'main', 2), photo(13, 'main', 3)], view([[11, 2, true], [12, 1, true], [13, 3, false]]));
  expect(screen.getByRole('status')).toHaveTextContent('2 photos differ from the template');
  fireEvent.click(screen.getByRole('button', { name: 'Back to template' }));
  await waitFor(() => expect(templateBack).toHaveBeenCalledTimes(1));
  expect(templateBack).toHaveBeenCalledWith(7);
});

it('T3·C4(b) nothing differs: no warning and no Back to template', () => {
  panel([photo(11, 'main', 1), photo(12, 'main', 2)], view([[11, 1, false], [12, 2, false]]));
  expect(screen.queryByRole('status')).toBeNull();
  expect(screen.queryByRole('button', { name: 'Back to template' })).toBeNull();
});

it('T3·C4(c) a differing photo carries the mark; a matching one does not', () => {
  panel([photo(11, 'main', 1), photo(12, 'main', 2)], view([[11, 1, false], [12, 2, true]]));
  const tile = (id: number) => document.querySelector(`.photo-tile[data-photo-id="${id}"]`)!;
  expect(tile(12)).toHaveAttribute('data-differs');
  expect(tile(11)).not.toHaveAttribute('data-differs');
});

it('T3·C4(d) dropAction: container placeholders and placed tiles are template drops; the rest is default', () => {
  const photos = [photo(11, 'main', 1), photo(12, 'main', 2), photo(13, 'main', 3), photo(14, 'condition', 1)];
  const templated = view([[11, 1, false], [12, 3, false], [13, null, true], [14, null, false]]);
  expect(dropAction(12, 'container:2', templated, photos)).toEqual({ kind: 'template', container: 2 });
  expect(dropAction(12, 'photo:11', templated, photos)).toEqual({ kind: 'template', container: 1 });
  // T4: a photo without a slot over a filled container's tile is a retake onto that container.
  expect(dropAction(14, 'photo:11', templated, photos)).toEqual({ kind: 'template', container: 1 });
  expect(dropAction(14, 'photo:13', templated, photos)).toEqual({ kind: 'default' });
  expect(dropAction(12, 'photo:13', templated, photos)).toEqual({ kind: 'default' });
  expect(dropAction(12, 'group:condition', templated, photos)).toEqual({ kind: 'default' });
  const untemplated: TemplateView = { templated: false, template_id: null, containers: [], photos: [], differs: 0, can_shift_back: false, next_shot: null };
  expect(dropAction(12, 'photo:11', untemplated, photos)).toEqual({ kind: 'default' });
});
