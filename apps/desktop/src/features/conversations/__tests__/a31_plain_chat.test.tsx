import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type {
  ConversationListRow,
  ConversationMessage,
  ConversationThread,
  ConversationView as View,
} from '@/services/conversations';

vi.mock('@/services/conversations', () => ({
  conversationThread: vi.fn(),
  conversationMarkRead: vi.fn(async () => undefined),
  conversationAttachable: vi.fn(async () => ({ photos: [], reason: 'nothing_published' })),
  conversationReply: vi.fn(async () => ({})),
  conversationRetry: vi.fn(),
  conversationDiscard: vi.fn(),
  conversationMarkAnswered: vi.fn(async () => undefined),
}));
vi.mock('@/services/events', () => ({ onConversationsChanged: vi.fn(async () => () => undefined) }));

import { conversationMarkRead, conversationReply, conversationThread } from '@/services/conversations';
import { ConversationRow } from '../ConversationRow';
import { chatEntries, ConversationView, defaultQuestion } from '../ConversationView';

/** 2026-11-22T10:00:00Z. */
const NOW = 1_795_341_600_000;

function message(id: string, text: string, kind: 'asker' | 'me', at: number): ConversationMessage {
  return {
    id,
    author: { kind, name: kind === 'asker' ? 'Elin Lindqvist' : 'Photographer' },
    text,
    quote: null,
    reply_to: null,
    attachments: [],
    created_at: at,
    edited_at: null,
  };
}

function question(id: string, topic: string, status: string, messages: ConversationMessage[]): View {
  return {
    id,
    sku: 'Ch1-201126',
    product_title: 'Ch1-201126',
    topic,
    blocking: false,
    status,
    resolved_by: null,
    session_id: null,
    messages,
    pending: [],
    can_send: true,
    off_reason: null,
  };
}

/** The owner's case of 2026-11-22: "title?" answered, "why not ?" answered, "yes" open. */
const Q1 = question('q1', 'Title', 'answered', [message('m1', 'title?', 'asker', NOW - 3 * 3600_000), message('m2', 'so im not in work pc', 'me', NOW - 3000_000)]);
const Q2 = question('q2', 'Other', 'answered', [message('m3', 'why not ?', 'asker', NOW - 120_000), message('m4', 'because', 'me', NOW - 90_000)]);
const Q3 = question('q3', 'Other', 'open', [message('m5', 'yes', 'asker', NOW - 60_000)]);

function thread(questions: View[]): ConversationThread {
  return { product_id: '16241668882762', sku: 'Ch1-201126', product_title: 'Ch1-201126', session_id: null, questions };
}

function row(extra: Partial<ConversationListRow> = {}): ConversationListRow {
  return {
    id: 'q3',
    product_id: '16241668882762',
    questions: 3,
    sku: 'Ch1-201126',
    product_title: 'Ch1-201126',
    topic: 'Other',
    status: 'open',
    blocking: false,
    waiting: false,
    last_line: 'yes',
    last_activity_at: NOW - 60_000,
    has_news: true,
    session_id: null,
    ...extra,
  };
}

beforeEach(() => {
  vi.mocked(conversationThread).mockResolvedValue(thread([Q1, Q2, Q3]));
  vi.spyOn(document, 'hasFocus').mockReturnValue(true);
  vi.spyOn(crypto, 'randomUUID').mockReturnValue('00000000-0000-4000-8000-000000000001');
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});


async function openThread(id = 'q3') {
  render(<ConversationView id={id} onBack={() => undefined} />);
  await screen.findByTestId('message-m5');
}

function send(text: string) {
  fireEvent.change(screen.getByRole('textbox', { name: 'Reply' }), { target: { value: text } });
  fireEvent.click(screen.getByRole('button', { name: 'Send' }));
}

/** The chat's messages in screen order, by id. */
function shownOrder(): string[] {
  return screen.getAllByTestId(/^message-/).map(m => m.dataset.testid!.replace('message-', ''));
}

it('a30_row_names_how_many_questions_the_product_has', () => {
  const { rerender } = render(<ConversationRow row={row()} onSelect={() => undefined} now={NOW} />);
  expect(screen.getByText('3 questions')).toBeInTheDocument();
  rerender(<ConversationRow row={row({ questions: 1 })} onSelect={() => undefined} now={NOW} />);
  expect(screen.queryByText(/questions$/)).not.toBeInTheDocument();
});

it('a31_one_chat_in_time_order_across_questions', async () => {
  // A later question whose message is older than an earlier question's reply: time decides, not the question.
  const second = question('q0', 'Condition', 'answered', [message('m0', 'scratches?', 'asker', NOW - 2 * 3600_000)]);
  vi.mocked(conversationThread).mockResolvedValue(thread([Q1, second, Q2, Q3]));
  await openThread();
  expect(shownOrder()).toEqual(['m1', 'm0', 'm2', 'm3', 'm4', 'm5']);
  expect(screen.getByText('Questions')).toBeInTheDocument();
  expect(screen.getByText('Not on this Mac')).toBeInTheDocument();
});

it('a31_equal_times_keep_question_then_message_order_and_unconfirmed_replies_follow', () => {
  const a = question('a', 'X', 'open', [message('a1', 'one', 'asker', NOW), message('a2', 'two', 'asker', NOW)]);
  const b: View = {
    ...question('b', 'Y', 'open', [message('b1', 'three', 'asker', NOW)]),
    pending: [{ client_mutation_id: 'u1', text: 'sending', reply_to: null, attachments: [], state: 'sending', failure: null, created_at: NOW }],
  };
  expect(chatEntries([a, b]).map(e => (e.kind === 'message' ? e.message.id : e.pending.client_mutation_id))).toEqual([
    'a1',
    'a2',
    'b1',
    'u1',
  ]);
});

it('a31_no_dividers_status_lines_mark_answered_or_answering_in_the_thread', async () => {
  await openThread();
  const view = screen.getByTestId('conversation-view');
  expect(view.querySelector('.conversation-question')).toBeNull();
  expect(view.querySelector('.conversation-status-dot')).toBeNull();
  expect(within(view).queryByText('Title')).not.toBeInTheDocument();
  expect(within(view).queryByText('Other')).not.toBeInTheDocument();
  expect(within(view).queryByText(/^(Open|Answered|Resolved)$/)).not.toBeInTheDocument();
  expect(within(view).queryByRole('button', { name: 'Mark answered' })).not.toBeInTheDocument();
  expect(within(view).queryByRole('combobox')).not.toBeInTheDocument();
  expect(within(view).queryByText('Answering')).not.toBeInTheDocument();
});

it('a31_each_message_shows_who_sent_it_and_when', async () => {
  await openThread();
  const received = screen.getByTestId('message-m3');
  expect(within(received).getByText('Elin Lindqvist')).toBeInTheDocument();
  expect(received.querySelector('time')).not.toBeNull();
  expect(within(screen.getByTestId('message-m4')).getByText('You')).toBeInTheDocument();
});

it('a30_the_default_question_is_the_newest_unresolved_else_the_newest', () => {
  expect(defaultQuestion([Q1, Q2, Q3])?.id).toBe('q3');
  expect(defaultQuestion([Q1, Q3, { ...Q2, status: 'resolved' }])?.id).toBe('q3');
  const allResolved = [Q1, Q2, Q3].map(q => ({ ...q, status: 'resolved' }));
  expect(defaultQuestion(allResolved)?.id).toBe('q3');
  expect(defaultQuestion([])).toBeUndefined();
});

it('a31_without_a_reply_target_the_send_goes_to_the_newest_unresolved_question', async () => {
  // Opened from the oldest question (a notification), the reply still goes to the newest unresolved one.
  await openThread('q1');
  send('Yes, the title is Ch1');
  await waitFor(() => expect(conversationReply).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationReply).mock.calls[0][0]).toMatchObject({ conversation_id: 'q3', reply_to_message_id: null });
});

it('a31_reply_from_the_message_button_targets_that_messages_question', async () => {
  await openThread();
  const m1 = screen.getByTestId('message-m1');
  fireEvent.mouseEnter(m1);
  fireEvent.click(within(m1).getByRole('button', { name: 'Reply' }));
  expect(screen.getByText('↩ Elin Lindqvist: title?').closest('.composer-reply')).not.toBeNull();
  send('The title comes from the shop');
  await waitFor(() => expect(conversationReply).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationReply).mock.calls[0][0]).toMatchObject({ conversation_id: 'q1', reply_to_message_id: 'm1' });
  expect(screen.queryByRole('button', { name: 'Cancel reply' })).not.toBeInTheDocument();
});

it('a31_right_click_offers_reply_and_it_targets_that_messages_question', async () => {
  await openThread();
  fireEvent.contextMenu(screen.getByTestId('message-m3'));
  const menu = screen.getByRole('menu', { name: 'Message' });
  fireEvent.click(within(menu).getByRole('menuitem', { name: 'Reply' }));
  expect(screen.queryByRole('menu', { name: 'Message' })).not.toBeInTheDocument();
  send('Because it was never set');
  await waitFor(() => expect(conversationReply).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationReply).mock.calls[0][0]).toMatchObject({ conversation_id: 'q2', reply_to_message_id: 'm3' });
});

it('a31_escape_closes_the_right_click_menu_without_replying', async () => {
  await openThread();
  fireEvent.contextMenu(screen.getByTestId('message-m3'));
  fireEvent.keyDown(document, { key: 'Escape' });
  expect(screen.queryByRole('menu', { name: 'Message' })).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Cancel reply' })).not.toBeInTheDocument();
});

it('a31_cancelling_a_reply_sends_to_the_default_question_without_reply_to', async () => {
  await openThread();
  fireEvent.click(within(screen.getByTestId('message-m1')).getByRole('button', { name: 'Reply' }));
  fireEvent.click(screen.getByRole('button', { name: 'Cancel reply' }));
  send('Never mind');
  await waitFor(() => expect(conversationReply).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationReply).mock.calls[0][0]).toMatchObject({ conversation_id: 'q3', reply_to_message_id: null });
});

it('a30_opening_the_thread_marks_every_question_read', async () => {
  await openThread();
  await waitFor(() => expect(conversationMarkRead).toHaveBeenCalledTimes(3));
  expect(vi.mocked(conversationMarkRead).mock.calls.map(c => c[0]).sort()).toEqual(['q1', 'q2', 'q3']);
});
