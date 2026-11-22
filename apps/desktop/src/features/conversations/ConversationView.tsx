import { useCallback, useEffect, useRef, useState } from 'react';
import { ArrowLeft } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import {
  conversationAttachable,
  conversationDiscard,
  conversationMarkAnswered,
  conversationMarkRead,
  conversationRetry,
  conversationThread,
  type ConversationAttachable,
  type ConversationThread,
  type ConversationView as View,
} from '@/services/conversations';
import { ServiceError } from '@/services/sessions';
import { Composer, replyExcerpt, type ReplyTarget } from './Composer';
import { MessageBubble, PendingBubble } from './MessageBubble';
import { useConversationsChanged, useConversationsContext } from './useConversations';

const STATUS_WORDS: Record<string, string> = { open: 'Open', answered: 'Answered' };

/** 09B §7, §8: the status in words; who resolved it when this Mac saw it happen. */
export function statusWords(view: Pick<View, 'status' | 'resolved_by'>): string {
  if (view.status !== 'resolved') return STATUS_WORDS[view.status] ?? view.status;
  if (view.resolved_by === 'asker') return 'Resolved by the pricer';
  if (view.resolved_by === 'activation') return 'Resolved when the product was activated';
  return 'Resolved';
}

/**
 * A30 (09B §7 Thread): the question the composer answers when the photographer has not chosen one: the newest
 * unresolved question, else the newest (`questions` are oldest first).
 */
export function defaultQuestion(questions: View[]): View | undefined {
  const open = questions.filter(q => q.status !== 'resolved');
  return open[open.length - 1] ?? questions[questions.length - 1];
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

/** A question's header line: status dot, topic, "· Blocking", the status in words; Mark answered on an open question. */
function HeaderLine({ view, onMarkAnswered }: { view: View; onMarkAnswered: () => void }) {
  return (
    <div className="conversation-header-line">
      <span className="conversation-status-dot" data-status={view.status} aria-hidden="true" />
      <span className="conversation-header-topic">{view.topic}</span>
      {view.blocking && <span className="conversation-header-blocking">· Blocking</span>}
      <span className="conversation-header-status">{statusWords(view)}</span>
      {view.status === 'open' && (
        <Button variant="outline" size="sm" className="conversation-mark-answered" onClick={onMarkAnswered}>
          Mark answered
        </Button>
      )}
    </div>
  );
}

/** A question's label in the composer's choice: its topic and the first line of its first message. */
function questionLabel(q: View): string {
  const first = q.messages[0];
  const excerpt = first ? replyExcerpt(first.text) : '';
  return excerpt ? `${q.topic} · ${excerpt}` : q.topic;
}

/**
 * A30 (09B §7 Thread; CHAT_UX §1–2): the product is the container. Its questions stacked oldest first, each with its
 * header line, its messages (the pricer's dark on the left, the photographer's light on the right) and its replies not
 * yet confirmed; resolved questions quieter. One composer at the bottom answers one question: the one opened (a row
 * opens the newest unresolved, a toast its own), changed with "Answering" or by pressing Reply on any message.
 */
export function ConversationView({ id, onBack }: { id: string; onBack: () => void }) {
  const { thread, reload } = useThread(id, onBack);
  const questions = thread?.questions ?? [];
  const [chosen, setChosen] = useState(id);
  const [replyTarget, setReplyTarget] = useState<ReplyTarget | null>(null);
  const target = questions.find(q => q.id === chosen) ?? defaultQuestion(questions);
  const attachable = useAttachable(target?.id, thread);
  useReadMarks(questions);

  // A toast's Open: the question it names is brought into view once the thread is on screen.
  const scrolled = useRef(false);
  const threadRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (scrolled.current || questions.length < 2) return;
    scrolled.current = true;
    const sections = threadRef.current?.querySelectorAll<HTMLElement>('[data-question-id]') ?? [];
    const section = Array.from(sections).find(s => s.dataset.questionId === id);
    section?.scrollIntoView?.({ block: 'start' });
  }, [questions, id]);

  const act = (work: () => Promise<unknown>) => {
    Promise.resolve()
      .then(work)
      .catch(error => toast(failureMessage(error)))
      .finally(reload);
  };
  const markAnswered = (questionId: string) => {
    Promise.resolve()
      .then(() => conversationMarkAnswered(questionId))
      .catch(error => {
        if (error instanceof ServiceError && error.code === 'conversation_resolved') {
          toast('Pricing has already resolved this');
        } else {
          toast(failureMessage(error));
        }
      })
      .finally(reload);
  };
  const choose = (questionId: string) => {
    if (questionId !== target?.id) setReplyTarget(null);
    setChosen(questionId);
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
        {questions.map(q => (
          <section
            key={q.id}
            className="conversation-question"
            data-question-id={q.id}
            data-status={q.status}
            data-testid={`question-${q.id}`}
            data-target={questions.length > 1 && q.id === target?.id ? 'true' : undefined}
          >
            <HeaderLine view={q} onMarkAnswered={() => markAnswered(q.id)} />
            {q.messages.map(m => (
              <MessageBubble
                key={m.id}
                message={m}
                sessionId={thread?.session_id ?? q.session_id}
                onReply={() => {
                  setChosen(q.id);
                  setReplyTarget({ message_id: m.id, author_name: m.author.name, excerpt: replyExcerpt(m.text) });
                }}
              />
            ))}
            {q.pending.map(p => (
              <PendingBubble
                key={p.client_mutation_id}
                pending={p}
                actions={
                  <>
                    <button type="button" className="message-action" onClick={() => act(() => conversationRetry(p.client_mutation_id))}>
                      Retry
                    </button>
                    <button type="button" className="message-action" onClick={() => act(() => conversationDiscard(p.client_mutation_id))}>
                      Remove
                    </button>
                  </>
                }
              />
            ))}
          </section>
        ))}
      </div>
      {target && questions.length > 1 && (
        <label className="composer-target">
          <span>Answering</span>
          <select aria-label="Answering" value={target.id} onChange={event => choose(event.target.value)}>
            {questions.map(q => (
              <option key={q.id} value={q.id}>
                {questionLabel(q)}
                {q.status === 'resolved' ? ' (resolved)' : ''}
              </option>
            ))}
          </select>
        </label>
      )}
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
