import { useState } from 'react';
import { Check, SlidersHorizontal } from 'lucide-react';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { TIME_RANGES, type TimeRange } from '@/lib/format';

/** Time filter popover inside the search field; a red dot marks an active filter. */
export function TimeFilter({ value, onChange }: { value: TimeRange; onChange: (range: TimeRange) => void }) {
  const [open, setOpen] = useState(false);
  function pick(range: TimeRange) {
    onChange(range);
    setOpen(false);
  }
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button type="button" className="time-trigger" aria-label="Filter by last activity" title="Filter by last activity">
          <SlidersHorizontal aria-hidden />
          {value !== 'any' && <span className="time-active" aria-hidden />}
        </button>
      </PopoverTrigger>
      <PopoverContent align="end" className="time-menu">
        <p className="menu-caption">Last activity</p>
        {TIME_RANGES.map(range => (
          <button key={range.value} type="button" className="menu-option" aria-pressed={value === range.value} onClick={() => pick(range.value)}>
            {range.label}
            {value === range.value && <Check aria-hidden />}
          </button>
        ))}
        <button type="button" className="menu-reset" onClick={() => pick('any')} disabled={value === 'any'}>
          Reset
        </button>
      </PopoverContent>
    </Popover>
  );
}
