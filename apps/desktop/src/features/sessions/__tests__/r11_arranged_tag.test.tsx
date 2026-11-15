import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import type { SessionList } from '@/services/sessions';

vi.mock('@/features/sessions/HoverPreview', () => ({ HoverPreview: ({ children }: { children: React.ReactNode }) => children, Thumb: () => null }));
vi.mock('@/features/sessions/StatusPill', () => ({ StatusPill: () => null }));

import { SessionRow } from '../SessionRow';

type Row = SessionList['groups'][number]['rows'][number];
function row(arranged: boolean): Row {
  return { id: 5, sku: 'SKU', item_type: 'Chair', title: 'Chair', main_count: 1, condition_count: 0,
    unreviewed_count: 0, status: 'ready', label: { kind: 'ready' }, progress: null,
    last_activity_at: 1, created_at: 1, cover_preview_path: null, publish: null, arranged } as Row;
}
afterEach(cleanup);

it('a25_c25a_arranged_tag_appears_only_on_marked_row', () => {
  const one = render(<SessionRow row={row(true)} selected={false} onToggle={() => undefined} onOpen={() => undefined} now={1} />);
  expect(screen.getByText('Arranged')).toBeInTheDocument();
  one.unmount();
  render(<SessionRow row={row(false)} selected={false} onToggle={() => undefined} onOpen={() => undefined} now={1} />);
  expect(screen.queryByText('Arranged')).not.toBeInTheDocument();
});
