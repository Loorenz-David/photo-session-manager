import { vi } from 'vitest';
import type { listSessions as ListSessions, sessionsPage as SessionsPage, SessionList, SessionPage } from '@/services/sessions';

type PageArgs = Parameters<typeof SessionsPage>[0];

/**
 * H2: the page the backend answers for `list` (MC-H2.1) when every row fits on one page: the time filter and
 * "To review" applied, each group with its row count. Lets the Sessions-page tests keep their `listSessions` answers.
 */
export function pageFromList(list: SessionList, args: PageArgs): SessionPage {
  const groups = list.groups
    .map(group => {
      const rows = group.rows.filter(
        row => (args.created_from === null || row.created_at >= args.created_from) && (args.only_ids === null || args.only_ids.includes(row.id)),
      );
      return { label: group.label, total: rows.length, rows };
    })
    .filter(group => group.rows.length > 0);
  const total = groups.reduce((sum, group) => sum + group.rows.length, 0);
  return { groups, counts: list.counts, unassigned_count: list.unassigned_count, page: 1, pages: 1, total };
}

/** H2: the mocked `sessionsPage` answers from the mocked `listSessions` (called with the page's filter and query). */
export function answerFromList(sessionsPage: typeof SessionsPage, listSessions: typeof ListSessions) {
  vi.mocked(sessionsPage).mockImplementation(async args => pageFromList(await listSessions({ filter: args.filter, query: args.query }), args));
}
