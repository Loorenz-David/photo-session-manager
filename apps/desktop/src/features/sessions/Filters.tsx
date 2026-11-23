import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import type { SessionFilter, SessionListCountsResponse } from '@/services/bindings';

/** The boxes of the sessions list: the three session filters and, L2 (09C §5), the Labels box. */
export type ListBox = SessionFilter | 'labels';

const FILTERS: { value: SessionFilter; label: string }[] = [
  { value: 'open', label: 'Open' },
  { value: 'published', label: 'Published' },
  { value: 'all', label: 'All' },
];

/**
 * Open (default, everything not published) / Published / All, with the backend's counts, then Labels (L2) with the
 * number the page gives it (empty when none).
 */
export function Filters({
  value,
  counts,
  labelsCount = null,
  onChange,
}: {
  value: ListBox;
  counts: SessionListCountsResponse | null;
  labelsCount?: number | null;
  onChange: (filter: ListBox) => void;
}) {
  return (
    <ToggleGroup
      type="single"
      value={value}
      onValueChange={next => next && onChange(next as ListBox)}
      className="filters"
      aria-label="Session filter"
    >
      {FILTERS.map(filter => (
        <ToggleGroupItem key={filter.value} value={filter.value} className="filter-item" data-testid={`filter-${filter.value}`}>
          {filter.label}
          <span className="filter-count" data-testid={`count-${filter.value}`}>
            {counts ? counts[filter.value] : ''}
          </span>
        </ToggleGroupItem>
      ))}
      <ToggleGroupItem value="labels" className="filter-item" data-testid="filter-labels">
        Labels
        <span className="filter-count" data-testid="count-labels">
          {labelsCount ?? ''}
        </span>
      </ToggleGroupItem>
    </ToggleGroup>
  );
}
