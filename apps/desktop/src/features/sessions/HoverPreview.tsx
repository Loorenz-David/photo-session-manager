import type { ReactNode } from 'react';
import { ImageIcon } from 'lucide-react';
import { HoverCard, HoverCardContent, HoverCardTrigger } from '@/components/ui/hover-card';
import { photoSummary } from '@/lib/format';
import { previewUrl, type SessionList } from '@/services/sessions';

type Row = SessionList['groups'][number]['rows'][number];

/**
 * A row picture, or a neutral placeholder when there is none (H2, M-H4: the backend only hands out paths whose file
 * exists, so a row never shows a broken picture).
 */
export function Thumb({ path, className }: { path: string | null | undefined; className: string }) {
  if (!path) {
    return (
      <span className={`${className} row-thumb-empty`} data-testid="thumb-placeholder">
        <ImageIcon aria-hidden />
      </span>
    );
  }
  return (
    <span className={className}>
      <img src={previewUrl(path)} alt="" loading="lazy" />
    </span>
  );
}

/**
 * Large cover preview with caption and properties when hovering a row's thumbnail or name.
 * Radix positions it to the right and flips it to the left near the window edge. `picture` is the row's picture as the
 * row shows it (H2: a cover that arrived after the list).
 */
export function HoverPreview({ row, picture, children }: { row: Row; picture?: string | null; children: ReactNode }) {
  return (
    <HoverCard openDelay={350} closeDelay={80}>
      <HoverCardTrigger asChild>{children}</HoverCardTrigger>
      <HoverCardContent side="right" align="start" sideOffset={16} collisionPadding={16} className="hover-preview">
        <Thumb path={picture === undefined ? row.cover_preview_path : picture} className="hover-image" />
        <strong>{row.item_type || row.title}</strong>
        <dl className="hover-props">
          <dt>SKU</dt>
          <dd>{row.sku}</dd>
          {row.item_type && row.title && (
            <>
              <dt>Title</dt>
              <dd>{row.title}</dd>
            </>
          )}
          <dt>Photos</dt>
          <dd>{photoSummary(row.main_count, row.condition_count)}</dd>
        </dl>
      </HoverCardContent>
    </HoverCard>
  );
}
