import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';
import { onWorkspaceEvents } from '@/services/events';
import {
  classifyPhoto,
  copyPhotos,
  deletePhoto,
  getSessionView,
  movePhotos,
  reorderPhoto,
  undoDelete,
  type Classification,
  type Photo,
  type ReviewResult,
  type SessionView,
} from '@/services/photos';
import { ServiceError } from '@/services/sessions';
import { displayOrder, displaySequence } from './order';

export type ViewMode = 'adjusted' | 'original';

/** Undo toast lifetime (evidence design_handoff §Review; NOTES.md §Undo timing). */
export const UNDO_TOAST_MS = 4500;
export const UNDO_TOAST_ID = 'photo-deleted';

function refusalText(error: unknown): string {
  if (error instanceof ServiceError) {
    if (error.code === 'target_published') return 'Published sessions cannot receive photos.';
    if (error.code === 'target_has_raw') return 'That session already has this photo.';
    if (error.code === 'nothing_to_undo') return 'Nothing to undo.';
  }
  return (error as Error).message || 'Something went wrong.';
}

/** First unreviewed photo in display order, else the first photo shown. */
function defaultActive(photos: readonly Photo[]): number | null {
  const order = displayOrder(photos);
  return order.unreviewed[0]?.id ?? displaySequence(photos)[0]?.id ?? null;
}

/**
 * Which photo opens after an action on `actedId` (ch06: "after an action the next unreviewed photo opens"):
 * the first remaining unreviewed photo in display order; with none left, the acted photo when it stays in the
 * session, otherwise its neighbour in the display sequence.
 */
function nextAfterAction(photos: readonly Photo[], actedId: number, actedLeaves: boolean): number | null {
  const next = displayOrder(photos).unreviewed.find(photo => photo.id !== actedId);
  if (next) return next.id;
  if (!actedLeaves) return actedId;
  const sequence = displaySequence(photos);
  const index = sequence.findIndex(photo => photo.id === actedId);
  const neighbour = sequence[index + 1] ?? sequence[index - 1];
  return neighbour && neighbour.id !== actedId ? neighbour.id : null;
}

export type WorkspaceState = ReturnType<typeof useWorkspaceState>;

/** Session workspace state: the loaded session view, the active photo, the Original/Adjusted mode, review actions. */
export function useWorkspaceState(sessionId: number) {
  const [view, setView] = useState<SessionView | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [activeId, setActiveIdState] = useState<number | null>(null);
  const [mode, setMode] = useState<ViewMode>('adjusted');
  const [selection, setSelection] = useState<ReadonlySet<number>>(new Set());
  const activeRef = useRef<number | null>(null);
  const request = useRef(0);

  const setActiveId = useCallback((id: number | null) => {
    activeRef.current = id;
    setActiveIdState(id);
  }, []);

  const reload = useCallback(async () => {
    const ticket = ++request.current;
    try {
      const next = await getSessionView(sessionId);
      if (ticket !== request.current) return;
      setView(next);
      setLoadError(null);
      // The active photo stays while it is visible; otherwise the default rule picks one.
      if (activeRef.current === null || !next.photos.some(photo => photo.id === activeRef.current)) {
        setActiveId(defaultActive(next.photos));
      }
      setSelection(current => {
        const visible = new Set(next.photos.map(photo => photo.id));
        const kept = [...current].filter(id => visible.has(id));
        return kept.length === current.size ? current : new Set(kept);
      });
    } catch (error) {
      if (ticket === request.current) setLoadError((error as Error).message || 'Could not load the session.');
    }
  }, [sessionId, setActiveId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let alive = true;
    onWorkspaceEvents(event => {
      if (event.session_id !== sessionId) return;
      // A new capture appears on the stage (ch04 S-C).
      if (event.kind === 'photo_added') activeRef.current = event.photo_id;
      void reload();
    })
      .then(stop => {
        if (alive) unlisten = stop;
        else stop();
      })
      .catch(() => undefined);
    return () => {
      alive = false;
      unlisten?.();
    };
  }, [sessionId, reload]);

  const photos = useMemo(() => view?.photos ?? [], [view]);
  const photosRef = useRef(photos);
  photosRef.current = photos;
  const active = photos.find(photo => photo.id === activeId) ?? null;

  /** Runs a review command, then opens `next` and reloads; failures are told, never silent. */
  const run = useCallback(
    async (command: () => Promise<ReviewResult>, next?: number | null): Promise<ReviewResult | null> => {
      try {
        const result = await command();
        if (next !== undefined) setActiveId(next);
        await reload();
        return result;
      } catch (error) {
        toast.error(refusalText(error));
        await reload();
        return null;
      }
    },
    [reload, setActiveId],
  );

  /** Classify into a group; a review action (`advance`) then opens the next unreviewed photo, a drag does not. */
  const classify = useCallback(
    (photoId: number, classification: Exclude<Classification, 'deleted' | 'unreviewed'>, advance = true) => {
      const next = advance ? nextAfterAction(photosRef.current, photoId, false) : undefined;
      return run(() => classifyPhoto(photoId, classification), next);
    },
    [run],
  );

  /** Opens the next unreviewed photo without changing anything (A on a photo already in Main). */
  const advanceFrom = useCallback(
    (photoId: number) => setActiveId(nextAfterAction(photosRef.current, photoId, false)),
    [setActiveId],
  );

  const undo = useCallback(async () => {
    try {
      const result = await undoDelete(sessionId);
      toast.dismiss(UNDO_TOAST_ID);
      if (result.active_photo_id !== null) activeRef.current = result.active_photo_id;
      setActiveId(result.active_photo_id ?? activeRef.current);
      await reload();
    } catch (error) {
      toast.error(refusalText(error));
    }
  }, [sessionId, reload, setActiveId]);

  const remove = useCallback(
    async (photoId: number) => {
      const next = nextAfterAction(photosRef.current, photoId, true);
      const result = await run(() => deletePhoto(photoId), next);
      if (result) {
        toast('Photo deleted · Undo ⌘Z', {
          id: UNDO_TOAST_ID,
          duration: UNDO_TOAST_MS,
          action: { label: 'Undo', onClick: () => void undo() },
        });
      }
    },
    [run, undo],
  );

  const reorder = useCallback((photoId: number, position: number) => run(() => reorderPhoto(photoId, position)), [run]);

  const move = useCallback(
    (photoIds: number[], target: number) => {
      const leaving = new Set(photoIds);
      const rest = photosRef.current.filter(photo => !leaving.has(photo.id));
      const next = activeRef.current !== null && leaving.has(activeRef.current) ? defaultActive(rest) : undefined;
      return run(() => movePhotos(photoIds, target), next);
    },
    [run],
  );

  const copy = useCallback((photoIds: number[], target: number) => run(() => copyPhotos(photoIds, target)), [run]);

  const step = useCallback(
    (delta: number) => {
      const sequence = displaySequence(photosRef.current);
      if (sequence.length === 0) return;
      const index = sequence.findIndex(photo => photo.id === activeRef.current);
      const target = sequence[Math.min(sequence.length - 1, Math.max(0, index + delta))];
      setActiveId(target.id);
    },
    [setActiveId],
  );

  const toggleMode = useCallback(() => setMode(current => (current === 'adjusted' ? 'original' : 'adjusted')), []);

  const toggleSelected = useCallback((photoId: number) => {
    setSelection(current => {
      const next = new Set(current);
      if (next.has(photoId)) next.delete(photoId);
      else next.add(photoId);
      return next;
    });
  }, []);
  const clearSelection = useCallback(() => setSelection(new Set()), []);

  return {
    sessionId,
    view,
    loadError,
    photos,
    active,
    activeId,
    setActiveId,
    mode,
    toggleMode,
    selection,
    toggleSelected,
    clearSelection,
    reload,
    classify,
    advanceFrom,
    remove,
    undo,
    reorder,
    move,
    copy,
    step,
  };
}
