import type { ConversationMessage } from '@/services/conversations';
import { useConversationsContext } from './useConversations';

type Quote = NonNullable<ConversationMessage['quote']>;

/**
 * C3 (09B §7; CHAT_UX §3): the quoted product field of a question: an accent bar, a contrasting fill, the field chip
 * (the field's label in uppercase) and the quoted text in “…”. C4 (quote chip jump): the chip is a button that opens
 * the form at that field iff the backend found the field in this Mac's form (`jump_key`) and the conversation's
 * session (`sessionId`) is the session on screen; otherwise it is plain text.
 */
export function QuoteBlock({ quote, sessionId = null }: { quote: Quote; sessionId?: number | null }) {
  const state = useConversationsContext();
  const label = quote.field_label.toUpperCase();
  const key = quote.jump_key ?? null;
  const jumps = key !== null && state !== null && sessionId !== null && sessionId === state.sessionId;
  return (
    <div className="quote-block" data-testid="quote-block">
      {jumps ? (
        <button type="button" className="quote-chip quote-chip-link" title="Open the form at this field" onClick={() => state.openField(key)}>
          {label}
        </button>
      ) : (
        <span className="quote-chip">{label}</span>
      )}
      <p className="quote-text">{`“${quote.text}”`}</p>
    </div>
  );
}
