import { useEffect, useRef } from 'react';
import { toast } from 'sonner';
import { onConversationsChanged } from '@/services/events';
import { useConversationsContext } from './useConversations';

/** A toast's life while the window is focused. */
export const TOAST_MS = 8_000;
/** After the window gets the focus back (a banner click brings it forward): 10 s more. */
export const AFTER_FOCUS_MS = 10_000;

type Shown = { id: string; text: string; conversationId?: string };

/**
 * C2 (09B §9; master plan §6): on `conversations_changed`, one toast per line ("New question · <SKU> — <topic>",
 * "New reply · <SKU>") with **Open**, and "and N more". Shown while the window is in the background, a toast stays
 * until the window's next focus and 10 s after (macOS does not tell which banner was clicked).
 */
export function ConversationToasts() {
  const conversations = useConversationsContext();
  const openPanel = useRef(conversations?.openPanel);
  openPanel.current = conversations?.openPanel;
  useEffect(() => {
    let alive = true;
    let off: (() => void) | undefined;
    let waiting: Shown[] = [];
    const show = (shown: Shown, duration: number) => {
      const conversationId = shown.conversationId;
      toast(shown.text, {
        id: shown.id,
        duration,
        action: conversationId === undefined ? undefined : { label: 'Open', onClick: () => openPanel.current?.(conversationId) },
      });
    };
    const onFocus = () => {
      const due = waiting;
      waiting = [];
      due.forEach(shown => show(shown, AFTER_FOCUS_MS));
    };
    window.addEventListener('focus', onFocus);
    Promise.resolve()
      .then(() =>
        onConversationsChanged(event => {
          const all: Shown[] = event.toasts.map(line => ({ id: `conversation-${line.conversation_id}`, text: line.text, conversationId: line.conversation_id }));
          if (event.more > 0) all.push({ id: 'conversation-more', text: `and ${event.more} more` });
          if (all.length === 0) return;
          if (document.hasFocus()) {
            all.forEach(shown => show(shown, TOAST_MS));
          } else {
            waiting = [...waiting.filter(old => !all.some(shown => shown.id === old.id)), ...all];
            all.forEach(shown => show(shown, Infinity));
          }
        }),
      )
      .then(unlisten => (alive ? (off = unlisten) : unlisten()))
      .catch(() => undefined);
    return () => {
      alive = false;
      off?.();
      window.removeEventListener('focus', onFocus);
    };
  }, []);
  return null;
}
