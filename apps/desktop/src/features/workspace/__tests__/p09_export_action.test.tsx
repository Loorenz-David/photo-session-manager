import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ExportProgress } from '@/services/export';

/** The handler the header registered for `export_progress`, so a test can deliver events. */
const delivered = vi.hoisted(() => ({ exportProgress: [] as ((progress: ExportProgress) => void)[] }));

vi.mock('@/services/export', () => ({
  integrationKind: vi.fn(),
  runExport: vi.fn(),
  openExportFolder: vi.fn(),
  onExportProgress: vi.fn(async (handler: (progress: ExportProgress) => void) => {
    delivered.exportProgress.push(handler);
    return () => undefined;
  }),
}));
vi.mock('@/services/sessions', () => ({ listSessions: vi.fn(async () => ({ groups: [] })) }));
vi.mock('@/services/camera', () => ({ cameraStatus: vi.fn(async () => ({ kind: 'disconnected' })), cameraRetry: vi.fn() }));
vi.mock('@/services/events', () => ({ onCameraStatus: vi.fn(async () => () => undefined) }));

import { Toaster } from '@/components/ui/sonner';
import { Header } from '@/features/shell/Header';
import { integrationKind, openExportFolder, runExport } from '@/services/export';

const SESSION = 5;

function renderHeader() {
  render(
    <>
      <Header sessionId={SESSION} theme="light" onThemeChange={() => undefined} onNewSession={() => undefined} />
      <Toaster />
    </>,
  );
}

describe('P09 export action', () => {
  beforeEach(() => {
    delivered.exportProgress.length = 0;
    vi.mocked(integrationKind).mockResolvedValue(null);
    vi.mocked(runExport).mockResolvedValue(1);
    vi.mocked(openExportFolder).mockResolvedValue(undefined);
  });
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it('C7(a) with no integration configured the primary action reads "Export" and clicking it runs the export of the session', async () => {
    renderHeader();
    const button = await screen.findByRole('button', { name: 'Export' });
    expect(screen.queryByRole('button', { name: /Publish/ })).toBeNull();
    fireEvent.click(button);
    expect(runExport).toHaveBeenCalledWith(SESSION);
  });

  it('C7(b) export_progress{state: done, total: 4} raises "Exported 4 files" with "Open folder"', async () => {
    renderHeader();
    await waitFor(() => expect(delivered.exportProgress.length).toBeGreaterThan(0));
    act(() => delivered.exportProgress.forEach(handler => handler({ session_id: SESSION, done: 4, total: 4, state: 'done' })));
    expect(await screen.findByText('Exported 4 files')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Open folder' }));
    expect(openExportFolder).toHaveBeenCalledWith(SESSION);
  });
});
