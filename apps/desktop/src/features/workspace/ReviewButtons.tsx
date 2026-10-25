import { Check, ScanSearch, Undo2, X } from 'lucide-react';
import type { Photo } from '@/services/photos';

type ReviewButtonsProps = {
  photo: Photo;
  onApprove: () => void;
  onMiddle: () => void;
  onDelete: () => void;
};

/** Round Delete / Condition (or "Move to Main" on a condition photo) / Approve buttons (ch06 §Classification). */
export function ReviewButtons({ photo, onApprove, onMiddle, onDelete }: ReviewButtonsProps) {
  const isCondition = photo.classification === 'condition';
  return (
    <div className="review-buttons">
      <button type="button" className="review-button review-delete" onClick={onDelete} aria-label="Delete" title="Delete (⌫)">
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
      <button type="button" className="review-button review-approve" onClick={onApprove} aria-label="Approve" title="Approve as main image (A)">
        <Check />
      </button>
    </div>
  );
}
