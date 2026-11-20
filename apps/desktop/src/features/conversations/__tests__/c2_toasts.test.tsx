import '@testing-library/jest-dom/vitest';
import { act, cleanup, render, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ConversationsChanged } from '@/services/events';

const delivered = vi.hoisted(() => ({ changed: [] as ((event: ConversationsChanged) => void)[] }));

vi.mock('@/services/events', () => ({
  onConversationsChanged: vi.fn(async (handler: (event: ConversationsChanged) => void) => {
    delivered.changed.push(handler);
    return () => undefined;
  }),
}));
vi.mock('sonner', () => ({ toast: vi.fn() }));

import { toast } from 'sonner';
import { ConversationToasts } from '../ConversationToasts';
import { ConversationsContext, type ConversationsState } from '../useConversations';

type ToastOptions = { id: string; duration: number; action?: { label: string; onClick: () => void } };

function calls(): [string, ToastOptions][] {
  return vi.mocked(toast).mock.calls as unknown as [string, ToastOptions][];
}

async function mount(focused = true) {
  vi.spyOn(document, 'hasFocus').mockReturnValue(focused);
  const openPanel = vi.fn();
  const state = { openPanel } as unknown as ConversationsState;
  render(
    <ConversationsContext.Provider value={state}>
      <ConversationToasts />
    </ConversationsContext.Provider>,
  );
  await waitFor(() => expect(delivered.changed.length).toBe(1));
  return openPanel;
}

function deliver(event: ConversationsChanged) {
  act(() => delivered.changed.forEach(handler => handler(event)));
}

const QUESTION = { conversation_id: 'c1', text: 'New question · BEYO-1880 — Condition' };

beforeEach(() => {
  delivered.changed = [];
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.mocked(toast).mockClear();
});

it('c2_c16a_a_notice_shows_one_toast_with_open', async () => {
  await mount();
  deliver({ toasts: [QUESTION], more: 0 });
  expect(calls()).toHaveLength(1);
  const [text, options] = calls()[0];
  expect(text).toBe('New question · BEYO-1880 — Condition');
  expect(options.id).toBe('conversation-c1');
  expect(options.duration).toBe(8000);
  expect(options.action?.label).toBe('Open');
});

it('c2_c16b_open_opens_the_panel_on_that_conversation', async () => {
  const openPanel = await mount();
  deliver({ toasts: [QUESTION], more: 0 });
  calls()[0][1].action!.onClick();
  expect(openPanel).toHaveBeenCalledTimes(1);
  expect(openPanel).toHaveBeenCalledWith('c1');
});

it('c2_c16c_three_toasts_and_the_rest_counted', async () => {
  await mount();
  deliver({
    toasts: [
      { conversation_id: 'a', text: 'New question · BEYO-1 — Condition' },
      { conversation_id: 'b', text: 'New reply · BEYO-2' },
      { conversation_id: 'c', text: 'New reply · BEYO-3' },
    ],
    more: 2,
  });
  expect(calls().map(([text]) => text)).toEqual([
    'New question · BEYO-1 — Condition',
    'New reply · BEYO-2',
    'New reply · BEYO-3',
    'and 2 more',
  ]);
});

it('c2_c16d_an_event_without_toasts_shows_nothing', async () => {
  await mount();
  deliver({ toasts: [], more: 0 });
  expect(toast).not.toHaveBeenCalled();
});

it('c2_c16e_in_the_background_a_toast_waits_for_focus_then_10_s', async () => {
  await mount(false);
  deliver({ toasts: [QUESTION], more: 0 });
  expect(calls()).toHaveLength(1);
  expect(calls()[0][1].duration).toBe(Infinity);
  act(() => {
    window.dispatchEvent(new Event('focus'));
  });
  expect(calls()).toHaveLength(2);
  const [text, options] = calls()[1];
  expect(text).toBe('New question · BEYO-1880 — Condition');
  expect(options.id).toBe('conversation-c1');
  expect(options.duration).toBe(10_000);
  expect(options.action?.label).toBe('Open');
});
