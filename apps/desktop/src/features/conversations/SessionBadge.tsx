import { useCallback, useEffect, useRef, useState } from 'react';
import { conversationsSessionBadges, type ConversationSessionBadge } from '@/services/conversations';
import { useConversationsChanged } from './useConversations';

/** C2 (09B §6): a session's conversations badge: "N open", or amber "Pricing is waiting" when one is blocking. */
export function SessionBadge({ badge }: { badge?: ConversationSessionBadge }) {
  if (!badge) return null;
  return badge.waiting ? (
    <span className="pill conversation-waiting">Pricing is waiting</span>
  ) : (
    <span className="pill conversation-badge">{badge.open} open</span>
  );
}

/** The badges of every session, by session id, re-read on `conversations_changed`. */
export function useSessionBadges(): Map<number, ConversationSessionBadge> {
  const [badges, setBadges] = useState<Map<number, ConversationSessionBadge>>(() => new Map());
  const latest = useRef(0);
  const load = useCallback(() => {
    const mine = ++latest.current;
    Promise.resolve()
      .then(() => conversationsSessionBadges())
      .then(next => {
        if (mine === latest.current) setBadges(new Map(next.map(badge => [badge.session_id, badge])));
      })
      .catch(() => undefined);
  }, []);
  useEffect(load, [load]);
  useConversationsChanged(load);
  return badges;
}
