import { useMemo, useRef, useSyncExternalStore } from 'react';
import { toast } from 'sonner';
import { setComposition, type Composition, type CompositionValues } from '@/services/compositions';

type SavesOptions = {
  /** Called with the backend's row after every successful save (the workspace row is patched, tiles redraw). */
  onStored: (photoId: number, composition: Composition) => void;
  /**
   * Called only when the photo's queue is quiet — a success with nothing newer waiting, or a refusal (with the last
   * stored row) — so the editor (machine rule 16) never hears an older answer while a newer save is still queued.
   */
  onSettled: (photoId: number, composition: Composition) => void;
  /** The last composition known to be stored for the photo (the workspace view's row). */
  storedOf: (photoId: number) => Composition | null;
};

type Entry = { inFlight: boolean; pending: CompositionValues | null; latest: number };

let pendingIds = new Set<number>();
const listeners = new Set<() => void>();
function markPending(photoId: number, pending: boolean) {
  const next = new Set(pendingIds);
  if (pending) next.add(photoId);
  else next.delete(photoId);
  pendingIds = next;
  listeners.forEach(listener => listener());
}
export function usePendingCompositions(): ReadonlySet<number> {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => { listeners.delete(listener); }; }, () => pendingIds);
}

/**
 * Autosave, latest-wins per photo (plan P31 task 4): one save in flight per photo, the newest values waiting behind
 * it; a settle older than the photo's latest ticket is ignored; a refused save tells the photographer and hands the
 * stored composition back so the editor snaps to it (machine rule 16). An older success that lands while a newer save
 * is queued reaches the tiles (`onStored`) but not the editor (`onSettled`), so the stage stays on what the
 * photographer sees (review r1 F1). Leaving the editor flushes nothing extra.
 */
export function useCompositionSaves(options: SavesOptions): (photoId: number, values: CompositionValues) => void {
  const latest = useRef(options);
  latest.current = options;
  return useMemo(() => {
    const entries = new Map<number, Entry>();
    const settle = (photoId: number, ticket: number, composition: Composition | null, error: unknown) => {
      const entry = entries.get(photoId);
      if (!entry || ticket !== entry.latest) return;
      entry.inFlight = false;
      if (composition) {
        latest.current.onStored(photoId, composition);
        const next = entry.pending;
        entry.pending = null;
        if (next) send(photoId, next);
        else {
          markPending(photoId, false);
          latest.current.onSettled(photoId, composition);
        }
        return;
      }
      entry.pending = null;
      markPending(photoId, false);
      toast.error(`Not saved: ${(error as Error).message}`);
      const stored = latest.current.storedOf(photoId);
      if (stored) latest.current.onSettled(photoId, stored);
    };
    const send = (photoId: number, values: CompositionValues) => {
      const entry = entries.get(photoId)!;
      const ticket = ++entry.latest;
      entry.inFlight = true;
      setComposition(photoId, values).then(
        composition => settle(photoId, ticket, composition, null),
        error => settle(photoId, ticket, null, error),
      );
    };
    return (photoId: number, values: CompositionValues) => {
      let entry = entries.get(photoId);
      if (!entry) {
        entry = { inFlight: false, pending: null, latest: 0 };
        entries.set(photoId, entry);
      }
      if (entry.inFlight) entry.pending = values;
      else send(photoId, values);
      markPending(photoId, true);
    };
  }, []);
}
