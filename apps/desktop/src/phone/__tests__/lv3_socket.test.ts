// LV3·U1 — the phone's socket (master_plan_live §5 PH5, PH7): rows LV3·C5(a)–(c).
import { afterEach, expect, test, vi } from 'vitest';
import { decodeFrame, PhoneSocket, reconnectDelay } from '../socket';
import { advance, phoneEnv } from './helpers';

const started: PhoneSocket[] = [];
afterEach(() => started.splice(0).forEach(socket => socket.stop()));

function open(): PhoneSocket {
  const socket = new PhoneSocket('T', { onMessage: vi.fn(), onFrame: vi.fn(), onDemand: vi.fn(), onOpen: vi.fn(), onRevoked: vi.fn() });
  socket.start();
  started.push(socket);
  return socket;
}

test('lv3_c5a_a_socket_closed_while_visible_is_retried_after_500_1000_2000_4000_4000_ms', async () => {
  const env = phoneEnv();
  open();
  await env.socket().accept();
  expect(env.sockets).toHaveLength(1);
  await env.socket().drop();
  // Each new connection fails at once; the times are measured from the first close.
  const at: number[] = [];
  let elapsed = 0;
  while (at.length < 5 && elapsed < 20_000) {
    await advance(100);
    elapsed += 100;
    if (env.sockets.length === at.length + 2) {
      at.push(elapsed);
      await env.socket().drop();
    }
  }
  expect(at).toEqual([500, 1500, 3500, 7500, 11_500]);
});

test('lv3_c5b_closed_while_hidden_waits_for_the_screen_then_reconnects_at_once_with_a_new_demand', async () => {
  const env = phoneEnv();
  const socket = open();
  await env.socket().accept();
  socket.setLive(true);
  await env.setVisibility('hidden');
  const before = env.socket().demands();
  const lastGen = before[before.length - 1].gen;
  await env.socket().drop();
  await advance(30_000);
  expect(env.sockets).toHaveLength(1);
  await env.setVisibility('visible');
  expect(env.sockets).toHaveLength(2);
  await env.socket().accept();
  const demands = env.socket().demands();
  expect(demands).toHaveLength(1);
  expect(demands[0]).toMatchObject({ type: 'demand', live: true, visible: true });
  expect(demands[0].gen).toBeGreaterThan(lastGen);
  expect(socket.gen).toBe(demands[0].gen);
});

test('lv3_c5c_decodeFrame_reads_the_39_byte_frame_message', () => {
  // The message of LV2·C29(a): PSMF, version 1, seq, gen 9, 1024 × 680, captured_at_ms, 7 JPEG bytes.
  const jpeg = [0xff, 0xd8, 1, 2, 3, 0xff, 0xd9];
  const bytes = new Uint8Array(39);
  const view = new DataView(bytes.buffer);
  bytes.set([0x50, 0x53, 0x4d, 0x46, 1, 0, 0, 0]);
  view.setBigUint64(8, 0x0102030405n, true);
  view.setUint32(16, 9, true);
  view.setUint16(20, 1024, true);
  view.setUint16(22, 680, true);
  view.setBigInt64(24, 1_764_150_123_456n, true);
  bytes.set(jpeg, 32);
  const frame = decodeFrame(bytes.buffer);
  expect(frame).toEqual({ seq: 0x0102030405, gen: 9, width: 1024, height: 680, capturedAtMs: 1_764_150_123_456, jpeg: new Uint8Array(jpeg) });
  expect(frame!.jpeg).toHaveLength(7);
});

test('lv3_c5_reconnectDelay_is_500_1000_2000_4000_then_4000', () => {
  expect([0, 1, 2, 3, 4, 5, 9].map(reconnectDelay)).toEqual([500, 1000, 2000, 4000, 4000, 4000, 4000]);
});
