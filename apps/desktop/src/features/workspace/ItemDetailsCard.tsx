import { X } from 'lucide-react';
import type { SessionView } from '@/services/photos';
import { SkeletonFields } from '@/components/pending/SkeletonFields';
import { Hint } from '@/components/ui/tooltip';
import { Checkbox } from '@/components/ui/checkbox';

type ItemDetailsCardProps = {
  view: SessionView | null;
  onClose: () => void;
  onEdit?: () => void;
  /** "Keep files on this Mac" (10F Q2): shown for an item when a handler is given. */
  keepFiles?: boolean;
  onKeepFilesChange?: (keep: boolean) => void;
};

/** Item details card (I): the item's identity and the session's photo counts; editing arrives with the form. */
export function ItemDetailsCard({ view, onClose, onEdit, keepFiles = false, onKeepFilesChange }: ItemDetailsCardProps) {
  if (!view) return <div className="item-card" role="dialog" aria-label="Item details"><SkeletonFields n={4} /></div>;
  const { item, session } = view;
  return (
    <div className="item-card" role="dialog" aria-label="Item details">
      <div className="item-card-head">
        <strong>{item ? item.item_type || item.title : 'Unassigned photos'}</strong>
        <button type="button" className="stage-icon-button" onClick={onClose} aria-label="Close item details">
          <X />
        </button>
      </div>
      <dl className="hover-props">
        {item && (
          <>
            <dt>Title</dt>
            <dd>{item.title}</dd>
            <dt>SKU</dt>
            <dd>{item.sku}</dd>
            <dt>Barcode</dt>
            <dd>{item.barcode ?? '—'}</dd>
            <dt>Type</dt>
            <dd>{item.item_type || '—'}</dd>
          </>
        )}
        <dt>Photos</dt>
        <dd>
          {session.main_count} main · {session.condition_count} condition · {session.unreviewed_count} to review
        </dd>
      </dl>
      {item && onKeepFilesChange && (
        <label className="keep-files">
          <Checkbox checked={keepFiles} onCheckedChange={checked => onKeepFilesChange(checked === true)} aria-label="Keep files on this Mac" />
          Keep files on this Mac
        </label>
      )}
      <Hint text="Edit details"><button type="button" className="item-card-edit" onClick={onEdit}>
        Edit details →
      </button></Hint>
    </div>
  );
}
