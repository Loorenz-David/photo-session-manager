import { useEffect, useMemo, useRef, useState } from 'react';
import { ArrowDown, Search, Sparkles, X } from 'lucide-react';
import { toast } from 'sonner';
import { Checkbox } from '@/components/ui/checkbox';
import { SkeletonRows } from '@/components/pending/SkeletonRows';
import { rangeStart, type TimeRange } from '@/lib/format';
import { useDebounced, useKeyboard } from '@/lib/keyboard';
import { usePageData } from '@/lib/pending';
import type { SessionFilter } from '@/services/bindings';
import { onLabelsChanged, onSessionsChanged } from '@/services/events';
import { labelsSummary } from '@/services/labels';
import { BatchQueue } from '@/features/publish/BatchQueue';
import { PublishPopover } from '@/features/publish/PublishPopover';
import { publishRetry } from '@/services/publish';
import { sessionsPage } from '@/services/sessions';
import { BatchBar } from './BatchBar';
import { Filters, type ListBox } from './Filters';
import { GroupHeader } from './GroupHeader';
import { Pagination } from './Pagination';
import { SessionRow, isSelectable } from './SessionRow';
import { useResearchOverview } from './ResearchPill';
import { waitingForReview } from './reviewQueue';
import { TimeFilter } from './TimeFilter';
import { UnassignedCard } from './UnassignedCard';
import { usePictures } from './usePictures';
import { useSessionBadges } from '@/features/conversations/SessionBadge';
import { useSessionLabels } from '@/features/labels/useSessionLabels';
import { LabelsBox, useThrottled } from '@/features/labels/LabelsBox';
import { LABELS_REFRESH_MS } from '@/features/labels/labelPages';

export const SEARCH_DEBOUNCE_MS = 250;

/**
 * Sessions list (ch04 S-A, ch07): grouped rows as delivered by `sessions_page` (creation order, newest created on top,
 * grouped by creation time: changelog 46 (1)), filters, search, time filter, 25 rows per page (H2, MC-H2). The search,
 * the filter tabs, the time filter and "To review" look through every session (the backend applies them); changing
 * one of them goes to page 1.
 *
 * L2 (09C §5): a fourth box, Labels, shows Pricing's label list (`LabelsBox`) in place of the session table; the page
 * keeps asking `sessions_page` for the box it left (the other boxes' numbers stay current) but shows none of its rows.
 * The Labels box's number is asked once when the page opens and after a label change outside the box (owner card 1).
 */
export function SessionsPage({ navigate }: { navigate: (sessionId: number) => void }) {
  const [filter, setFilter] = useState<SessionFilter>('open');
  const [box, setBox] = useState<ListBox>('open');
  const inLabels = box === 'labels';
  const [query, setQuery] = useState('');
  const [labelQuery, setLabelQuery] = useState('');
  const debouncedLabelQuery = useDebounced(labelQuery.trim(), SEARCH_DEBOUNCE_MS);
  const [labelsNumber, setLabelsNumber] = useState<number | null>(null);
  const [range, setRange] = useState<TimeRange>('any');
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [queueIds, setQueueIds] = useState<number[]>([]);
  const [publishId, setPublishId] = useState<number | null>(null);
  // A19: only the sessions whose research finished with AI suggestions waiting (the review queue).
  const [reviewOnly, setReviewOnly] = useState(false);
  const debouncedQuery = useDebounced(query.trim(), SEARCH_DEBOUNCE_MS);
  const research = useResearchOverview(null);
  // A19: the sessions whose suggestions wait for review, from the research overview alone (every session).
  const waiting = useMemo(
    () => [...research.values()].filter(waitingForReview).map(row => row.session_id).sort((a, b) => a - b),
    [research],
  );
  const onlyIds = reviewOnly ? waiting : null;
  // MC-H2.3: the page belongs to the question asked; a new filter, search, time filter or "To review" is page 1.
  const question = JSON.stringify([filter, debouncedQuery, range, onlyIds]);
  const [asked, setAsked] = useState({ question, page: 1 });
  const page = asked.question === question ? asked.page : 1;
  const { data: list, loading, error, reload: load } = usePageData(
    () => sessionsPage({ filter, query: debouncedQuery || null, created_from: rangeStart(range), only_ids: onlyIds, page }),
    [question, page],
  );
  // MC-H2.1: an answer clamped to the last page is the page shown.
  useEffect(() => {
    if (list && list.page !== page) setAsked({ question, page: list.page });
    // Only a new answer moves the page.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [list]);
  const pictures = usePictures();
  // C2 (09B §6): "N open" / "Pricing is waiting" per session.
  const conversationBadges = useSessionBadges();
  useEffect(() => { if (error) toast.error(`Could not load sessions: ${(error as Error).message}`); }, [error]);

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
  const groups = useMemo(() => list?.groups ?? [], [list]);
  // L1 (MC-L3.7): the page's label markers, asked after each page answer (never delaying the rows).
  const shownIds = useMemo(() => groups.flatMap(group => group.rows).map(row => row.id), [groups]);
  const labels = useSessionLabels(inLabels ? null : shownIds, inLabels ? box : list);

  // L2 (MC-L1.6, owner card 1): the Labels box's number outside the box: asked when the page opens, after leaving the
  // box and on a label change (throttled); inside the box it is the last list answer's.
  function askSummary() {
    try {
      labelsSummary({ refresh: true }).then(got => setLabelsNumber(got.missing_price_tag), () => undefined);
    } catch {
      // No number.
    }
  }
  const summaryOnChange = useThrottled(askSummary, LABELS_REFRESH_MS);
  const inLabelsRef = useRef(inLabels);
  inLabelsRef.current = inLabels;
  useEffect(() => {
    askSummary();
    let alive = true;
    let unlisten: (() => void) | undefined;
    try {
      onLabelsChanged(() => {
        if (!inLabelsRef.current) summaryOnChange();
      })
        .then(off => (alive ? (unlisten = off) : off()))
        .catch(() => undefined);
    } catch {
      // No listener: the number follows the next visit.
    }
    return () => {
      alive = false;
      unlisten?.();
    };
    // Once per mount.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function changeBox(next: ListBox) {
    if (next === box) return;
    if (next === 'labels') {
      setBox('labels');
      return;
    }
    if (inLabels) askSummary();
    setFilter(next);
    setBox(next);
  }
  const toReview = waiting.length;
  const selectable = useMemo(() => groups.flatMap(group => group.rows).filter(isSelectable).map(row => row.id), [groups]);

  // Keep the selection to rows still visible and selectable.
  useEffect(() => {
    setSelected(previous => {
      const kept = new Set([...previous].filter(id => selectable.includes(id)));
      return kept.size === previous.size ? previous : kept;
    });
  }, [selectable]);

  useKeyboard({ escape: () => setSelected(new Set()) }, selected.size > 0 && !inLabels);

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
          {inLabels ? (
            <input
              type="search"
              placeholder="SKU, title or type"
              aria-label="Search labels by SKU, title or type"
              value={labelQuery}
              onChange={event => setLabelQuery(event.target.value)}
            />
          ) : (
            <input
              type="search"
              placeholder="Title or SKU"
              aria-label="Search sessions by title or SKU"
              value={query}
              onChange={event => setQuery(event.target.value)}
            />
          )}
          {!inLabels && <TimeFilter value={range} onChange={setRange} />}
        </label>
        <Filters value={box} counts={list?.counts ?? null} labelsCount={labelsNumber} onChange={changeBox} />
        {!inLabels && (toReview > 0 || reviewOnly) && (
          <button
            type="button"
            className="pill research-pill review-queue-toggle"
            aria-pressed={reviewOnly}
            title={reviewOnly ? 'Show every session again' : 'Show only the sessions whose AI suggestions wait for review'}
            onClick={() => setReviewOnly(on => !on)}
          >
            <Sparkles aria-hidden /> To review · {toReview}
            {reviewOnly && <X aria-hidden />}
          </button>
        )}
      </div>

      {inLabels ? (
        <LabelsBox
          query={debouncedLabelQuery}
          navigate={navigate}
          onNumber={setLabelsNumber}
          research={research}
          conversations={conversationBadges}
        />
      ) : (
        <>
          <UnassignedCard count={list?.unassigned_count ?? 0} onOpen={() => navigate(1)} />

          {loading ? <div className="session-table" role="table" aria-label="Sessions" aria-busy="true"><SkeletonRows n={6} /></div> : list && groups.length === 0 ? (
            <div className="empty-sessions">
              {searching ? (
                <>
                  <strong>No sessions match{debouncedQuery ? ` “${debouncedQuery}”` : ''}</strong>
                  <p>Try another title or SKU, or widen the time filter.</p>
                </>
              ) : reviewOnly ? (
                <>
                  <strong>Nothing waits for review</strong>
                  <p>Sessions appear here once AI research has suggestions for you to check.</p>
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
                  Created <ArrowDown aria-hidden />
                </span>
                <span aria-hidden />
              </div>
              {groups.map(group => (
                <section key={group.label} className="session-group" aria-label={group.label}>
                  <GroupHeader label={group.label} count={group.total} />
                  {group.rows.map(row => (
                    <SessionRow
                      key={row.id}
                      row={row}
                      now={now}
                      research={research.get(row.id)}
                      conversations={conversationBadges.get(row.id)}
                      picture={pictures.cover(row.id)}
                      label={labels.labels.get(row.id)}
                      labelsAccess={labels.access}
                      selected={selected.has(row.id)}
                      onToggle={() => toggle(row.id)}
                      onOpen={() => navigate(row.id)}
                      onPublish={() => setPublishId(row.id)}
                      onRetry={row.publish?.item_id ? () => publishRetry(row.publish!.item_id).then(load) : undefined}
                      onChanged={() => void load()}
                      onDeleted={() => void load()}
                    />
                  ))}
                </section>
              ))}
            </div>
          )}
          {list && <Pagination page={list.page} pages={list.pages} onPage={next => setAsked({ question, page: next })} />}

          <BatchBar
            count={selected.size}
            selectableCount={selectable.length}
            onSelectAll={() => setSelected(new Set(selectable))}
            onClear={() => setSelected(new Set())}
            onReview={() => setQueueIds([...selected])}
          />
        </>
      )}
      {queueIds.length > 0 && <BatchQueue ids={queueIds} onClose={() => { setQueueIds([]); setSelected(new Set()); void load(); }} />}
      {publishId !== null && <PublishPopover sessionIds={[publishId]} onClose={() => { setPublishId(null); void load(); }} />}
    </main>
  );
}
