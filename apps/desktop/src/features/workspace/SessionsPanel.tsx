import { useEffect, useState } from 'react';
import { useDroppable } from '@dnd-kit/core';
import { Search, X } from 'lucide-react';
import { useDebounced } from '@/lib/keyboard';
import { onSessionsChanged } from '@/services/events';
import { listSessions, previewUrl } from '@/services/sessions';
import type { SessionRowResponse_Serialize } from '@/services/bindings';

export type SessionRow = SessionRowResponse_Serialize;
export const sessionDropId = (sessionId: number) => `session:${sessionId}`;

/** Published sessions are never move or copy targets (MC-2, ratify R-I12/R-I3). */
export function acceptsPhotos(row: SessionRow): boolean {
  return row.status !== 'published';
}

function photoCount(row: SessionRow): number {
  return row.main_count + row.condition_count + row.unreviewed_count;
}

function SessionDropRow({ row }: { row: SessionRow }) {
  const enabled = acceptsPhotos(row);
  const { setNodeRef, isOver } = useDroppable({ id: sessionDropId(row.id), disabled: !enabled });
  return (
    <li
      ref={setNodeRef}
      className="sessions-panel-row"
      data-session-id={row.id}
      data-droppable={enabled}
      data-over={(enabled && isOver) || undefined}
      aria-disabled={!enabled || undefined}
      title={enabled ? `Drop photos to move them to ${row.title}` : 'Published sessions cannot receive photos'}
    >
      <span className="row-thumb">{row.cover_preview_path && <img src={previewUrl(row.cover_preview_path)} alt="" />}</span>
      <span className="row-name">
        <strong>{row.item_type || row.title}</strong>
        <small>
          {row.sku} · {photoCount(row)} photos{enabled ? '' : ' · published'}
        </small>
      </span>
    </li>
  );
}

/** Sessions panel (NOTES.md §Screens 5): drag photos onto a session to move them there. */
export function SessionsPanel({ currentSessionId, onClose }: { currentSessionId: number; onClose: () => void }) {
  const [query, setQuery] = useState('');
  const [rows, setRows] = useState<SessionRow[]>([]);
  const [version, setVersion] = useState(0);
  const debounced = useDebounced(query.trim(), 200);

  useEffect(() => {
    let alive = true;
    listSessions({ filter: 'all', query: debounced || null })
      .then(list => {
        if (!alive) return;
        setRows(list.groups.flatMap(group => group.rows).filter(row => row.id !== currentSessionId));
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [debounced, currentSessionId, version]);

  useEffect(() => {
    let stop: (() => void) | undefined;
    let alive = true;
    onSessionsChanged(() => setVersion(current => current + 1))
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

  return (
    <aside className="sessions-panel" aria-label="Sessions">
      <div className="panel-head">
        <span className="panel-title">Sessions</span>
        <button type="button" className="stage-icon-button" onClick={onClose} aria-label="Close sessions panel">
          <X />
        </button>
      </div>
      <label className="search-field sessions-search">
        <Search aria-hidden />
        <input value={query} onChange={event => setQuery(event.target.value)} placeholder="Title or SKU" aria-label="Search sessions" />
      </label>
      <ul className="sessions-panel-list">
        {rows.map(row => (
          <SessionDropRow key={row.id} row={row} />
        ))}
      </ul>
    </aside>
  );
}
