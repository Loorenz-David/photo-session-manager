import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { ArrowDown, Search } from 'lucide-react';
import { toast } from 'sonner';
import { Checkbox } from '@/components/ui/checkbox';
import { withinRange, type TimeRange } from '@/lib/format';
import { useDebounced, useKeyboard } from '@/lib/keyboard';
import type { SessionFilter } from '@/services/bindings';
import { onSessionsChanged } from '@/services/events';
import { BatchQueue } from '@/features/publish/BatchQueue';
import { PublishPopover } from '@/features/publish/PublishPopover';
import { publishRetry } from '@/services/publish';
import { listSessions, type SessionList } from '@/services/sessions';
import { BatchBar } from './BatchBar';
import { Filters } from './Filters';
import { GroupHeader } from './GroupHeader';
import { SessionRow, isSelectable } from './SessionRow';
import { TimeFilter } from './TimeFilter';
import { UnassignedCard } from './UnassignedCard';

export const SEARCH_DEBOUNCE_MS = 250;

/** Sessions list (ch04 S-A, ch07): grouped rows as delivered by `sessions_list`, filters, search, time filter. */
export function SessionsPage({ navigate }: { navigate: (sessionId: number) => void }) {
  const [filter, setFilter] = useState<SessionFilter>('open');
  const [query, setQuery] = useState('');
  const [range, setRange] = useState<TimeRange>('any');
  const [list, setList] = useState<SessionList | null>(null);
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [queueIds, setQueueIds] = useState<number[]>([]);
  const [publishId, setPublishId] = useState<number | null>(null);
  const debouncedQuery = useDebounced(query.trim(), SEARCH_DEBOUNCE_MS);
  const request = useRef(0);

  const load = useCallback(async () => {
    const ticket = ++request.current;
    try {
      const next = await listSessions({ filter, query: debouncedQuery || null });
      if (ticket === request.current) setList(next);
    } catch (error) {
      if (ticket === request.current) toast.error(`Could not load sessions: ${(error as Error).message}`);
    }
  }, [filter, debouncedQuery]);

  useEffect(() => {
    void load();
  }, [load]);

  const latestLoad = useRef(load);
  latestLoad.current = load;
  useEffect(() => {
    let alive = true;
    let unlisten: (() => void) | undefined;
    onSessionsChanged(() => void latestLoad.current())
      .then(off => (alive ? (unlisten = off) : off()))
      .catch(() => undefined);
    return () => {
      alive = false;
      unlisten?.();
    };
  }, []);

  const now = Date.now();
  const groups = useMemo(() => {
    const at = Date.now();
    return (list?.groups ?? [])
      .map(group => ({ ...group, rows: group.rows.filter(row => withinRange(row.last_activity_at, range, at)) }))
      .filter(group => group.rows.length > 0);
  }, [list, range]);
  const selectable = useMemo(() => groups.flatMap(group => group.rows).filter(isSelectable).map(row => row.id), [groups]);

  // Keep the selection to rows still visible and selectable.
  useEffect(() => {
    setSelected(previous => {
      const kept = new Set([...previous].filter(id => selectable.includes(id)));
      return kept.size === previous.size ? previous : kept;
    });
  }, [selectable]);

  useKeyboard({ escape: () => setSelected(new Set()) }, selected.size > 0);

  function toggle(id: number) {
    setSelected(previous => {
      const next = new Set(previous);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  const allSelected = selectable.length > 0 && selected.size === selectable.length;
  const headerState = allSelected ? true : selected.size > 0 ? 'indeterminate' : false;
  const searching = debouncedQuery !== '' || range !== 'any';

  return (
    <main className="sessions-page">
      <div className="list-toolbar">
        <label className="search-field">
          <Search aria-hidden />
          <input
            type="search"
            placeholder="Title or SKU"
            aria-label="Search sessions by title or SKU"
            value={query}
            onChange={event => setQuery(event.target.value)}
          />
          <TimeFilter value={range} onChange={setRange} />
        </label>
        <Filters value={filter} counts={list?.counts ?? null} onChange={setFilter} />
      </div>

      <UnassignedCard count={list?.unassigned_count ?? 0} onOpen={() => navigate(1)} />

      {list && groups.length === 0 ? (
        <div className="empty-sessions">
          {searching ? (
            <>
              <strong>No sessions match{debouncedQuery ? ` “${debouncedQuery}”` : ''}</strong>
              <p>Try another title or SKU, or widen the time filter.</p>
            </>
          ) : filter === 'published' ? (
            <>
              <strong>Nothing published yet</strong>
              <p>Published sessions appear here.</p>
            </>
          ) : (
            <>
              <strong>No sessions yet</strong>
              <p>
                Press <kbd>N</kbd> to start a new session.
              </p>
            </>
          )}
        </div>
      ) : (
        <div className="session-table" role="table" aria-label="Sessions">
          <div className="list-labels" role="row">
            <span className="row-check">
              <Checkbox
                checked={headerState}
                disabled={selectable.length === 0}
                onCheckedChange={() => setSelected(allSelected ? new Set() : new Set(selectable))}
                aria-label="Select all publishable sessions"
              />
            </span>
            <span>Item</span>
            <span>Photos</span>
            <span>Status</span>
            <span className="label-activity">
              Last activity <ArrowDown aria-hidden />
            </span>
          </div>
          {groups.map(group => (
            <section key={group.label} className="session-group" aria-label={group.label}>
              <GroupHeader label={group.label} count={group.rows.length} />
              {group.rows.map(row => (
                <SessionRow
                  key={row.id}
                  row={row}
                  now={now}
                  selected={selected.has(row.id)}
                  onToggle={() => toggle(row.id)}
                  onOpen={() => navigate(row.id)}
                  onPublish={() => setPublishId(row.id)}
                  onRetry={row.publish?.item_id ? () => void publishRetry(row.publish!.item_id).then(load).catch(error => toast.error(`Retry failed: ${(error as Error).message}`)) : undefined}
                />
              ))}
            </section>
          ))}
        </div>
      )}

      <BatchBar
        count={selected.size}
        selectableCount={selectable.length}
        onSelectAll={() => setSelected(new Set(selectable))}
        onClear={() => setSelected(new Set())}
        onReview={() => setQueueIds([...selected])}
      />
      {queueIds.length > 0 && <BatchQueue ids={queueIds} onClose={() => { setQueueIds([]); setSelected(new Set()); void load(); }} />}
      {publishId !== null && <PublishPopover sessionIds={[publishId]} onClose={() => { setPublishId(null); void load(); }} />}
    </main>
  );
}
