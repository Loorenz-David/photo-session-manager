import { Contrast, Loader2 } from 'lucide-react';
import type { ViewMode } from './useWorkspaceState';

type TogglePillProps = { mode: ViewMode; processing: boolean; onToggle: () => void };

/** Original/Adjusted pill (ch06): one per session view, Space toggles; a spinner while the adjusted render is pending. */
export function TogglePill({ mode, processing, onToggle }: TogglePillProps) {
  return (
    <button
      type="button"
      className="toggle-pill"
      aria-pressed={mode === 'original'}
      aria-label={mode === 'adjusted' ? 'Showing adjusted, switch to original (Space)' : 'Showing original, switch to adjusted (Space)'}
      onClick={onToggle}
    >
      {processing ? <Loader2 className="toggle-spinner" role="status" aria-label="Processing" /> : <Contrast aria-hidden />}
      <span>{mode === 'adjusted' ? 'Adjusted' : 'Original'}</span>
    </button>
  );
}
