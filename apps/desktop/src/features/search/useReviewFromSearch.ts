import { useEffect, useRef } from 'react';
import { onOpenAiReview } from '@/services/search';

/**
 * A14: "Open AI review" in a "Search this photo" window opens that session's details on the AI review tab in the
 * main window. Another session than the one shown is opened first. Call it after the effect that closes the
 * details when the session changes, so the review opens after that.
 */
export function useReviewFromSearch(sessionId: number | undefined, navigate: (sessionId?: number) => void, openReview: () => void) {
  const pending = useRef<number | null>(null);
  const shown = useRef(sessionId);
  shown.current = sessionId;
  const open = useRef(openReview);
  open.current = openReview;

  useEffect(() => {
    if (pending.current === null || pending.current !== sessionId) return;
    pending.current = null;
    open.current();
  }, [sessionId]);

  useEffect(() => {
    let alive = true;
    let off: (() => void) | undefined;
    Promise.resolve()
      .then(() => onOpenAiReview(reviewed => {
        if (reviewed === shown.current) {
          open.current();
        } else {
          pending.current = reviewed;
          navigate(reviewed);
        }
      }))
      .then(unlisten => { if (alive) off = unlisten; else unlisten(); })
      .catch(() => undefined);
    return () => { alive = false; off?.(); };
  }, [navigate]);
}
