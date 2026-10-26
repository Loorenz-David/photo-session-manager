import { CloudUpload, RotateCw } from 'lucide-react';
import { Progress } from '@/components/ui/progress';
import { useAction } from '@/lib/pending';
import type { RowLabelResponse } from '@/services/bindings';

type StatusPillProps = {
  label: RowLabelResponse;
  /** Publish actions are wired with publishing (P15); without a handler the buttons are shown disabled. */
  onRetry?: () => Promise<unknown> | void;
  onPublish?: () => void;
};

const LATER = 'Arrives with publishing';

/** Row status in the order of ch07: Publishing n%, Publish failed, Description failed, Published, Ready · Publish, In progress. */
export function StatusPill({ label, onRetry, onPublish }: StatusPillProps) {
  const retry = useAction(async () => { await onRetry?.(); });
  switch (label.kind) {
    case 'publishing':
      return (
        <div className="status-cell">
          <span className="pill pill-progress">Publishing {label.value}%</span>
          <Progress value={label.value} aria-label={`Publishing ${label.value}%`} className="pill-bar" />
        </div>
      );
    case 'publish_failed':
      return (
        <div className="status-cell status-row">
          <span className="pill pill-failed">Publish failed</span>
          <button type="button" className="pill pill-action" onClick={() => void retry.run()} disabled={!onRetry || retry.busy} aria-busy={retry.busy ? 'true' : undefined} title={onRetry ? 'Retry' : LATER}>
            <RotateCw aria-hidden /> Retry{retry.busy ? '…' : ''}
          </button>
        </div>
      );
    case 'description_failed':
      return (
        <div className="status-cell status-row">
          <span className="pill pill-failed">Description failed</span>
          {onRetry && <button type="button" className="pill pill-action" onClick={() => void retry.run()} disabled={retry.busy} aria-busy={retry.busy ? 'true' : undefined}><RotateCw aria-hidden /> Retry{retry.busy ? '…' : ''}</button>}
        </div>
      );
    case 'published':
      return (
        <div className="status-cell">
          <span className="pill pill-published">Published</span>
        </div>
      );
    case 'ready':
      return (
        <div className="status-cell">
          <button type="button" className="pill pill-ready" onClick={onPublish} disabled={!onPublish} title={onPublish ? 'Publish' : LATER}>
            <CloudUpload aria-hidden /> Ready · Publish
          </button>
        </div>
      );
    case 'in_progress':
      return (
        <div className="status-cell">
          <span className="pill">In progress</span>
          {label.value > 0 && <span className="to-review">{label.value} to review</span>}
        </div>
      );
  }
}
