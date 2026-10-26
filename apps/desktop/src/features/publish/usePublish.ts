import { useCallback, useEffect, useState } from 'react';
import { listSessions } from '@/services/sessions';
import { onSessionsChanged } from '@/services/events';
import { notificationsList, onPublishProgress, type NotificationDto } from '@/services/publish';
import type { RowLabelResponse } from '@/services/bindings';

/** Refreshes backend-derived labels after either publish or session events. */
export function usePublish(sessionId?: number) {
  const [label, setLabel] = useState<RowLabelResponse | null>(null);
  const [notifications, setNotifications] = useState<NotificationDto[]>([]);
  const refresh = useCallback(async () => {
    const [list, history] = await Promise.all([listSessions({ filter: 'all', query: null }), notificationsList()]);
    setLabel(list.groups.flatMap(group => group.rows).find(row => row.id === sessionId)?.label ?? null);
    setNotifications(history);
  }, [sessionId]);
  useEffect(() => {
    void refresh().catch(() => undefined);
    if (!('__TAURI_INTERNALS__' in window)) return;
    let alive = true;
    let offSessions: (() => void) | undefined;
    let offProgress: (() => void) | undefined;
    void onSessionsChanged(() => void refresh().catch(() => undefined)).then(off => { if (alive) offSessions = off; else off(); }).catch(() => undefined);
    void onPublishProgress(() => void refresh().catch(() => undefined)).then(off => { if (alive) offProgress = off; else off(); }).catch(() => undefined);
    return () => { alive = false; offSessions?.(); offProgress?.(); };
  }, [refresh]);
  return { label, notifications, refresh };
}
