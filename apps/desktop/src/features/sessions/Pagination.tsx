import { ChevronLeft, ChevronRight } from 'lucide-react';

/**
 * H2 (10H §5): the page numbers to show: every page up to 7, otherwise the first, the last and the current one with
 * its neighbours, with `null` for a gap.
 */
export function pageNumbers(page: number, pages: number): (number | null)[] {
  if (pages <= 7) return Array.from({ length: pages }, (_, i) => i + 1);
  const shown = [...new Set([1, page - 1, page, page + 1, pages])].filter(n => n >= 1 && n <= pages).sort((a, b) => a - b);
  const out: (number | null)[] = [];
  shown.forEach((n, i) => {
    if (i > 0 && n - shown[i - 1] > 1) out.push(null);
    out.push(n);
  });
  return out;
}

/** H2 (10H §5): the page control under the sessions list (shown only with more than one page). */
export function Pagination({ page, pages, onPage }: { page: number; pages: number; onPage: (page: number) => void }) {
  if (pages <= 1) return null;
  return (
    <nav className="pagination" aria-label="Pages">
      <button type="button" className="page-step" disabled={page <= 1} onClick={() => onPage(page - 1)} aria-label="Previous page">
        <ChevronLeft aria-hidden />
      </button>
      {pageNumbers(page, pages).map((n, i) =>
        n === null ? (
          <span key={`gap-${i}`} className="page-gap" aria-hidden>
            …
          </span>
        ) : (
          <button
            key={n}
            type="button"
            className="page-number"
            aria-current={n === page ? 'page' : undefined}
            aria-label={`Page ${n}`}
            onClick={() => onPage(n)}
          >
            {n}
          </button>
        ),
      )}
      <button type="button" className="page-step" disabled={page >= pages} onClick={() => onPage(page + 1)} aria-label="Next page">
        <ChevronRight aria-hidden />
      </button>
    </nav>
  );
}
