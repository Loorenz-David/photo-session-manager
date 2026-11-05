import { Redo2, Undo2 } from 'lucide-react';
import { Hint } from '@/components/ui/tooltip';
import { useHistoryState, redo, undo } from './useHistory';

export function HistoryButtons({ sessionId }: { sessionId: number }) {
  const history = useHistoryState();
  const undoLabel = history.sessionId === sessionId ? history.undo : null;
  const redoLabel = history.sessionId === sessionId ? history.redo : null;
  return <>
    <Hint text={{ name: 'Undo', key: '⌘Z', description: undoLabel ?? 'Nothing to undo' }}>
      <button type="button" className="history-button" aria-label="Undo" aria-disabled={undoLabel === null || undefined} onClick={() => { if (undoLabel) void undo(sessionId); }}><Undo2 aria-hidden /></button>
    </Hint>
    <Hint text={{ name: 'Redo', key: '⌘⇧Z', description: redoLabel ?? 'Nothing to redo' }}>
      <button type="button" className="history-button" aria-label="Redo" aria-disabled={redoLabel === null || undefined} onClick={() => { if (redoLabel) void redo(sessionId); }}><Redo2 aria-hidden /></button>
    </Hint>
  </>;
}
