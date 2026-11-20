import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ConversationsChanged } from '@/services/events';
import type {
  AttachablePhoto,
  ConversationAttachable,
  ConversationMessage,
  ConversationView as View,
  PendingReply,
} from '@/services/conversations';

vi.mock('@/services/conversations', () => ({
  conversationGet: vi.fn(),
  conversationMarkRead: vi.fn(async () => undefined),
  conversationAttachable: vi.fn(),
  conversationReply: vi.fn(),
  conversationRetry: vi.fn(async () => undefined),
  conversationDiscard: vi.fn(async () => undefined),
  conversationMarkAnswered: vi.fn(async () => undefined),
}));
vi.mock('@/services/events', () => ({
  onConversationsChanged: vi.fn(async (_handler: (event: ConversationsChanged) => void) => () => undefined),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('sonner', () => ({ toast: vi.fn() }));

import { toast } from 'sonner';
import {
  conversationAttachable,
  conversationDiscard,
  conversationGet,
  conversationMarkAnswered,
  conversationReply,
  conversationRetry,
} from '@/services/conversations';
import { ServiceError } from '@/services/sessions';
import { ConversationView } from '../ConversationView';

const NOW = 1_795_176_000_000;

function message(extra: Partial<ConversationMessage> = {}): ConversationMessage {
  return {
    id: 'm9',
    author: { kind: 'asker', name: 'Elin Lindqvist' },
    text: 'Is the glass original?',
    quote: null,
    reply_to: null,
    attachments: [],
    created_at: NOW - 60_000,
    edited_at: null,
    ...extra,
  };
}

function view(extra: Partial<View> = {}): View {
  return {
    id: 'c1',
    sku: 'BEYO-1880',
    product_title: 'Vintage Brass Wall Mirror, Round',
    topic: 'Condition',
    blocking: true,
    status: 'open',
    resolved_by: null,
    session_id: 7,
    messages: [message()],
    pending: [],
    can_send: true,
    off_reason: null,
    ...extra,
  };
}

function pending(extra: Partial<PendingReply> = {}): PendingReply {
  return {
    client_mutation_id: 'u-1',
    text: 'Original glass.',
    reply_to: null,
    attachments: [],
    state: 'failed',
    failure: "Couldn't send",
    created_at: NOW,
    ...extra,
  };
}

function photos(n: number): AttachablePhoto[] {
  return Array.from({ length: n }, (_, i) => ({
    media_id: `gid-${i + 1}`,
    photo_id: i + 1,
    slot: 'single',
    role: 'gallery',
    preview_path: `/previews/p${i + 1}.jpg`,
  }));
}

function attachable(extra: Partial<ConversationAttachable> = {}): ConversationAttachable {
  return { photos: photos(3), reason: null, ...extra };
}

async function shown(v: View = view(), a: ConversationAttachable = attachable()) {
  vi.mocked(conversationGet).mockResolvedValue(v);
  vi.mocked(conversationAttachable).mockResolvedValue(a);
  render(<ConversationView id={v.id} onBack={() => undefined} />);
  const node = await screen.findByTestId('conversation-view');
  await within(node).findByText(v.topic);
  await waitFor(() => expect(vi.mocked(conversationAttachable)).toHaveBeenCalled());
  return node;
}

function box(): HTMLTextAreaElement {
  return screen.getByRole('textbox', { name: 'Reply' }) as HTMLTextAreaElement;
}

function type(text: string) {
  fireEvent.change(box(), { target: { value: text } });
}

function enter(shift = false) {
  fireEvent.keyDown(box(), { key: 'Enter', shiftKey: shift });
}

function replyCalls() {
  return vi.mocked(conversationReply).mock.calls.map(call => call[0]);
}

beforeEach(() => {
  vi.spyOn(document, 'hasFocus').mockReturnValue(true);
  vi.spyOn(crypto, 'randomUUID').mockReturnValue('u-1-0000-0000-0000-000000000000' as `${string}-${string}-${string}-${string}-${string}`);
  vi.mocked(conversationReply).mockResolvedValue(pending({ state: 'sending', failure: null }));
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.mocked(conversationReply).mockReset();
  vi.mocked(conversationGet).mockReset();
  vi.mocked(conversationAttachable).mockReset();
  vi.mocked(conversationRetry).mockClear();
  vi.mocked(conversationDiscard).mockClear();
  vi.mocked(conversationMarkAnswered).mockReset();
  vi.mocked(toast).mockClear();
});

function stubIds(...ids: string[]) {
  const spy = vi.mocked(crypto.randomUUID);
  spy.mockReset();
  ids.forEach(id => spy.mockReturnValueOnce(id as `${string}-${string}-${string}-${string}-${string}`));
}

// ---------- C3·C16 the text box ----------

it('c3_c16a_enter_sends_the_text_and_empties_the_box', async () => {
  await shown();
  type('Original glass.');
  enter();
  await waitFor(() => expect(vi.mocked(conversationReply)).toHaveBeenCalledTimes(1));
  expect(replyCalls()[0].text).toBe('Original glass.');
  expect(box().value).toBe('');
});

it('c3_c16b_shift_enter_is_a_new_line', async () => {
  await shown();
  type('a');
  enter(true);
  await Promise.resolve();
  expect(vi.mocked(conversationReply)).not.toHaveBeenCalled();
  expect(box().value).toBe('a');
  type('a\nb');
  expect(box().value).toBe('a\nb');
});

it('c3_c16c_empty_or_too_long_text_is_not_sent', async () => {
  await shown();
  type('     ');
  enter();
  type('é'.repeat(4001));
  enter();
  await Promise.resolve();
  expect(vi.mocked(conversationReply)).not.toHaveBeenCalled();
});

// ---------- C3·C17 one id per press ----------

it('c3_c17a_the_call_carries_the_id_made_at_the_press', async () => {
  stubIds('u-1');
  await shown();
  type('Original glass.');
  enter();
  await waitFor(() => expect(vi.mocked(conversationReply)).toHaveBeenCalledTimes(1));
  expect(replyCalls()[0].client_mutation_id).toBe('u-1');
});

it('c3_c17b_two_presses_carry_two_ids', async () => {
  stubIds('u-1', 'u-2');
  await shown();
  type('First.');
  enter();
  type('Second.');
  enter();
  await waitFor(() => expect(vi.mocked(conversationReply)).toHaveBeenCalledTimes(2));
  expect(replyCalls().map(c => c.client_mutation_id)).toEqual(['u-1', 'u-2']);
});

it('c3_c17c_after_a_send_the_sending_reply_is_shown_dimmed', async () => {
  const node = await shown();
  vi.mocked(conversationGet).mockResolvedValue(view({ pending: [pending({ state: 'sending', failure: null })] }));
  type('Original glass.');
  enter();
  const bubble = await within(node).findByTestId('pending-u-1');
  expect(bubble).toHaveClass('message-pending');
  expect(within(bubble).getByText('Sending…')).toBeInTheDocument();
  expect(within(bubble).getByText('Original glass.')).toBeInTheDocument();
});

// ---------- C3·C18 Retry and Remove ----------

it('c3_c18a_retry_reuses_the_reply_and_never_makes_a_new_one', async () => {
  const node = await shown(view({ pending: [pending()] }));
  fireEvent.click(within(within(node).getByTestId('pending-u-1')).getByRole('button', { name: 'Retry' }));
  await waitFor(() => expect(vi.mocked(conversationRetry)).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationRetry)).toHaveBeenCalledWith('u-1');
  expect(vi.mocked(conversationReply)).not.toHaveBeenCalled();
});

it('c3_c18b_remove_discards_the_failed_reply', async () => {
  const node = await shown(view({ pending: [pending()] }));
  fireEvent.click(within(within(node).getByTestId('pending-u-1')).getByRole('button', { name: 'Remove' }));
  await waitFor(() => expect(vi.mocked(conversationDiscard)).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationDiscard)).toHaveBeenCalledWith('u-1');
});

// ---------- C3·C19 replying to one message ----------

it('c3_c19a_reply_shows_the_target_above_the_box', async () => {
  const node = await shown();
  fireEvent.click(within(within(node).getByTestId('message-m9')).getByRole('button', { name: 'Reply' }));
  const line = within(node).getByText('↩ Elin Lindqvist: Is the glass original?');
  expect(line.closest('.composer-reply')).not.toBeNull();
  expect(within(node).getByRole('button', { name: 'Cancel reply' })).toBeInTheDocument();
});

it('c3_c19b_the_send_carries_the_target_and_clears_it', async () => {
  const node = await shown();
  fireEvent.click(within(within(node).getByTestId('message-m9')).getByRole('button', { name: 'Reply' }));
  type('Yes, original.');
  enter();
  await waitFor(() => expect(vi.mocked(conversationReply)).toHaveBeenCalledTimes(1));
  expect(replyCalls()[0].reply_to_message_id).toBe('m9');
  expect(within(node).queryByText('↩ Elin Lindqvist: Is the glass original?')).toBeNull();
});

it('c3_c19c_cancelling_the_target_sends_without_it', async () => {
  const node = await shown();
  fireEvent.click(within(within(node).getByTestId('message-m9')).getByRole('button', { name: 'Reply' }));
  fireEvent.click(within(node).getByRole('button', { name: 'Cancel reply' }));
  type('Yes, original.');
  enter();
  await waitFor(() => expect(vi.mocked(conversationReply)).toHaveBeenCalledTimes(1));
  expect(replyCalls()[0].reply_to_message_id).toBeNull();
});

// ---------- C3·C20 the photo picker ----------

it('c3_c20a_ticked_photos_travel_in_the_order_ticked', async () => {
  const node = await shown();
  fireEvent.click(within(node).getByRole('button', { name: 'Add photos' }));
  const picker = within(node).getByRole('group', { name: 'Published photos' });
  expect(within(picker).getAllByRole('checkbox')).toHaveLength(3);
  expect(within(picker).getAllByRole('img').map(i => i.getAttribute('src'))).toEqual([
    'asset:///previews/p1.jpg',
    'asset:///previews/p2.jpg',
    'asset:///previews/p3.jpg',
  ]);
  fireEvent.click(within(picker).getByRole('checkbox', { name: 'Photo 3' }));
  fireEvent.click(within(picker).getByRole('checkbox', { name: 'Photo 1' }));
  type('Here they are.');
  enter();
  await waitFor(() => expect(vi.mocked(conversationReply)).toHaveBeenCalledTimes(1));
  expect(replyCalls()[0].attachment_media_ids).toEqual(['gid-3', 'gid-1']);
});

it('c3_c20b_at_most_ten_photos_can_be_ticked', async () => {
  const node = await shown(view(), attachable({ photos: photos(12) }));
  fireEvent.click(within(node).getByRole('button', { name: 'Add photos' }));
  const picker = within(node).getByRole('group', { name: 'Published photos' });
  for (let i = 1; i <= 10; i++) fireEvent.click(within(picker).getByRole('checkbox', { name: `Photo ${i}` }));
  expect(within(picker).getByRole('checkbox', { name: 'Photo 11' })).toBeDisabled();
  expect(within(picker).getByRole('checkbox', { name: 'Photo 12' })).toBeDisabled();
  type('Ten photos.');
  enter();
  await waitFor(() => expect(vi.mocked(conversationReply)).toHaveBeenCalledTimes(1));
  expect(replyCalls()[0].attachment_media_ids).toEqual(photos(10).map(p => p.media_id));
});

it('c3_c20c_nothing_published_says_publish_first', async () => {
  const node = await shown(view(), attachable({ photos: [], reason: 'nothing_published' }));
  fireEvent.click(within(node).getByRole('button', { name: 'Add photos' }));
  expect(within(node).getByText('Publish first to attach photos')).toBeInTheDocument();
  expect(within(node).queryAllByRole('checkbox')).toHaveLength(0);
  expect(within(node).queryByRole('img')).toBeNull();
});

it('c3_c20d_not_on_this_mac_disables_photos_but_not_text', async () => {
  const node = await shown(view({ session_id: 7 }), attachable({ photos: [], reason: 'not_on_this_mac' }));
  await waitFor(() => expect(within(node).getByRole('button', { name: 'Add photos' })).toBeDisabled());
  type('Answered without photos.');
  enter();
  await waitFor(() => expect(vi.mocked(conversationReply)).toHaveBeenCalledTimes(1));
});

// ---------- C3·C21 Mark answered, resolved, off ----------

it('c3_c21a_an_open_conversation_offers_mark_answered', async () => {
  const node = await shown(view({ status: 'open' }));
  fireEvent.click(within(node).getByRole('button', { name: 'Mark answered' }));
  await waitFor(() => expect(vi.mocked(conversationMarkAnswered)).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationMarkAnswered)).toHaveBeenCalledWith('c1');
});

it('c3_c21b_answered_or_resolved_offers_no_mark_answered', async () => {
  const answered = await shown(view({ status: 'answered' }));
  expect(within(answered).queryByRole('button', { name: 'Mark answered' })).toBeNull();
  cleanup();
  const resolved = await shown(view({ status: 'resolved' }));
  expect(within(resolved).queryByRole('button', { name: 'Mark answered' })).toBeNull();
});

it('c3_c21c_already_resolved_is_told_and_the_view_re_read', async () => {
  const node = await shown(view({ status: 'open' }));
  const reads = vi.mocked(conversationGet).mock.calls.length;
  vi.mocked(conversationMarkAnswered).mockRejectedValue(
    new ServiceError({ code: 'conversation_resolved', message: 'Pricing has already resolved this', data: {} }),
  );
  fireEvent.click(within(node).getByRole('button', { name: 'Mark answered' }));
  await waitFor(() => expect(vi.mocked(toast)).toHaveBeenCalledWith('Pricing has already resolved this'));
  await waitFor(() => expect(vi.mocked(conversationGet).mock.calls.length).toBeGreaterThan(reads));
});

it('c3_c21d_resolved_notes_the_reopen_and_off_disables_the_box', async () => {
  const resolved = await shown(view({ status: 'resolved', can_send: true }));
  expect(within(resolved).getByText('Replying re-opens it as answered')).toBeInTheDocument();
  expect(box()).toBeEnabled();
  cleanup();
  const reason = 'Pricing refused the key: check Settings → Pricing';
  const off = await shown(view({ status: 'open', can_send: false, off_reason: reason }));
  expect(box()).toBeDisabled();
  expect(within(off).getByText(reason)).toBeInTheDocument();
});
