import { useCallback, useEffect, useMemo, useState } from 'react';
import { onPictureReady, type PictureReady } from '@/services/events';

export type Pictures = {
  /** The cover kept on this Mac for a session since the list was answered (`picture_ready` kind `cover`). */
  cover: (sessionId: number) => string | undefined;
  /** The picture kept on this Mac for a shop search result (`picture_ready` kind `search`). */
  search: (externalId: string) => string | undefined;
};

/**
 * H2 (MC-H1.3, MC-H2.5): the pictures that arrived after an answer was shown, by session id and by shop product id.
 * One `picture_ready` listener per mount; a picture updates its row without asking the list again. A missing listener
 * never breaks the page (the row keeps what the answer carried).
 */
export function usePictures(): Pictures {
  const [covers, setCovers] = useState<ReadonlyMap<number, string>>(() => new Map());
  const [searches, setSearches] = useState<ReadonlyMap<string, string>>(() => new Map());
  useEffect(() => {
    let alive = true;
    let off: (() => void) | undefined;
    const take = (event: PictureReady) => {
      if (event.kind === 'cover') setCovers(previous => new Map(previous).set(event.session_id, event.path));
      else if (event.kind === 'search') setSearches(previous => new Map(previous).set(event.external_id, event.path));
    };
    Promise.resolve()
      .then(() => onPictureReady(take))
      .then(unlisten => (alive ? (off = unlisten) : unlisten?.()))
      .catch(() => undefined);
    return () => {
      alive = false;
      off?.();
    };
  }, []);
  const cover = useCallback((sessionId: number) => covers.get(sessionId), [covers]);
  const search = useCallback((externalId: string) => searches.get(externalId), [searches]);
  return useMemo(() => ({ cover, search }), [cover, search]);
}
