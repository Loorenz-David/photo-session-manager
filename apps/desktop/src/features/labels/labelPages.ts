/**
 * L2 (09C §5 Pages; master plan labels MC-L1.9): the Labels box's page state, changed only by `pageStep`. The cursor
 * stack holds `cursors[0] = null` (the first page) and `cursors[k + 1]` = the `next_cursor` of the answer for page `k`,
 * so Previous reuses the cursors already seen and Next is offered only when the page shown has a next cursor.
 */

/** Rows per page of the Labels box (the service's `LIST_PAGE`). */
export const LABELS_PAGE = 25;
/** A refresh of the box runs at most once per this many ms (the first at once, the rest folded into one more). */
export const LABELS_REFRESH_MS = 2000;

export type PageState = { cursors: (string | null)[]; index: number };

export type PageAction =
  | { type: 'answer'; index: number; nextCursor: string | null; rows: number }
  | { type: 'next' }
  | { type: 'previous' }
  | { type: 'reset' };

export const INITIAL_PAGES: PageState = { cursors: [null], index: 0 };

/**
 * MC-L1.9: an answer for the page shown keeps the cursors up to it and sets the next one (dropping the later ones); an
 * answer for another page is ignored; an empty answer for a page after the first goes back to the first page. Next and
 * Previous move only where a cursor is known; `reset` is the first page. An action that changes nothing answers the
 * same state.
 */
export function pageStep(state: PageState, action: PageAction): PageState {
  switch (action.type) {
    case 'answer': {
      if (action.index !== state.index) return state;
      if (action.rows === 0 && action.index > 0) return INITIAL_PAGES;
      const cursors = state.cursors.slice(0, state.index + 1);
      if (typeof action.nextCursor === 'string') cursors.push(action.nextCursor);
      return { cursors, index: state.index };
    }
    case 'next':
      return canNext(state) ? { cursors: state.cursors, index: state.index + 1 } : state;
    case 'previous':
      return state.index > 0 ? { cursors: state.cursors, index: state.index - 1 } : state;
    case 'reset':
      return INITIAL_PAGES;
  }
}

/** The cursor the page shown is asked with (`null` = the first page). */
export function askCursor(state: PageState): string | null {
  return state.cursors[state.index] ?? null;
}

/** Whether Next is offered: the page shown has a next cursor. */
export function canNext(state: PageState): boolean {
  return typeof state.cursors[state.index + 1] === 'string';
}

/** `Page <index + 1> of <n>`, `n` = the pages of the filter's count, never below the page shown (no page buttons). */
export function pageLabel(index: number, count: number): string {
  const pages = Math.max(index + 1, Math.ceil(count / LABELS_PAGE), 1);
  return `Page ${index + 1} of ${pages}`;
}
