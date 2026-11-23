import { useCallback, useEffect, useRef, useState } from 'react';
import { ArrowLeft } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import {
  conversationAttachable,
  conversationDiscard,
  conversationMarkRead,
  conversationRetry,
  conversationThread,
  type ConversationAttachable,
  type ConversationMessage,
  type ConversationThread,
  type ConversationView as View,
  type PendingReply,
} from '@/services/conversations';
import { ServiceError } from '@/services/sessions';
import { Composer, replyExcerpt, type ReplyTarget } from './Composer';
import { MessageBubble, PendingBubble } from './MessageBubble';
import { useConversationsChanged, useConversationsContext } from './useConversations';

/**
 * A30/A31 (09B §8): the question a reply goes to when the photographer has not picked a message to reply to: the newest
 * unresolved question, else the newest (`questions` are oldest first). It is never shown.
 */
export function defaultQuestion(questions: View[]): View | undefined {
  const open = questions.filter(q => q.status !== 'resolved');
  return open[open.length - 1] ?? questions[questions.length - 1];
}

/** One line of the chat: a confirmed message or a reply not yet confirmed, with the question it belongs to. */
export type ChatEntry =
  | { kind: 'message'; questionId: string; message: ConversationMessage }
  | { kind: 'pending'; questionId: string; pending: PendingReply };

/**
 * A31 (09B §7 Thread): every question's messages and unconfirmed replies as one chat, ordered by time. Equal times keep
 * the questions' order (oldest question first), then the message order within it, confirmed before unconfirmed.
 */
export function chatEntries(questions: View[]): ChatEntry[] {
  const entries: { entry: ChatEntry; at: number; order: number }[] = [];
  questions.forEach(q => {
    q.messages.forEach(message => entries.push({ entry: { kind: 'message', questionId: q.id, message }, at: message.created_at, order: entries.length }));
  });
  questions.forEach(q => {
    q.pending.forEach(pending => entries.push({ entry: { kind: 'pending', questionId: q.id, pending }, at: pending.created_at, order: entries.length }));
  });
  return entries.sort((a, b) => a.at - b.at || a.order - b.order).map(e => e.entry);
}

/** The product's thread, re-read on `conversations_changed`; `conversations_off` (the copy is hidden) leaves it. */
function useThread(id: string, onOff: () => void) {
  const [thread, setThread] = useState<ConversationThread | null>(null);
  const latest = useRef(0);
  const off = useRef(onOff);
  off.current = onOff;
  const load = useCallback(() => {
    const mine = ++latest.current;
    Promise.resolve()
      .then(() => conversationThread(id))
      .then(next => {
        if (mine === latest.current) setThread(next);
      })
      .catch(error => {
        if (mine === latest.current && error instanceof ServiceError && error.code === 'conversations_off') off.current();
      });
  }, [id]);
  useEffect(load, [load]);
  useConversationsChanged(load);
  return { thread, reload: load };
}

/**
 * 09B §6 (A30: every question of the product): a question counts as read while the thread is on screen and the window
 * is focused: marked when the thread opens and whenever a re-read brings a newer message; when the window is not
 * focused, at the next `focus`.
 */
function useReadMarks(questions: View[]) {
  const marked = useRef(new Map<string, number>());
  useEffect(() => {
    if (questions.length === 0) return;
    const mark = () => {
      for (const q of questions) {
        const newest = q.messages.reduce((max, m) => Math.max(max, m.created_at), 0);
        const before = marked.current.get(q.id);
        if (before !== undefined && newest <= before) continue;
        marked.current.set(q.id, newest);
        Promise.resolve()
          .then(() => conversationMarkRead(q.id))
          .catch(() => undefined);
      }
    };
    if (document.hasFocus()) mark();
    window.addEventListener('focus', mark);
    return () => window.removeEventListener('focus', mark);
  }, [questions]);
}

/** The photos a reply may attach (the product's, through the targeted question), read again with every re-read. */
function useAttachable(id: string | undefined, thread: ConversationThread | null): ConversationAttachable | null {
  const [attachable, setAttachable] = useState<ConversationAttachable | null>(null);
  const latest = useRef(0);
  useEffect(() => {
    if (!thread || id === undefined) return;
    const mine = ++latest.current;
    Promise.resolve()
      .then(() => conversationAttachable(id))
      .then(next => {
        if (mine === latest.current) setAttachable(next);
      })
      .catch(() => undefined);
  }, [id, thread]);
  return attachable;
}

function failureMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/**
 * The product line: "Not on this Mac". C4 (09B §7.1): "Open session" when the product's session is not the one on
 * screen; it opens that session as a visit (the panel stays open on this thread).
 */
function ProductLine({ sessionId }: { sessionId: number | null }) {
  const state = useConversationsContext();
  const canOpen = state !== null && sessionId !== null && sessionId !== state.sessionId;
  if (sessionId !== null && !canOpen) return null;
  return (
    <div className="conversation-product-line">
      {sessionId === null && <span className="conversation-elsewhere">Not on this Mac</span>}
      {canOpen && (
        <Button variant="outline" size="sm" className="conversation-open-session" onClick={() => state.openSession(sessionId)}>
          Open session
        </Button>
      )}
    </div>
  );
}

/**
 * A31 (09B §7 Thread; owner 2026-11-23): the product is the container and the thread is a plain chat. Every message of
 * every question in time order, the pricer's dark on the left, the photographer's light on the right, each with who sent
 * it and when; no question dividers, status lines or answering controls. Reply is offered on the message itself (on
 * hover or focus, and on right-click) and sends to that message's question; otherwise a reply goes to the newest
 * unresolved question.
 */
export function ConversationView({ id, onBack }: { id: string; onBack: () => void }) {
  const { thread, reload } = useThread(id, onBack);
  const questions = thread?.questions ?? [];
  const [replyTarget, setReplyTarget] = useState<(ReplyTarget & { questionId: string }) | null>(null);
  const target = questions.find(q => q.id === replyTarget?.questionId) ?? defaultQuestion(questions);
  const attachable = useAttachable(target?.id, thread);
  useReadMarks(questions);
  const entries = chatEntries(questions);

  // A toast's Open: the first message of the question it names is brought into view once the thread is on screen.
  const scrolled = useRef(false);
  const threadRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (scrolled.current || questions.length < 2) return;
    scrolled.current = true;
    const lines = threadRef.current?.querySelectorAll<HTMLElement>('[data-question-id]') ?? [];
    const line = Array.from(lines).find(l => l.dataset.questionId === id);
    line?.scrollIntoView?.({ block: 'start' });
  }, [questions, id]);

  const act = (work: () => Promise<unknown>) => {
    Promise.resolve()
      .then(work)
      .catch(error => toast(failureMessage(error)))
      .finally(reload);
  };

  return (
    <div className="conversation-view" data-testid="conversation-view">
      <div className="conversation-view-head">
        <Button variant="ghost" size="icon" aria-label="Back to conversations" onClick={onBack}>
          <ArrowLeft />
        </Button>
        {thread && (
          <div className="conversation-view-product">
            <span className="conversation-view-label">Questions</span>
            {thread.sku && <strong>{thread.sku}</strong>}
            <span className="conversation-title">{thread.product_title}</span>
          </div>
        )}
      </div>
      {thread && <ProductLine sessionId={thread.session_id} />}
      <div className="conversation-thread" ref={threadRef}>
        {entries.map(entry =>
          entry.kind === 'message' ? (
            <MessageBubble
              key={entry.message.id}
              message={entry.message}
              questionId={entry.questionId}
              sessionId={thread?.session_id ?? null}
              onReply={() =>
                setReplyTarget({
                  questionId: entry.questionId,
                  message_id: entry.message.id,
                  author_name: entry.message.author.name,
                  excerpt: replyExcerpt(entry.message.text),
                })
              }
            />
          ) : (
            <PendingBubble
              key={entry.pending.client_mutation_id}
              pending={entry.pending}
              actions={
                <>
                  <button
                    type="button"
                    className="message-action"
                    onClick={() => act(() => conversationRetry(entry.pending.client_mutation_id))}
                  >
                    Retry
                  </button>
                  <button
                    type="button"
                    className="message-action"
                    onClick={() => act(() => conversationDiscard(entry.pending.client_mutation_id))}
                  >
                    Remove
                  </button>
                </>
              }
            />
          ),
        )}
      </div>
      {target && (
        <Composer
          view={target}
          attachable={attachable}
          replyTarget={replyTarget}
          onClearReply={() => setReplyTarget(null)}
          onSent={reload}
        />
      )}
    </div>
  );
}
