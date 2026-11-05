import { useEffect, useSyncExternalStore } from 'react';
import { toast } from 'sonner';
import { ServiceError } from '@/services/sessions';
import { editMenuSet, historyGet, historyRedo, historyUndo, onEditMenu, onHistoryChanged, type HistoryState } from '@/services/history';

type OpenHistory = HistoryState & { sessionId: number | null };
let state: OpenHistory = { sessionId: null, undo: null, redo: null };
const listeners = new Set<() => void>();
let activePhoto: ((id: number) => void) | null = null;

function publish(next: OpenHistory) {
  if (state.sessionId === next.sessionId && state.undo === next.undo && state.redo === next.redo) return;
  state = next;
  listeners.forEach(listener => listener());
  void editMenuSet(next.undo, next.redo).catch(() => undefined);
}

export function useHistoryState(): OpenHistory {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => { listeners.delete(listener); }; }, () => state);
}

export function registerActivePhoto(handler: (id: number) => void): () => void {
  activePhoto = handler;
  return () => { if (activePhoto === handler) activePhoto = null; };
}

async function act(sessionId: number, direction: 'undo' | 'redo') {
  try {
    const outcome = direction === 'undo' ? await historyUndo(sessionId) : await historyRedo(sessionId);
    toast(`${outcome.direction === 'undone' ? 'Undone' : 'Redone'}: ${outcome.label} · ${outcome.direction === 'undone' ? 'Redo ⌘⇧Z' : 'Undo ⌘Z'}`, { id: 'history' });
    toast.dismiss('photo-deleted');
    if (state.sessionId === sessionId && outcome.active_photo_id !== null) activePhoto?.(outcome.active_photo_id);
  } catch (error) {
    const message = error instanceof ServiceError && error.code === 'nothing_to_undo' ? 'Nothing to undo.'
      : error instanceof ServiceError && error.code === 'nothing_to_redo' ? 'Nothing to redo.'
      : (error as Error).message;
    toast(message, { id: 'history' });
  }
}

export const undo = (sessionId: number) => act(sessionId, 'undo');
export const redo = (sessionId: number) => act(sessionId, 'redo');

/** One history subscription for the open route; old requests and other sessions cannot replace its labels. */
export function useHistory(sessionId?: number) {
  useEffect(() => {
    let alive = true;
    let stopHistory: (() => void) | undefined;
    let stopMenu: (() => void) | undefined;
    publish({ sessionId: sessionId ?? null, undo: null, redo: null });
    if (sessionId === undefined) return () => { alive = false; };
    let changed = false;
    void historyGet(sessionId).then(labels => {
      if (alive && !changed) publish({ sessionId, ...labels });
    }).catch(() => undefined);
    void onHistoryChanged(event => {
      if (!alive || event.session_id !== sessionId) return;
      changed = true;
      publish({ sessionId, undo: event.undo, redo: event.redo });
    }).then(stop => { if (alive) stopHistory = stop; else stop(); }).catch(() => undefined);
    void onEditMenu(event => {
      if (!alive) return;
      void (event.action === 'undo' ? undo(sessionId) : redo(sessionId));
    }).then(stop => { if (alive) stopMenu = stop; else stop(); }).catch(() => undefined);
    return () => { alive = false; stopHistory?.(); stopMenu?.(); };
  }, [sessionId]);
}
