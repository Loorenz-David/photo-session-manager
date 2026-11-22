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
import { ConversationView, defaultQuestion } from '../ConversationView';

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
  await screen.findByTestId('question-q3');
}

function send(text: string) {
  fireEvent.change(screen.getByRole('textbox', { name: 'Reply' }), { target: { value: text } });
  fireEvent.click(screen.getByRole('button', { name: 'Send' }));
}

it('a30_row_names_how_many_questions_the_product_has', () => {
  const { rerender } = render(<ConversationRow row={row()} onSelect={() => undefined} now={NOW} />);
  expect(screen.getByText('3 questions')).toBeInTheDocument();
  rerender(<ConversationRow row={row({ questions: 1 })} onSelect={() => undefined} now={NOW} />);
  expect(screen.queryByText(/questions$/)).not.toBeInTheDocument();
});

it('a30_thread_stacks_every_question_oldest_first_each_with_its_header_line', async () => {
  await openThread();
  const sections = screen.getAllByTestId(/^question-/);
  expect(sections.map(s => s.dataset.questionId)).toEqual(['q1', 'q2', 'q3']);
  expect(within(sections[0]).getByText('Title', { selector: '.conversation-header-topic' })).toBeInTheDocument();
  expect(within(sections[0]).getByText('title?')).toBeInTheDocument();
  expect(within(sections[1]).getByText('because')).toBeInTheDocument();
  expect(within(sections[2]).getByText('yes')).toBeInTheDocument();
  expect(within(sections[2]).getByRole('button', { name: 'Mark answered' })).toBeInTheDocument();
  expect(within(sections[0]).queryByRole('button', { name: 'Mark answered' })).not.toBeInTheDocument();
  expect(screen.getByText('Questions')).toBeInTheDocument();
  expect(screen.getByText('Not on this Mac')).toBeInTheDocument();
});

it('a30_the_default_question_is_the_newest_unresolved_else_the_newest', () => {
  expect(defaultQuestion([Q1, Q2, Q3])?.id).toBe('q3');
  expect(defaultQuestion([Q1, Q3, { ...Q2, status: 'resolved' }])?.id).toBe('q3');
  const allResolved = [Q1, Q2, Q3].map(q => ({ ...q, status: 'resolved' }));
  expect(defaultQuestion(allResolved)?.id).toBe('q3');
  expect(defaultQuestion([])).toBeUndefined();
});

it('a30_the_composer_answers_the_opened_question_and_sends_to_it', async () => {
  await openThread('q3');
  expect(screen.getByRole('combobox', { name: 'Answering' })).toHaveValue('q3');
  send('Yes, the title is Ch1');
  await waitFor(() => expect(conversationReply).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationReply).mock.calls[0][0]).toMatchObject({ conversation_id: 'q3', reply_to_message_id: null });
});

it('a30_reply_on_a_message_retargets_the_composer_to_that_question', async () => {
  await openThread('q3');
  fireEvent.click(within(screen.getByTestId('message-m1')).getByRole('button', { name: 'Reply' }));
  expect(screen.getByRole('combobox', { name: 'Answering' })).toHaveValue('q1');
  send('The title comes from the shop');
  await waitFor(() => expect(conversationReply).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationReply).mock.calls[0][0]).toMatchObject({ conversation_id: 'q1', reply_to_message_id: 'm1' });
});

it('a30_changing_answering_sends_to_the_chosen_question_and_drops_another_reply_target', async () => {
  await openThread('q3');
  fireEvent.click(within(screen.getByTestId('message-m5')).getByRole('button', { name: 'Reply' }));
  fireEvent.change(screen.getByRole('combobox', { name: 'Answering' }), { target: { value: 'q2' } });
  expect(screen.queryByRole('button', { name: 'Cancel reply' })).not.toBeInTheDocument();
  send('More on that');
  await waitFor(() => expect(conversationReply).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationReply).mock.calls[0][0]).toMatchObject({ conversation_id: 'q2', reply_to_message_id: null });
});

it('a30_opening_the_thread_marks_every_question_read', async () => {
  await openThread();
  await waitFor(() => expect(conversationMarkRead).toHaveBeenCalledTimes(3));
  expect(vi.mocked(conversationMarkRead).mock.calls.map(c => c[0]).sort()).toEqual(['q1', 'q2', 'q3']);
});

it('a30_a_single_question_has_no_answering_choice', async () => {
  vi.mocked(conversationThread).mockResolvedValue(thread([Q3]));
  render(<ConversationView id="q3" onBack={() => undefined} />);
  await screen.findByTestId('question-q3');
  expect(screen.queryByRole('combobox', { name: 'Answering' })).not.toBeInTheDocument();
});
