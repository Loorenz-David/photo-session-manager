import { activityLabel } from '@/lib/format';
import type { ConversationListRow } from '@/services/conversations';

const STATUS_WORDS: Record<string, string> = { open: 'Open', answered: 'Answered', resolved: 'Resolved' };

/**
 * C2 (09B §7 List): one conversation: SKU and product title, topic, a status dot, amber "Pricing is waiting" for a
 * blocking unresolved one, the last message's first line and its time, an unread dot, "Not on this Mac".
 */
export function ConversationRow({ row, onSelect, now = Date.now() }: { row: ConversationListRow; onSelect: () => void; now?: number }) {
  const status = STATUS_WORDS[row.status] ?? row.status;
  return (
    <button type="button" className="conversation-row" data-testid={`conversation-row-${row.id}`} onClick={onSelect}>
      <span className="conversation-row-top">
        <span className="conversation-status-dot" data-status={row.status} role="img" aria-label={status} />
        <span className="conversation-product">
          {row.sku && <strong className="conversation-sku">{row.sku}</strong>}
          <span className="conversation-title">{row.product_title}</span>
        </span>
        {row.has_news && <span className="conversation-unread" role="img" aria-label="Unread" />}
      </span>
      <span className="conversation-row-meta">
        <span className="conversation-topic">{row.topic}</span>
        {row.waiting && <span className="pill conversation-waiting">Pricing is waiting</span>}
        {row.session_id === null && <span className="conversation-elsewhere">Not on this Mac</span>}
        <time className="conversation-time" dateTime={new Date(row.last_activity_at).toISOString()}>
          {activityLabel(row.last_activity_at, now)}
        </time>
      </span>
      {row.last_line && <span className="conversation-last-line">{row.last_line}</span>}
    </button>
  );
}
