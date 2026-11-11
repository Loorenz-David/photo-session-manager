import { Check, ScanSearch, Undo2, X } from 'lucide-react';
import type { Photo } from '@/services/photos';

type ReviewButtonsProps = {
  photo: Photo;
  onApprove: () => void;
  onMiddle: () => void;
  onDelete: () => void;
};

/**
 * Round Deny / Condition (or "Move to Main" on a condition photo) buttons (ch06 §Classification, changelog 46 (2)):
 * photos arrive approved, so Approve is offered only for a photo still unreviewed from before.
 */
export function ReviewButtons({ photo, onApprove, onMiddle, onDelete }: ReviewButtonsProps) {
  const isCondition = photo.classification === 'condition';
  return (
    <div className="review-buttons">
      <button type="button" className="review-button review-delete" onClick={onDelete} aria-label="Deny" title="Deny (⌫)">
        <X />
      </button>
      <button
        type="button"
        className="review-button review-middle"
        onClick={onMiddle}
        aria-label={isCondition ? 'Move to Main' : 'Move to Condition'}
        title={isCondition ? 'Move to Main (A)' : 'Move to Condition (C)'}
      >
        {isCondition ? <Undo2 /> : <ScanSearch />}
      </button>
      {photo.classification === 'unreviewed' && (
        <button type="button" className="review-button review-approve" onClick={onApprove} aria-label="Approve" title="Approve as main image (A)">
          <Check />
        </button>
      )}
    </div>
  );
}
