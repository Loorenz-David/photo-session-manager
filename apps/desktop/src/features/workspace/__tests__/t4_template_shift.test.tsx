import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import type { Photo } from '@/services/photos';
import type { TemplateView } from '@/services/templates';

vi.mock('@/services/templates', () => ({
  templateBack: vi.fn(async () => ({ sessions: [], active_photo_id: null })),
  templateShiftBack: vi.fn(async () => ({ sessions: [], active_photo_id: null })),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  previewUrl: (path: string) => `asset://${path}`,
}));

import { templateShiftBack } from '@/services/templates';
import { PhotoPanel } from '../PhotoPanel';

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
function view(entries: Array<[number, number | null, boolean]>, can_shift_back: boolean): TemplateView {
  return {
    templated: true,
    template_id: 8,
    containers: [container(1, 2), container(2, 1), container(3, 3)],
    photos: entries.map(([photo_id, slot, differs]) => ({ photo_id, slot, by_hand: differs, differs })),
    differs: entries.filter(([, , differs]) => differs).length,
    can_shift_back,
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

it('T4·C3(a) Shift the rest back is shown while the view can shift, also when nothing differs, and calls the command once', async () => {
  panel([photo(11, 'main', 1), photo(13, 'main', 2)], view([[11, 2, false], [13, 3, false]], true));
  expect(screen.queryByRole('button', { name: 'Back to template' })).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Shift the rest back' }));
  await waitFor(() => expect(templateShiftBack).toHaveBeenCalledTimes(1));
  expect(templateShiftBack).toHaveBeenCalledWith(7);
});

it('T4·C3(b) nothing to shift: no Shift the rest back button', () => {
  panel([photo(11, 'main', 1), photo(12, 'main', 2), photo(13, 'main', 3)], view([[11, 2, true], [12, 1, true], [13, 3, false]], false));
  expect(screen.queryByRole('button', { name: 'Shift the rest back' })).toBeNull();
});
