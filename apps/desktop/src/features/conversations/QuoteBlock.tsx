import type { ConversationMessage } from '@/services/conversations';

type Quote = NonNullable<ConversationMessage['quote']>;

/**
 * C3 (09B §7; CHAT_UX §3): the quoted product field of a question: an accent bar, a contrasting fill, the field chip
 * (the field's label in uppercase; plain text here, the jump to the field is C4's) and the quoted text in “…”.
 */
export function QuoteBlock({ quote }: { quote: Quote }) {
  return (
    <div className="quote-block" data-testid="quote-block">
      <span className="quote-chip">{quote.field_label.toUpperCase()}</span>
      <p className="quote-text">{`“${quote.text}”`}</p>
    </div>
  );
}
