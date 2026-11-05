import { useEffect } from 'react';
import { isTyping } from '@/lib/keyboard';
import { redo, undo } from './useHistory';

export function useHistoryKeys(sessionId?: number) {
  useEffect(() => {
    if (sessionId === undefined) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.defaultPrevented || isTyping(event.target) || event.altKey || !(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== 'z') return;
      event.preventDefault();
      void (event.shiftKey ? redo(sessionId) : undo(sessionId));
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [sessionId]);
}
