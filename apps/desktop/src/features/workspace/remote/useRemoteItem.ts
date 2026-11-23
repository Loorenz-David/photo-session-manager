import { createContext, useCallback, useContext, useEffect, useRef, useState } from 'react';
import { onBringBack, onPictureReady, onWorkspaceEvents } from '@/services/events';
import { bringBack as bringBackCommand, getRemote, isRemote, refreshRemote, type RemoteState, type RemoteView } from '@/services/remote';

/** The Unassigned workspace has no item, so no remote state. */
const UNASSIGNED = 1;

/** A loaded answer, kept with the session it was loaded for (`RemoteViewDto` carries no session id). */
type Loaded = { sessionId: number; view: RemoteView | null };

export type RemoteItem = {
  /** The session this instance follows (`undefined` = none). */
  sessionId: number | undefined;
  /** `undefined` while not loaded for `sessionId` (a state loaded for another session counts as not loaded). */
  view: RemoteView | null | undefined;
  loaded: boolean;
  /** Archived or shop-only: the photos are on Shopify. */
  remote: boolean;
  reload: () => Promise<void>;
  refresh: () => Promise<void>;
  bringBack: () => Promise<void>;
};

/** MC-H3.4: the capture folder may follow the session: `local`, or `unknown` whose first read failed. */
export function activatable(item: Pick<RemoteItem, 'view' | 'loaded'> | undefined): boolean {
  if (!item?.loaded) return false;
  const state: RemoteState = item.view?.state ?? 'local';
  return state === 'local' || (state === 'unknown' && item.view?.check_failed === true);
}

/**
 * H4 (10H §6, MC-H3.5): the remote view of one session: `getRemote` on mount and on `session_changed` for it, the
 * `picture_ready {kind: 'remote'}` paths and `bring_back_progress` merged in place. Keyed by session id.
 */
export function useRemoteItem(sessionId: number | undefined): RemoteItem {
  const target = sessionId === undefined || sessionId === UNASSIGNED ? undefined : sessionId;
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const latest = useRef(0);

  const reload = useCallback(async () => {
    if (target === undefined) return;
    const ticket = ++latest.current;
    let view: RemoteView | null;
    try {
      view = await getRemote(target);
    } catch {
      // A view that cannot be read behaves as before this change (the backend still refuses to point the camera at a
      // remote product, MC-H3.4).
      view = null;
    }
    if (ticket === latest.current) setLoaded({ sessionId: target, view });
  }, [target]);

  useEffect(() => {
    if (target === undefined) return;
    void reload();
    let alive = true;
    const stops: (() => void)[] = [];
    const keep = (stop: () => void) => (alive ? stops.push(stop) : stop());
    Promise.resolve()
      .then(() => onWorkspaceEvents(event => {
        if (event.kind === 'session_changed' && event.session_id === target) void reload();
      }))
      .then(keep)
      .catch(() => undefined);
    Promise.resolve()
      .then(() => onPictureReady(event => {
        if (event.kind !== 'remote' || event.session_id !== target) return;
        setLoaded(current => current?.sessionId !== target || !current.view ? current : {
          ...current,
          view: {
            ...current.view,
            pictures: current.view.pictures.map(picture =>
              picture.role === event.role && picture.position === event.position ? { ...picture, path: event.path } : picture),
          },
        });
      }))
      .then(keep)
      .catch(() => undefined);
    Promise.resolve()
      .then(() => onBringBack(event => {
        if (event.session_id !== target) return;
        setLoaded(current => current?.sessionId !== target || !current.view ? current : {
          ...current,
          view: { ...current.view, bring_back: event.state === 'running' ? { done: event.done, total: event.total } : null },
        });
      }))
      .then(keep)
      .catch(() => undefined);
    return () => {
      alive = false;
      stops.forEach(stop => stop());
    };
  }, [target, reload]);

  const refresh = useCallback(async () => {
    if (target === undefined) return;
    await refreshRemote(target);
    await reload();
  }, [target, reload]);

  const bringBack = useCallback(async () => {
    if (target === undefined) return;
    await bringBackCommand(target);
    await reload();
  }, [target, reload]);

  const view = loaded !== null && loaded.sessionId === target ? loaded.view : undefined;
  return { sessionId: target, view, loaded: view !== undefined, remote: isRemote(view?.state), reload, refresh, bringBack };
}

/** The remote view of the session on screen, shared by the app shell (capture target, header menu, visit bar) and the workspace. */
export const RemoteItemContext = createContext<RemoteItem | null>(null);

/** The shared instance when it follows `sessionId`, else `null`. */
export function useSharedRemoteItem(sessionId: number | undefined): RemoteItem | null {
  const shared = useContext(RemoteItemContext);
  return shared && shared.sessionId !== undefined && shared.sessionId === sessionId ? shared : null;
}
