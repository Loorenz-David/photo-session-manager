import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ConversationMessage, ConversationView as View } from '@/services/conversations';

vi.mock('@/services/conversations', () => ({
  conversationsCounts: vi.fn(async () => ({ here: 0, elsewhere: 0 })),
  conversationsList: vi.fn(),
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
vi.mock('@/services/events', () => ({ onConversationsChanged: vi.fn(async () => () => undefined) }));

import { conversationGet } from '@/services/conversations';
import { ConversationsContext, useConversations } from '../useConversations';
import { ConversationView } from '../ConversationView';

const openField = vi.fn();

function message(jumpKey: string | null): ConversationMessage {
  return {
    id: 'm1',
    author: { kind: 'asker', name: 'Elin Lindqvist' },
    text: 'Who designed it?',
    quote: { field: 'mf:custom.designer', field_label: 'Designer', text: 'Unknown', ...(jumpKey === null ? {} : { jump_key: jumpKey }) },
    reply_to: null,
    attachments: [],
    created_at: 1_795_176_000_000,
    edited_at: null,
  };
}

function view(jumpKey: string | null): View {
  return {
    id: 'c1', sku: 'BEYO-7', product_title: 'Mirror', topic: 'Details', blocking: false, status: 'open', resolved_by: null,
    session_id: 7, messages: [message(jumpKey)], pending: [], can_send: true, off_reason: null,
  };
}

/** The conversation view inside the app's conversations state, with `onScreen` the session on screen. */
function Harness({ onScreen }: { onScreen: number }) {
  const conversations = useConversations(onScreen, {
    visit: null, openSession: () => undefined, visitBack: () => undefined, shootHere: () => undefined, openField,
  });
  return (
    <ConversationsContext.Provider value={conversations}>
      <ConversationView id="c1" onBack={() => undefined} />
    </ConversationsContext.Provider>
  );
}

async function chipBlock() {
  return screen.findByTestId('quote-block');
}

beforeEach(() => openField.mockClear());
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('c4_c11a_on_the_conversations_session_the_chip_is_a_button_that_opens_the_field', async () => {
  vi.mocked(conversationGet).mockResolvedValue(view('custom.designer'));
  render(<Harness onScreen={7} />);
  const chip = within(await chipBlock()).getByRole('button', { name: 'DESIGNER' });
  fireEvent.click(chip);
  expect(openField).toHaveBeenCalledTimes(1);
  expect(openField).toHaveBeenCalledWith('custom.designer');
});

it('c4_c11b_on_another_session_the_chip_is_plain_text', async () => {
  vi.mocked(conversationGet).mockResolvedValue(view('custom.designer'));
  render(<Harness onScreen={3} />);
  const block = await chipBlock();
  expect(within(block).getByText('DESIGNER').closest('button')).toBeNull();
  expect(within(block).queryByRole('button')).toBeNull();
});

it('c4_c11c_without_a_jump_key_the_chip_is_plain_text', async () => {
  vi.mocked(conversationGet).mockResolvedValue(view(null));
  render(<Harness onScreen={7} />);
  const block = await chipBlock();
  expect(within(block).getByText('DESIGNER').closest('button')).toBeNull();
  expect(within(block).queryByRole('button')).toBeNull();
});
