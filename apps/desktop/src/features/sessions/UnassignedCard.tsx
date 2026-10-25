import { ChevronRight } from 'lucide-react';

/** Shown on top of the list only while unassigned photos exist (ch04 S-E); opens the Unassigned workspace. */
export function UnassignedCard({ count, onOpen }: { count: number; onOpen: () => void }) {
  if (count <= 0) return null;
  return (
    <button type="button" className="unassigned-card" onClick={onOpen} data-testid="unassigned-card">
      <span className="unassigned-text">
        <strong>Unassigned photos</strong>
        <small>
          <span data-testid="unassigned-count">{count}</span> {count === 1 ? 'photo' : 'photos'} taken without an item
        </small>
      </span>
      <ChevronRight aria-hidden />
    </button>
  );
}
