import { useEffect, useState } from 'react';
import { onWorkspaceEvents } from '@/services/events';
import { onHistoryChanged } from '@/services/history';
import { getTemplateView, type TemplateView } from '@/services/templates';

/** Reads placement when a session opens and when its photos change. */
export function useTemplateView(sessionId: number): TemplateView | null {
  const [view, setView] = useState<TemplateView | null>(null);
  useEffect(() => {
    let alive = true;
    let ticket = 0;
    let stop: (() => void) | undefined;
    let stopHistory: (() => void) | undefined;
    setView(null);
    const reload = () => {
      const request = ++ticket;
      void getTemplateView(sessionId).then(next => {
        if (alive && request === ticket) setView(next);
      }).catch(() => {
        if (alive && request === ticket) setView(null);
      });
    };
    reload();
    void onWorkspaceEvents(event => {
      if (event.session_id === sessionId && (event.kind === 'photo_added' || event.kind === 'photo_changed' || event.kind === 'session_changed')) reload();
    }).then(unlisten => {
      if (alive) stop = unlisten;
      else unlisten();
    }).catch(() => undefined);
    // A person's action (classify, reorder, compose, drop, undo…) records or replays a step: "differs" may change.
    void onHistoryChanged(event => {
      if (event.session_id === sessionId) reload();
    }).then(unlisten => {
      if (alive) stopHistory = unlisten;
      else unlisten();
    }).catch(() => undefined);
    return () => { alive = false; stop?.(); stopHistory?.(); };
  }, [sessionId]);
  return view;
}
