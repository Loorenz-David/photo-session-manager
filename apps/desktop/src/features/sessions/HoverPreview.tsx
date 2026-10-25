import type { ReactNode } from 'react';
import { HoverCard, HoverCardContent, HoverCardTrigger } from '@/components/ui/hover-card';
import { photoSummary } from '@/lib/format';
import { previewUrl, type SessionList } from '@/services/sessions';

type Row = SessionList['groups'][number]['rows'][number];

export function Thumb({ path, className }: { path: string | null; className: string }) {
  return <span className={className}>{path ? <img src={previewUrl(path)} alt="" loading="lazy" /> : null}</span>;
}

/**
 * Large cover preview with caption and properties when hovering a row's thumbnail or name.
 * Radix positions it to the right and flips it to the left near the window edge.
 */
export function HoverPreview({ row, children }: { row: Row; children: ReactNode }) {
  return (
    <HoverCard openDelay={350} closeDelay={80}>
      <HoverCardTrigger asChild>{children}</HoverCardTrigger>
      <HoverCardContent side="right" align="start" sideOffset={16} collisionPadding={16} className="hover-preview">
        <Thumb path={row.cover_preview_path} className="hover-image" />
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
