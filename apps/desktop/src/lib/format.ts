const MINUTE = 60_000;
const DAY = 86_400_000;

function startOfDay(ms: number): number {
  const day = new Date(ms);
  day.setHours(0, 0, 0, 0);
  return day.getTime();
}

const clock = new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', hourCycle: 'h23' });
const weekday = new Intl.DateTimeFormat(undefined, { weekday: 'short' });
const dayMonth = new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short' });

/** Last-activity label: "27 min ago" within the hour, "10:45" today, "Fri 11:45" this week, "12 Oct" before. */
export function activityLabel(ms: number, now = Date.now()): string {
  const minutes = Math.floor((now - ms) / MINUTE);
  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes} min ago`;
  if (ms >= startOfDay(now)) return clock.format(ms);
  if (ms >= startOfDay(now) - 6 * DAY) return `${weekday.format(ms)} ${clock.format(ms)}`;
  return dayMonth.format(ms);
}

export type TimeRange = 'any' | 'today' | 'last7' | 'last30';

export const TIME_RANGES: { value: TimeRange; label: string }[] = [
  { value: 'any', label: 'Any time' },
  { value: 'today', label: 'Today' },
  { value: 'last7', label: 'Last 7 days' },
  { value: 'last30', label: 'Last 30 days' },
];

/** Calendar-day based (like MC-4): "Last 7 days" = today and the six local days before it. */
export function withinRange(ms: number, range: TimeRange, now = Date.now()): boolean {
  if (range === 'any') return true;
  const daysBack = range === 'today' ? 0 : range === 'last7' ? 6 : 29;
  const start = new Date(startOfDay(now));
  start.setDate(start.getDate() - daysBack);
  return ms >= start.getTime();
}

export function photoSummary(main: number, condition: number): string {
  return `${main} main · ${condition} condition`;
}
