import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ConversationsChanged } from '@/services/events';
import type { ConversationListRow, ConversationMessage, ConversationView as View, PendingReply } from '@/services/conversations';

const delivered = vi.hoisted(() => ({ changed: [] as ((event: ConversationsChanged) => void)[] }));

vi.mock('@/services/conversations', () => ({
  conversationsCounts: vi.fn(async () => ({ here: 0, elsewhere: 0 })),
  conversationsList: vi.fn(),
  conversationsSettingsGet: vi.fn(async () => ({ sound: true })),
  conversationsSettingsSave: vi.fn(),
  conversationGet: vi.fn(),
  // A30: one question per product unless a test says otherwise: the thread wraps conversationGet.
  conversationThread: vi.fn(async (id: string) => {
    const view = await (await import("@/services/conversations")).conversationGet(id);
    return { product_id: "p", sku: view.sku, product_title: view.product_title, session_id: view.session_id, questions: [view] };
  }),
  conversationMarkRead: vi.fn(async () => undefined),
  conversationAttachable: vi.fn(async () => ({ photos: [], reason: 'nothing_published' })),
  conversationReply: vi.fn(),
  conversationRetry: vi.fn(),
  conversationDiscard: vi.fn(),
  conversationMarkAnswered: vi.fn(),
}));
vi.mock('@/services/events', () => ({
  onConversationsChanged: vi.fn(async (handler: (event: ConversationsChanged) => void) => {
    delivered.changed.push(handler);
    return () => undefined;
  }),
}));

import { activityLabel } from '@/lib/format';
import { conversationGet, conversationMarkRead, conversationsList } from '@/services/conversations';
import { ConversationsContext, useConversations } from '../useConversations';
import { ConversationsButton } from '../ConversationsButton';
import { ConversationsPanel } from '../ConversationsPanel';
import { ConversationView } from '../ConversationView';
import { MessageBubble } from '../MessageBubble';
import { QuoteBlock } from '../QuoteBlock';

/** 2026-11-20T12:00:00Z. */
const NOW = 1_795_176_000_000;

function message(extra: Partial<ConversationMessage> = {}): ConversationMessage {
  return {
    id: 'm1',
    author: { kind: 'asker', name: 'Elin Lindqvist' },
    text: 'Is the glass original?',
    quote: null,
    reply_to: null,
    attachments: [],
    created_at: NOW - 30 * 60_000,
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
    state: 'sending',
    failure: null,
    created_at: NOW,
    ...extra,
  };
}

function row(): ConversationListRow {
  return {
    id: 'c1',
    product_id: 'p1',
    questions: 1,
    sku: 'BEYO-1880',
    product_title: 'Vintage Brass Wall Mirror, Round',
    topic: 'Condition',
    status: 'open',
    blocking: true,
    waiting: true,
    last_line: 'Is the glass original?',
    last_activity_at: NOW - 60_000,
    has_news: true,
    session_id: 7,
  };
}

function Harness() {
  const conversations = useConversations(undefined);
  return (
    <ConversationsContext.Provider value={conversations}>
      <ConversationsButton />
      <ConversationsPanel />
    </ConversationsContext.Provider>
  );
}

async function shown(v: View) {
  vi.mocked(conversationGet).mockResolvedValue(v);
  render(<ConversationView id={v.id} onBack={() => undefined} />);
  return screen.findByTestId('conversation-view').then(async node => {
    await within(node).findByRole('textbox', { name: 'Reply' });
    return node;
  });
}

function deliver() {
  act(() => delivered.changed.forEach(handler => handler({ toasts: [], more: 0 })));
}

beforeEach(() => {
  delivered.changed = [];
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(NOW);
  vi.spyOn(document, 'hasFocus').mockReturnValue(true);
  vi.mocked(conversationsList).mockResolvedValue({ state: 'ready', problem: null, rows: [row()], news_elsewhere: 0 });
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.mocked(conversationGet).mockReset();
  vi.mocked(conversationMarkRead).mockClear();
});

// ---------- C3·C9 opening, the header line ----------

it('c3_c9a_a_row_opens_the_conversation_and_back_returns_to_the_list', async () => {
  vi.mocked(conversationGet).mockResolvedValue(view());
  render(<Harness />);
  fireEvent.click(screen.getByRole('button', { name: 'Conversations' }));
  const panel = await screen.findByRole('dialog', { name: 'Conversations' });
  fireEvent.click(await within(panel).findByTestId('conversation-row-c1'));
  await waitFor(() => expect(vi.mocked(conversationGet)).toHaveBeenCalledWith('c1'));
  expect(within(panel).getByTestId('conversation-view')).toBeInTheDocument();
  expect(within(panel).queryByTestId('conversation-row-c1')).toBeNull();
  fireEvent.click(within(panel).getByRole('button', { name: 'Back to conversations' }));
  expect(within(panel).getByTestId('conversation-row-c1')).toBeInTheDocument();
  expect(within(panel).queryByTestId('conversation-view')).toBeNull();
});

it('a31_c9b_no_topic_blocking_or_status_line_in_the_thread', async () => {
  const node = await shown(view({ topic: 'Condition', blocking: true, status: 'open', session_id: 7 }));
  expect(within(node).queryByText('Condition')).toBeNull();
  expect(within(node).queryByText('· Blocking')).toBeNull();
  expect(within(node).queryByText('Open')).toBeNull();
  expect(node.querySelector('.conversation-status-dot')).toBeNull();
  expect(within(node).queryByText('Not on this Mac')).toBeNull();
});

it('a31_c9c_an_answered_question_not_on_this_mac_shows_no_status_but_the_product_line', async () => {
  const node = await shown(view({ blocking: false, status: 'answered', session_id: null }));
  expect(within(node).queryByText('Answered')).toBeNull();
  expect(within(node).getByText('Not on this Mac')).toBeInTheDocument();
});

// ---------- C3·C10 bubbles ----------

it('c3_c10a_the_pricers_message_is_received_with_name_and_time', () => {
  const m = message({ author: { kind: 'asker', name: 'Elin Lindqvist' } });
  render(<MessageBubble message={m} />);
  const bubble = screen.getByTestId('message-m1');
  expect(bubble).toHaveAttribute('data-side', 'received');
  expect(within(bubble).getByText('Elin Lindqvist')).toBeInTheDocument();
  expect(within(bubble).getByText(activityLabel(m.created_at, NOW))).toBeInTheDocument();
  expect(activityLabel(m.created_at, NOW)).toBe('30 min ago');
});

it('c3_c10b_my_message_is_sent_and_labelled_you', () => {
  render(<MessageBubble message={message({ author: { kind: 'me', name: 'Photographer' } })} />);
  const bubble = screen.getByTestId('message-m1');
  expect(bubble).toHaveAttribute('data-side', 'sent');
  expect(within(bubble).getByText('You')).toBeInTheDocument();
  expect(within(bubble).queryByText('Photographer')).toBeNull();
});

it('c3_c10c_another_accounts_message_is_received_with_its_name', () => {
  render(<MessageBubble message={message({ author: { kind: 'other', name: 'Operations' } })} />);
  const bubble = screen.getByTestId('message-m1');
  expect(bubble).toHaveAttribute('data-side', 'received');
  expect(within(bubble).getByText('Operations')).toBeInTheDocument();
});

it('c3_c10d_edited_shows_after_the_time_only_when_set', () => {
  const { rerender } = render(<MessageBubble message={message({ edited_at: NOW - 60_000 })} />);
  const time = screen.getByText('30 min ago');
  const edited = screen.getByText('· edited');
  expect(time.compareDocumentPosition(edited) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  rerender(<MessageBubble message={message({ edited_at: null })} />);
  expect(screen.queryByText('· edited')).toBeNull();
});

// ---------- C3·C11 the quote ----------

it('c3_c11a_the_quote_block_shows_the_field_chip_and_the_quoted_text', () => {
  render(<QuoteBlock quote={{ field: 'condition', field_label: 'Condition', text: 'Possible desilvering at the lower edge.' }} />);
  const block = screen.getByTestId('quote-block');
  const chip = within(block).getByText('CONDITION');
  expect(chip.closest('button')).toBeNull();
  expect(within(block).queryByRole('button')).toBeNull();
  expect(within(block).getByText('“Possible desilvering at the lower edge.”')).toBeInTheDocument();
});

it('c3_c11b_a_message_without_quote_has_no_quote_block', () => {
  render(<MessageBubble message={message({ quote: null })} />);
  expect(screen.queryByTestId('quote-block')).toBeNull();
});

// ---------- C3·C12 photos ----------

const TWO_PHOTOS = [
  { media_id: 'a', thumb_url: 'https://cdn.example/a-thumb.jpg', url: 'https://cdn.example/a.jpg' },
  { media_id: 'b', thumb_url: 'https://cdn.example/b-thumb.jpg', url: 'https://cdn.example/b.jpg' },
];

it('c3_c12a_attachments_are_thumbnails_above_the_text', () => {
  render(<MessageBubble message={message({ attachments: TWO_PHOTOS })} />);
  const images = screen.getAllByRole('img', { name: /Photo \d/ });
  expect(images.map(i => i.getAttribute('src'))).toEqual(['https://cdn.example/a-thumb.jpg', 'https://cdn.example/b-thumb.jpg']);
  const text = screen.getByText('Is the glass original?');
  expect(images[1].compareDocumentPosition(text) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
});

it('c3_c12b_a_thumbnail_opens_the_photo_large', async () => {
  render(<MessageBubble message={message({ attachments: TWO_PHOTOS })} />);
  fireEvent.click(screen.getByRole('button', { name: 'Open photo 1' }));
  const dialog = await screen.findByRole('dialog');
  expect(within(dialog).getByRole('img', { name: 'Photo' })).toHaveAttribute('src', 'https://cdn.example/a.jpg');
});

it('c3_c12c_an_attachment_without_addresses_is_not_recorded', () => {
  render(<MessageBubble message={message({ attachments: [{ media_id: 'x', thumb_url: null, url: null }] })} />);
  expect(screen.getByText('Photo not recorded')).toBeInTheDocument();
  expect(screen.queryByRole('img')).toBeNull();
});

// ---------- C3·C13 the reply line ----------

it('c3_c13a_the_reply_line_shows_above_the_text', () => {
  render(
    <MessageBubble
      message={message({
        author: { kind: 'me', name: 'Photographer' },
        text: 'Yes, original.',
        reply_to: { message_id: 'm0', author_name: 'Elin', excerpt: 'Is the glass original?' },
      })}
    />,
  );
  const line = screen.getByText('↩ Elin: Is the glass original?');
  const text = screen.getByText('Yes, original.');
  expect(line.compareDocumentPosition(text) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
});

// ---------- C3·C14 the read mark ----------

it('c3_c14a_opening_while_focused_marks_read_once', async () => {
  await shown(view());
  await waitFor(() => expect(vi.mocked(conversationMarkRead)).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationMarkRead)).toHaveBeenCalledWith('c1');
});

it('c3_c14b_opening_unfocused_marks_read_at_the_next_focus', async () => {
  vi.mocked(document.hasFocus).mockReturnValue(false);
  await shown(view());
  expect(vi.mocked(conversationMarkRead)).not.toHaveBeenCalled();
  vi.mocked(document.hasFocus).mockReturnValue(true);
  act(() => {
    window.dispatchEvent(new Event('focus'));
  });
  await waitFor(() => expect(vi.mocked(conversationMarkRead)).toHaveBeenCalledTimes(1));
  expect(vi.mocked(conversationMarkRead)).toHaveBeenCalledWith('c1');
});

it('c3_c14c_a_newer_message_while_open_is_shown_and_marked_read_again', async () => {
  const node = await shown(view());
  await waitFor(() => expect(vi.mocked(conversationMarkRead)).toHaveBeenCalledTimes(1));
  vi.mocked(conversationGet).mockResolvedValue(
    view({ messages: [message(), message({ id: 'm2', text: 'And the frame?', created_at: NOW - 60_000 })] }),
  );
  deliver();
  expect(await within(node).findByText('And the frame?')).toBeInTheDocument();
  await waitFor(() => expect(vi.mocked(conversationMarkRead)).toHaveBeenCalledTimes(2));
});

// ---------- C3·C15 resolved, pending, empty ----------

it('a31_c15a_a_resolved_question_shows_no_resolved_wording_and_is_not_dimmed', async () => {
  const node = await shown(view({ status: 'resolved', resolved_by: 'asker' }));
  expect(within(node).queryByText(/Resolved/)).toBeNull();
  expect(node.querySelector('[data-status="resolved"]')).toBeNull();
  expect(within(node).getByTestId('message-m1')).toBeInTheDocument();
});

it('c3_c15c_pending_replies_follow_the_messages_dimmed_on_the_sent_side', async () => {
  const node = await shown(
    view({
      pending: [
        pending({ client_mutation_id: 'u-1', text: 'First.', state: 'sending' }),
        pending({ client_mutation_id: 'u-2', text: 'Second.', state: 'failed', failure: "Couldn't send" }),
      ],
    }),
  );
  const sending = within(node).getByTestId('pending-u-1');
  const failed = within(node).getByTestId('pending-u-2');
  const last = within(node).getByTestId('message-m1');
  expect(sending).toHaveAttribute('data-side', 'sent');
  expect(sending).toHaveClass('message-pending');
  expect(within(sending).getByText('Sending…')).toBeInTheDocument();
  expect(failed).toHaveAttribute('data-side', 'sent');
  expect(failed).toHaveClass('message-pending');
  expect(within(failed).getByText("Couldn't send")).toBeInTheDocument();
  expect(last.compareDocumentPosition(sending) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  expect(sending.compareDocumentPosition(failed) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
});

it('c3_c15d_no_message_and_no_pending_shows_only_the_product_header', async () => {
  const node = await shown(view({ messages: [], pending: [] }));
  expect(within(node).getByText('Questions')).toBeInTheDocument();
  expect(within(node).queryByTestId(/^message-/)).toBeNull();
  expect(within(node).queryByTestId(/^pending-/)).toBeNull();
});
