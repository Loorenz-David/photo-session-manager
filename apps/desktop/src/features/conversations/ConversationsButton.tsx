import { MessageCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useConversationsContext } from './useConversations';

/**
 * C2 (09B §6): the header's chat icon on every page. The count = conversations with news (that session's inside a
 * session, all of them elsewhere); a small dot inside a session when other sessions have news.
 */
export function ConversationsButton() {
  const conversations = useConversationsContext();
  if (!conversations) return null;
  const { counts, sessionId, openPanel } = conversations;
  return (
    <Button variant="ghost" size="icon" aria-label="Conversations" className="relative" onClick={() => openPanel()}>
      <MessageCircle />
      {counts.here > 0 && <span className="bell-count conversations-count">{counts.here}</span>}
      {sessionId !== undefined && counts.elsewhere > 0 && (
        <span className="conversations-dot" data-testid="conversations-dot" title="New in other sessions" />
      )}
    </Button>
  );
}
