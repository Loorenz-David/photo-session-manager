import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { PhoneSection } from '../PhoneSection';
import {
  phonePairingCreate, phonePairingRename, phonePairingRevoke, phonePairingsList, phoneServerStatus,
  type Pairing, type PairingCreated,
} from '@/services/phone';

vi.mock('@/services/phone', () => ({
  phonePairingCreate: vi.fn(), phonePairingsList: vi.fn(), phonePairingRename: vi.fn(), phonePairingRevoke: vi.fn(),
  phoneServerStatus: vi.fn(),
}));
vi.mock('@/services/settings', () => ({ saveSettings: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const running = { enabled: true, running: true, port: 8686, error: null };
const created: PairingCreated = {
  id: 1, name: 'Phone 1',
  url_name: 'http://Studio-Mac.local:8686/phone/?pair=TOKEN', url_address: 'http://192.168.1.20:8686/phone/?pair=TOKEN',
  qr_name_svg: '<svg data-qr="name"></svg>', qr_address_svg: '<svg data-qr="address"></svg>',
};
const pairing = (id: number, name: string, connected = false): Pairing =>
  ({ id, name, created_at: 1_700_000_000_000, last_seen_at: null, connected });

beforeEach(() => {
  vi.mocked(phoneServerStatus).mockResolvedValue(running);
  vi.mocked(phonePairingsList).mockResolvedValue([]);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('LV2·C16(a) Pair a phone → Create shows the QR code and the .local URL, then the address on request', async () => {
  vi.mocked(phonePairingCreate).mockResolvedValue(created);
  const { container } = render(<PhoneSection />);
  fireEvent.click(await screen.findByRole('button', { name: 'Pair a phone' }));
  fireEvent.click(screen.getByRole('button', { name: 'Create' }));
  expect(await screen.findByText(created.url_name)).toBeInTheDocument();
  expect(phonePairingCreate).toHaveBeenCalledTimes(1);
  expect(container.querySelector('svg[data-qr="name"]')).not.toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Show the address instead' }));
  expect(screen.getByText(created.url_address)).toBeInTheDocument();
  expect(container.querySelector('svg[data-qr="address"]')).not.toBeNull();
  expect(container.querySelector('svg[data-qr="name"]')).toBeNull();
});

it('LV2·C16(b) Remove on the second pairing, confirmed, revokes it and the re-read list shows only the first', async () => {
  vi.mocked(phonePairingsList).mockResolvedValueOnce([pairing(1, 'Phone 1'), pairing(2, 'Phone 2')])
    .mockResolvedValue([pairing(1, 'Phone 1')]);
  vi.mocked(phonePairingRevoke).mockResolvedValue();
  render(<PhoneSection />);
  fireEvent.click(await screen.findByRole('button', { name: 'Remove Phone 2' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Remove phone' }));
  await waitFor(() => expect(phonePairingRevoke).toHaveBeenCalledTimes(1));
  expect(phonePairingRevoke).toHaveBeenCalledWith(2);
  const list = screen.getByRole('list', { name: 'Paired phones' });
  await waitFor(() => expect(within(list).queryByText('Phone 2')).not.toBeInTheDocument());
  expect(within(list).getByText('Phone 1')).toBeInTheDocument();
});

it('LV2·C16(c) a server that could not bind shows why', async () => {
  vi.mocked(phoneServerStatus).mockResolvedValue({ enabled: true, running: false, port: 8686, error: 'port 8686 in use' });
  render(<PhoneSection />);
  expect(await screen.findByText('Phone server could not start: port 8686 in use')).toBeInTheDocument();
});

it('LV2·C16(d) renaming the first pairing to "iPhone" saves that name once', async () => {
  vi.mocked(phonePairingsList).mockResolvedValue([pairing(1, 'Phone 1'), pairing(2, 'Phone 2')]);
  vi.mocked(phonePairingRename).mockResolvedValue();
  render(<PhoneSection />);
  fireEvent.click(await screen.findByRole('button', { name: 'Rename Phone 1' }));
  fireEvent.change(screen.getByLabelText('Name of Phone 1'), { target: { value: 'iPhone' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save name' }));
  await waitFor(() => expect(phonePairingRename).toHaveBeenCalledTimes(1));
  expect(phonePairingRename).toHaveBeenCalledWith(1, 'iPhone');
});

it('LV2·C16(e) a connected phone shows its connected dot', async () => {
  vi.mocked(phonePairingsList).mockResolvedValue([pairing(1, 'Phone 1', true), pairing(2, 'Phone 2')]);
  render(<PhoneSection />);
  const list = await screen.findByRole('list', { name: 'Paired phones' });
  await waitFor(() => expect(within(list).getAllByRole('listitem')).toHaveLength(2));
  const [first, second] = within(list).getAllByRole('listitem');
  expect(within(first).getByLabelText('Connected')).toBeInTheDocument();
  expect(within(second).queryByLabelText('Connected')).not.toBeInTheDocument();
});
