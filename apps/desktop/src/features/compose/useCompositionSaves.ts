import { useMemo, useRef } from 'react';
import { toast } from 'sonner';
import { setComposition, type Composition, type CompositionValues } from '@/services/compositions';

type SavesOptions = {
  /** Called with the stored composition after every settled save (success: the backend's row; failure: the last stored row). */
  onSaved: (photoId: number, composition: Composition) => void;
  /** The last composition known to be stored for the photo (the workspace view's row). */
  storedOf: (photoId: number) => Composition | null;
};

type Entry = { inFlight: boolean; pending: CompositionValues | null; latest: number };

/**
 * Autosave, latest-wins per photo (plan P31 task 4): one save in flight per photo, the newest values waiting behind
 * it; a settle older than the photo's latest ticket is ignored; a refused save tells the photographer and hands the
 * stored composition back so the editor snaps to it (machine rule 16). Leaving the editor flushes nothing extra.
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
        latest.current.onSaved(photoId, composition);
        const next = entry.pending;
        entry.pending = null;
        if (next) send(photoId, next);
        return;
      }
      entry.pending = null;
      toast.error(`Not saved: ${(error as Error).message}`);
      const stored = latest.current.storedOf(photoId);
      if (stored) latest.current.onSaved(photoId, stored);
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
    };
  }, []);
}
