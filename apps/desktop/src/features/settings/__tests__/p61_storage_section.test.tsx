import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { DiskWarning, StorageSection } from '../StorageSection';
import { getStorage, setMinFreeGb, type Storage } from '@/services/storage';

vi.mock('@/services/storage', () => ({ getStorage: vi.fn(), setMinFreeGb: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const report: Storage = {
  computed: true, disk: 'the startup disk', free_bytes: 123_400_000_000, raws_bytes: 9_400_000_000, duplicates_bytes: 4_700_000_000,
  preview_exports_bytes: 70_000_000, preview_cache_bytes: 280_000_000, masters_bytes: 1_200_000_000, min_free_gb: 20, warning: null,
};
const full = 'Disk almost full: 3.2 GB free; nothing more can be cleared safely';

afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('C4 Settings → Storage shows the space per kind and saves the minimum free space', async () => {
  vi.mocked(getStorage).mockResolvedValue(report);
  vi.mocked(setMinFreeGb).mockResolvedValue({ ...report, min_free_gb: 150 });
  render(<StorageSection />);
  expect(await screen.findByText('Free space on the startup disk')).toBeInTheDocument();
  const line = (label: string) => screen.getByText(label).nextElementSibling?.textContent;
  expect([line('Free space on the startup disk'), line('RAW files'), line('Duplicates'), line('Capture One preview exports'), line('Previews'), line('Masters')])
    .toEqual(['123.4 GB', '9.4 GB', '4.7 GB', '70 MB', '280 MB', '1.2 GB']);
  const input = screen.getByLabelText('Minimum free space (GB)');
  expect(input).toHaveValue(20);
  fireEvent.change(input, { target: { value: '150' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save minimum' }));
  await waitFor(() => expect(setMinFreeGb).toHaveBeenCalledWith(150));
  expect(screen.getByLabelText('Minimum free space (GB)')).toHaveValue(150);
});

it('C4(b) the header and the Storage section show the disk-almost-full warning', async () => {
  vi.mocked(getStorage).mockResolvedValue({ ...report, free_bytes: 3_200_000_000, min_free_gb: 20, warning: full });
  render(<><DiskWarning /><StorageSection /></>);
  await waitFor(() => expect(screen.getAllByText(full)).toHaveLength(2));
});

it('no warning in the header while the minimum can be met', async () => {
  vi.mocked(getStorage).mockResolvedValue(report);
  render(<DiskWarning />);
  await waitFor(() => expect(getStorage).toHaveBeenCalled());
  expect(screen.queryByRole('status')).not.toBeInTheDocument();
});
