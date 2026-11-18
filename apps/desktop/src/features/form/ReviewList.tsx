export type { ReviewEntry } from './AiReviewTab';

type ReviewListProps = {
  /** Fields holding an unchecked AI suggestion. */
  count: number;
  /** Opens the AI review tab. */
  onOpen: () => void;
};

/**
 * "n to review" (A6, owner 2026-11-17). A11 (owner 2026-11-18): the chip opens the AI review tab, where every
 * suggestion can be confirmed in place, instead of a list that sent each field to its own step.
 */
export function ReviewList({ count, onOpen }: ReviewListProps) {
  if (count === 0) return null;
  return (
    <button type="button" data-to-review onClick={onOpen} className="rounded-full bg-(--ai-muted) px-2.5 py-0.5 text-xs font-semibold text-(--ai-foreground)">
      {count} to review
    </button>
  );
}
