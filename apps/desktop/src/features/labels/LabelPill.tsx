import type { SessionLabel } from '@/services/labels';
import { markerText } from './labelWords';

/**
 * MC-L3.6: the label marker, a pill in a session row's pill line and beside the session page's title; nothing when there
 * is no marker. Printed reads neutral; missing, stale and no price read as needing attention.
 */
export function LabelPill({ label, now }: { label?: SessionLabel | null; now: number }) {
  const text = markerText(label?.marker, now);
  if (!text) return null;
  const tone = label?.marker?.status === 'printed' ? 'printed' : 'attention';
  return <span className="pill label-pill" data-tone={tone} data-testid="label-pill">{text}</span>;
}
