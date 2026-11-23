import { useCallback, useEffect, useRef, useState } from 'react';
import { ArrowDown, ChevronLeft, ChevronRight } from 'lucide-react';
import { Checkbox } from '@/components/ui/checkbox';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import { SkeletonRows } from '@/components/pending/SkeletonRows';
import { useKeyboard } from '@/lib/keyboard';
import { conversationsSettingsGet, type ConversationSessionBadge } from '@/services/conversations';
import { onLabelsChanged } from '@/services/events';
import { labelsList, labelsSelectAll, type LabelCounts, type LabelFilter, type LabelPage } from '@/services/labels';
import type { SessionResearch } from '@/services/research';
import { CONNECT_PRICING, connectionLine } from '@/features/conversations/connectionLine';
import { openSettingsSection } from '@/features/settings/SettingsNav';
import { usePictures } from '@/features/sessions/usePictures';
import { LabelRow } from './LabelRow';
import { LabelsBatchBar } from './LabelsBatchBar';
import { INITIAL_PAGES, LABELS_REFRESH_MS, askCursor, canNext, pageLabel, pageStep, type PageAction, type PageState } from './labelPages';
import { DENIED_HINT, workFromLine } from './labelWords';

const SUB_FILTERS: { value: LabelFilter; label: string; count: keyof LabelCounts }[] = [
  { value: 'missing_price_tag', label: 'Missing price tag', count: 'missing_price_tag' },
  { value: 'not_printed', label: 'Not printed', count: 'not_printed' },
  { value: 'printed', label: 'Printed', count: 'printed' },
  { value: 'all', label: 'All', count: 'total' },
];

const ALLOWED = { state: 'allowed', reason: null } as const;

type Failure = { code: string; message: string; data?: Record<string, string> };

/**
 * MC-L1.9 Refresh: runs `fn` at once, then at most once more per `ms` window (the calls inside a window are folded into
 * one at its end).
 */
export function useThrottled(fn: () => void, ms: number): () => void {
  const latest = useRef(fn);
  latest.current = fn;
  const timer = useRef<number | null>(null);
  const pending = useRef(false);
  useEffect(() => () => {
    if (timer.current !== null) window.clearTimeout(timer.current);
  }, []);
  return useCallback(() => {
    if (timer.current !== null) {
      pending.current = true;
      return;
    }
    latest.current();
    const open = () => {
      timer.current = window.setTimeout(() => {
        timer.current = null;
        if (!pending.current) return;
        pending.current = false;
        latest.current();
        open();
      }, ms);
    };
    open();
  }, [ms]);
}

/**
 * L2 (09C §5, MC-L1.9): the Labels box: the sub-filters with their counts, the work-from line, Pricing's rows 25 at a
 * time with Previous / Next through its cursors, the selection with Select all, and the refresh on `labels_changed`. The
 * search comes from the page (debounced). Entering, a new filter or search, and Try again ask the first page; a refresh
 * asks the page shown with its own cursor and keeps the rows, the page and the selection.
 */
export function LabelsBox({
  query,
  navigate,
  onNumber,
  research,
  conversations,
}: {
  /** The debounced, trimmed search ('' = none). */
  query: string;
  navigate: (sessionId: number) => void;
  /** The box's number: the last list answer's `missing_price_tag` (search applied). */
  onNumber: (count: number) => void;
  research: Map<number, SessionResearch>;
  conversations: Map<number, ConversationSessionBadge>;
}) {
  const [filter, setFilter] = useState<LabelFilter>('missing_price_tag');
  const [pages, setPages] = useState<PageState>(INITIAL_PAGES);
  const [page, setPage] = useState<LabelPage | null>(null);
  const [failure, setFailure] = useState<Failure | null>(null);
  const [offWords, setOffWords] = useState<string | null>(null);
  const [selection, setSelection] = useState<Map<string, string>>(() => new Map());
  const [allSelected, setAllSelected] = useState(false);
  const pictures = usePictures();
  const ticket = useRef(0);
  const pagesRef = useRef(pages);
  pagesRef.current = pages;
  const question = useRef({ filter, query });
  question.current = { filter, query };
  const numberRef = useRef(onNumber);
  numberRef.current = onNumber;

  function showFailure(error: Failure) {
    setFailure({ code: error.code, message: error.message, data: error.data });
    if (error.code !== 'labels_off') return;
    setOffWords(null);
    try {
      conversationsSettingsGet().then(
        settings => setOffWords(connectionLine(settings.connection) ?? CONNECT_PRICING),
        () => setOffWords(CONNECT_PRICING),
      );
    } catch {
      setOffWords(CONNECT_PRICING);
    }
  }

  /** Asks the page of `state`; an answer applies to the page state as it is when it arrives. */
  function ask(state: PageState, backToFirst = true) {
    const current = ++ticket.current;
    const { filter: asked, query: searched } = question.current;
    const cursor = askCursor(state);
    labelsList({ filter: asked, query: searched || null, cursor }).then(
      answer => {
        if (current !== ticket.current) return;
        const stepped = pageStep(pagesRef.current, { type: 'answer', index: state.index, nextCursor: answer.next_cursor, rows: answer.rows.length });
        pagesRef.current = stepped;
        setPages(stepped);
        if (stepped === INITIAL_PAGES && state.index > 0 && backToFirst) {
          ask(INITIAL_PAGES, false);
          return;
        }
        setPage(answer);
        setFailure(null);
        numberRef.current(answer.counts.missing_price_tag);
      },
      (error: Failure) => {
        if (current !== ticket.current) return;
        if (error.code === 'labels_rejected' && cursor !== null && backToFirst) {
          pagesRef.current = INITIAL_PAGES;
          setPages(INITIAL_PAGES);
          ask(INITIAL_PAGES, false);
          return;
        }
        showFailure(error);
      },
    );
  }
  const latestAsk = useRef(ask);
  latestAsk.current = ask;

  // Entering the box, a new filter or a new search: the first page, the selection cleared.
  useEffect(() => {
    pagesRef.current = INITIAL_PAGES;
    setPages(INITIAL_PAGES);
    setSelection(new Map());
    setAllSelected(false);
    latestAsk.current(INITIAL_PAGES);
  }, [filter, query]);

  const refresh = useThrottled(() => latestAsk.current(pagesRef.current), LABELS_REFRESH_MS);
  useEffect(() => {
    let alive = true;
    let unlisten: (() => void) | undefined;
    try {
      onLabelsChanged(() => refresh())
        .then(off => (alive ? (unlisten = off) : off()))
        .catch(() => undefined);
    } catch {
      // No listener: the box follows the next ask.
    }
    return () => {
      alive = false;
      unlisten?.();
    };
  }, [refresh]);

  function move(action: PageAction) {
    const next = pageStep(pages, action);
    if (next === pages) return;
    pagesRef.current = next;
    setPages(next);
    latestAsk.current(next);
  }

  /** Try again: the first page. */
  function tryAgain() {
    pagesRef.current = INITIAL_PAGES;
    setPages(INITIAL_PAGES);
    latestAsk.current(INITIAL_PAGES);
  }

  function clearSelection() {
    setSelection(new Map());
    setAllSelected(false);
  }
  useKeyboard({ escape: clearSelection }, selection.size > 0);

  const rows = page?.rows ?? [];
  const tickable = rows.filter(row => row.sku);
  const pageTicked = tickable.length > 0 && tickable.every(row => selection.has(row.product_id));
  const someTicked = tickable.some(row => selection.has(row.product_id));
  const counts = page?.counts ?? null;
  const filterCount = counts ? counts[SUB_FILTERS.find(f => f.value === filter)!.count] : 0;
  const offer = !allSelected && pageTicked && filterCount > selection.size ? filterCount : null;

  function toggle(productId: string, sku: string) {
    setAllSelected(false);
    setSelection(previous => {
      const next = new Map(previous);
      if (next.has(productId)) next.delete(productId);
      else next.set(productId, sku);
      return next;
    });
  }

  function togglePage() {
    setAllSelected(false);
    setSelection(previous => {
      const next = new Map(previous);
      for (const row of tickable) {
        if (pageTicked) next.delete(row.product_id);
        else next.set(row.product_id, row.sku!);
      }
      return next;
    });
  }

  function selectAll() {
    const { filter: asked, query: searched } = question.current;
    labelsSelectAll({ filter: asked, query: searched || null }).then(
      selected => {
        setSelection(new Map(selected.map(product => [product.product_id, product.sku])));
        setAllSelected(true);
      },
      (error: Failure) => showFailure(error),
    );
  }

  const blocked = failure && (failure.code === 'labels_off' || failure.code === 'labels_denied');
  const workFrom = workFromLine(page?.work_from);
  const now = Date.now();

  return (
    <section className="labels-box" aria-label="Labels">
      <div className="labels-toolbar">
        <ToggleGroup
          type="single"
          value={filter}
          onValueChange={next => next && setFilter(next as LabelFilter)}
          className="filters label-filters"
          aria-label="Label filter"
        >
          {SUB_FILTERS.map(sub => (
            <ToggleGroupItem key={sub.value} value={sub.value} className="filter-item" data-testid={`label-filter-${sub.value}`}>
              {sub.label}
              <span className="filter-count">{counts ? counts[sub.count] : ''}</span>
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>
      {workFrom && !blocked && <p className="labels-work-from">{workFrom}</p>}

      {blocked ? (
        <div className="empty-sessions labels-message">
          <strong>{failure.code === 'labels_denied' ? DENIED_HINT : offWords}</strong>
          <p>
            <button type="button" className="batch-link" onClick={() => openSettingsSection('pricing')}>
              Open Settings → Pricing
            </button>
          </p>
        </div>
      ) : (
        <>
          {failure && (
            <div className="labels-error" role="alert">
              <span>{failure.message}</span>
              <button type="button" className="batch-link" onClick={tryAgain}>
                Try again
              </button>
            </div>
          )}
          {!page && !failure ? (
            <div className="session-table" role="table" aria-label="Labels" aria-busy="true"><SkeletonRows n={6} /></div>
          ) : page && rows.length === 0 ? (
            <div className="empty-sessions">
              <strong>No products here</strong>
            </div>
          ) : page ? (
            <div className="session-table label-table" role="table" aria-label="Labels">
              <div className="list-labels label-labels" role="row">
                <span className="row-check">
                  <Checkbox
                    checked={pageTicked ? true : someTicked ? 'indeterminate' : false}
                    disabled={tickable.length === 0}
                    onCheckedChange={togglePage}
                    aria-label="Select the products of this page"
                  />
                </span>
                <span>Item</span>
                <span>Price</span>
                <span>Label</span>
                <span className="label-activity">
                  Created <ArrowDown aria-hidden />
                </span>
                <span aria-hidden />
              </div>
              {rows.map(row => (
                <LabelRow
                  key={row.product_id}
                  row={row}
                  now={now}
                  selected={selection.has(row.product_id)}
                  onToggle={() => row.sku && toggle(row.product_id, row.sku)}
                  navigate={navigate}
                  pictures={pictures}
                  research={row.session ? research.get(row.session.id) : undefined}
                  conversations={row.session ? conversations.get(row.session.id) : undefined}
                  labelsAccess={ALLOWED}
                  onChanged={refresh}
                />
              ))}
            </div>
          ) : null}
          {page && (
            <nav className="pagination label-pager" aria-label="Label pages">
              <button type="button" className="page-step" disabled={pages.index === 0} onClick={() => move({ type: 'previous' })}>
                <ChevronLeft aria-hidden /> Previous
              </button>
              <span className="label-page">{pageLabel(pages.index, filterCount)}</span>
              <button type="button" className="page-step" disabled={!canNext(pages)} onClick={() => move({ type: 'next' })}>
                Next <ChevronRight aria-hidden />
              </button>
            </nav>
          )}
        </>
      )}

      <LabelsBatchBar count={selection.size} all={allSelected} offer={offer} onSelectAll={selectAll} onClear={clearSelection} />
    </section>
  );
}
