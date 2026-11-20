import type { ConversationsConnection } from '@/services/conversations';

/**
 * C1 (09B §4, master plan §4 Words): the connection in words; `null` when there is nothing to say (no key). One place for
 * these words: Settings → Pricing and the conversations panel (A28) both show them.
 */
export function connectionLine(connection: ConversationsConnection): string | null {
  const shop = connection.shop ?? '';
  switch (connection.state) {
    case 'ready': return `Connected to ${connection.store_name ?? ''} (${shop})`;
    case 'store_unmatched': return connection.visible_stores.length > 0
      ? `This Pricing key isn't tied to ${shop}. It can see: ${connection.visible_stores.join(', ')}.`
      : `This Pricing key isn't tied to ${shop}. Press Test connection.`;
    case 'no_shop': return 'Connect Shopify first; conversations are matched to its products.';
    case 'refused': return 'Pricing refused the key: check Settings → Pricing';
    case 'store_gone': return `This key is no longer tied to ${shop}`;
    default: return null;
  }
}

/** The panel's line when there is no key yet (09B §4). */
export const CONNECT_PRICING = 'Connect Pricing in Settings → Pricing';

/** A clock time (HH:MM:SS, local). */
export function clock(ms: number): string {
  const at = new Date(ms);
  return [at.getHours(), at.getMinutes(), at.getSeconds()].map(n => String(n).padStart(2, '0')).join(':');
}
