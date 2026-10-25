import { useEffect, useRef } from 'react';
import { toast } from 'sonner';
import { onPhotoAdded } from '@/services/events';

const UNASSIGNED_SESSION_ID = 1;
export const ARRIVAL_TOAST_ID = 'unassigned-arrival';

type ArrivalToastsProps = {
  /** The session the window shows (undefined on the Sessions page). */
  sessionId?: number;
  navigate: (sessionId?: number) => void;
};

/**
 * ch04 S-E: photos that arrive in Unassigned while the user is anywhere but the Unassigned workspace raise one toast
 * "Photos arrived in Unassigned" with "Open" (repeated arrivals refresh the same toast).
 */
export function ArrivalToasts({ sessionId, navigate }: ArrivalToastsProps) {
  const where = useRef({ sessionId, navigate });
  where.current = { sessionId, navigate };

  useEffect(() => {
    let stop: (() => void) | undefined;
    let alive = true;
    onPhotoAdded(event => {
      if (event.session_id !== UNASSIGNED_SESSION_ID || where.current.sessionId === UNASSIGNED_SESSION_ID) return;
      toast('Photos arrived in Unassigned', {
        id: ARRIVAL_TOAST_ID,
        duration: 6000,
        action: { label: 'Open', onClick: () => where.current.navigate(UNASSIGNED_SESSION_ID) },
      });
    })
      .then(unlisten => {
        if (alive) stop = unlisten;
        else unlisten();
      })
      .catch(() => undefined);
    return () => {
      alive = false;
      stop?.();
    };
  }, []);

  return null;
}
