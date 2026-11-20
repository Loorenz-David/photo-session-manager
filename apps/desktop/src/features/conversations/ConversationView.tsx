import { useCallback, useEffect, useRef, useState } from 'react';
import { ArrowLeft } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import {
  conversationAttachable,
  conversationDiscard,
  conversationGet,
  conversationMarkAnswered,
  conversationMarkRead,
  conversationRetry,
  type ConversationAttachable,
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

/** The conversation, re-read on `conversations_changed`; `conversations_off` (the copy is hidden) leaves it. */
function useView(id: string, onOff: () => void) {
  const [view, setView] = useState<View | null>(null);
  const latest = useRef(0);
  const off = useRef(onOff);
  off.current = onOff;
  const load = useCallback(() => {
    const mine = ++latest.current;
    Promise.resolve()
      .then(() => conversationGet(id))
      .then(next => {
        if (mine === latest.current) setView(next);
      })
      .catch(error => {
        if (mine === latest.current && error instanceof ServiceError && error.code === 'conversations_off') off.current();
      });
  }, [id]);
  useEffect(load, [load]);
  useConversationsChanged(load);
  return { view, reload: load };
}

/**
 * 09B §6: the conversation counts as read while it is on screen and the window is focused: marked when it opens and
 * whenever a re-read brings a newer message; when the window is not focused, at the next `focus`.
 */
function useReadMark(id: string, view: View | null) {
  const marked = useRef<number | null>(null);
  useEffect(() => {
    marked.current = null;
  }, [id]);
  useEffect(() => {
    if (!view) return;
    const newest = view.messages.reduce((max, m) => Math.max(max, m.created_at), 0);
    const mark = () => {
      if (marked.current !== null && newest <= marked.current) return;
      marked.current = newest;
      Promise.resolve()
        .then(() => conversationMarkRead(id))
        .catch(() => undefined);
    };
    if (document.hasFocus()) mark();
    window.addEventListener('focus', mark);
    return () => window.removeEventListener('focus', mark);
  }, [id, view]);
}

/** The photos a reply may attach, read when the view opens and again with every re-read of the view. */
function useAttachable(id: string, view: View | null): ConversationAttachable | null {
  const [attachable, setAttachable] = useState<ConversationAttachable | null>(null);
  const latest = useRef(0);
  useEffect(() => {
    if (!view) return;
    const mine = ++latest.current;
    Promise.resolve()
      .then(() => conversationAttachable(id))
      .then(next => {
        if (mine === latest.current) setAttachable(next);
      })
      .catch(() => undefined);
  }, [id, view]);
  return attachable;
}

function failureMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/**
 * The header line: status dot, topic, "· Blocking", the status in words, "Not on this Mac"; Mark answered on an open
 * conversation. C4 (09B §7.1): "Open session" when the conversation's session is not the one on screen; it opens that
 * session as a visit (the panel stays open on this conversation).
 */
function HeaderLine({ view, onMarkAnswered }: { view: View; onMarkAnswered: () => void }) {
  const state = useConversationsContext();
  const sessionId = view.session_id;
  const canOpen = state !== null && sessionId !== null && sessionId !== state.sessionId;
  return (
    <div className="conversation-header-line">
      <span className="conversation-status-dot" data-status={view.status} aria-hidden="true" />
      <span className="conversation-header-topic">{view.topic}</span>
      {view.blocking && <span className="conversation-header-blocking">· Blocking</span>}
      <span className="conversation-header-status">{statusWords(view)}</span>
      {view.session_id === null && <span className="conversation-elsewhere">Not on this Mac</span>}
      {canOpen && (
        <Button variant="outline" size="sm" className="conversation-open-session" onClick={() => state.openSession(sessionId)}>
          Open session
        </Button>
      )}
      {view.status === 'open' && (
        <Button variant="outline" size="sm" className="conversation-mark-answered" onClick={onMarkAnswered}>
          Mark answered
        </Button>
      )}
    </div>
  );
}

/**
 * C3 (09B §7 Conversation): the conversation as in Pricing: the header line, the messages in order (the pricer's dark
 * on the left, the photographer's light on the right), then the replies not yet confirmed.
 */
export function ConversationView({ id, onBack }: { id: string; onBack: () => void }) {
  const { view, reload } = useView(id, onBack);
  const attachable = useAttachable(id, view);
  const [replyTarget, setReplyTarget] = useState<ReplyTarget | null>(null);
  useReadMark(id, view);

  const act = (work: () => Promise<unknown>) => {
    Promise.resolve()
      .then(work)
      .catch(error => toast(failureMessage(error)))
      .finally(reload);
  };
  const markAnswered = () => {
    Promise.resolve()
      .then(() => conversationMarkAnswered(id))
      .catch(error => {
        if (error instanceof ServiceError && error.code === 'conversation_resolved') {
          toast('Pricing has already resolved this');
        } else {
          toast(failureMessage(error));
        }
      })
      .finally(reload);
  };

  return (
    <div className="conversation-view" data-testid="conversation-view">
      <div className="conversation-view-head">
        <Button variant="ghost" size="icon" aria-label="Back to conversations" onClick={onBack}>
          <ArrowLeft />
        </Button>
        {view && (
          <div className="conversation-view-product">
            {view.sku && <strong>{view.sku}</strong>}
            <span className="conversation-title">{view.product_title}</span>
          </div>
        )}
      </div>
      {view && <HeaderLine view={view} onMarkAnswered={markAnswered} />}
      <div className="conversation-thread">
        {view?.messages.map(m => (
          <MessageBubble
            key={m.id}
            message={m}
            sessionId={view.session_id}
            onReply={() => setReplyTarget({ message_id: m.id, author_name: m.author.name, excerpt: replyExcerpt(m.text) })}
          />
        ))}
        {view?.pending.map(p => (
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
      </div>
      {view && (
        <Composer
          view={view}
          attachable={attachable}
          replyTarget={replyTarget}
          onClearReply={() => setReplyTarget(null)}
          onSent={reload}
        />
      )}
    </div>
  );
}
