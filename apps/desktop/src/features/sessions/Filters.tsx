import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import type { SessionFilter, SessionListCountsResponse } from '@/services/bindings';

const FILTERS: { value: SessionFilter; label: string }[] = [
  { value: 'open', label: 'Open' },
  { value: 'published', label: 'Published' },
  { value: 'all', label: 'All' },
];

/** Open (default, everything not published) / Published / All, with the backend's counts. */
export function Filters({
  value,
  counts,
  onChange,
}: {
  value: SessionFilter;
  counts: SessionListCountsResponse | null;
  onChange: (filter: SessionFilter) => void;
}) {
  return (
    <ToggleGroup
      type="single"
      value={value}
      onValueChange={next => next && onChange(next as SessionFilter)}
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
    </ToggleGroup>
  );
}
