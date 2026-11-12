import { useCallback, useEffect, useState, type ReactNode } from 'react';
import { Progress } from '@/components/ui/progress';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { publishActivity, publishCancel, type PublishActivityDto } from '@/services/publish';

/** `m:ss` (or `h:mm:ss`) of an elapsed time in milliseconds. */
export function formatElapsed(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = String(total % 60).padStart(2, '0');
  return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${s}` : `${m}:${s}`;
}

const RETRY_TEXT: Record<string, string> = {
  retrying: 'Retrying after this error',
  retry_available: 'Stopped · press Retry to continue',
};

/** P64: what the publish is doing now, its elapsed time and last error; Cancel publish with a confirmation. */
export function PublishActivityPanel({ sessionId }: { sessionId: number }) {
  const [activity, setActivity] = useState<PublishActivityDto | null>(null);
  const [confirming, setConfirming] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const load = useCallback(async () => {
    try { setActivity(await publishActivity(sessionId)); } catch { /* keep the last reading */ }
  }, [sessionId]);
  useEffect(() => {
    void load();
    const timer = window.setInterval(() => void load(), 1000);
    return () => window.clearInterval(timer);
  }, [load]);
  if (!activity) return <p className="text-muted-foreground">Reading the publish…</p>;
  const cancel = async () => {
    setCancelling(true);
    try { await publishCancel(activity.item_id); setConfirming(false); await load(); } finally { setCancelling(false); }
  };
  return (
    <div className="publish-activity" data-testid="publish-activity">
      <p className="font-medium">{activity.text}</p>
      <p className="text-muted-foreground">{activity.percent}% · Elapsed {formatElapsed(activity.elapsed_ms)}</p>
      {activity.masters_ready && <p className="text-muted-foreground">{activity.masters_ready}</p>}
      {activity.error && (
        <div role="alert">
          <p>Last error: {activity.error}</p>
          {activity.retry && <p className="text-muted-foreground">{RETRY_TEXT[activity.retry] ?? activity.retry}</p>}
        </div>
      )}
      {activity.can_cancel && !confirming && (
        <button type="button" className="pill pill-action" onClick={() => setConfirming(true)}>Cancel publish</button>
      )}
      {confirming && (
        <div role="group" aria-label="Confirm cancel">
          <p>Stop this publish? Photos already on Shopify stay there; the next Publish continues from them.</p>
          <button type="button" className="pill" onClick={() => setConfirming(false)} disabled={cancelling}>Keep publishing</button>
          <button type="button" className="pill pill-failed" onClick={() => void cancel()} disabled={cancelling}>Stop publish</button>
        </div>
      )}
    </div>
  );
}

/** Wraps a publish progress bar: click or hover opens the activity popover (P64). Without a session it is the bar alone. */
export function PublishActivityPopover({ sessionId, children }: { sessionId?: number; children: ReactNode }) {
  const [open, setOpen] = useState(false);
  if (sessionId === undefined) return <>{children}</>;
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button type="button" className="publish-progress-trigger" aria-label="Publish progress details" onMouseEnter={() => setOpen(true)}>
          {children}
        </button>
      </PopoverTrigger>
      <PopoverContent align="end">{open && <PublishActivityPanel sessionId={sessionId} />}</PopoverContent>
    </Popover>
  );
}

export function PublishProgress({ value, sessionId }: { value: number; sessionId?: number }) {
  return (
    <PublishActivityPopover sessionId={sessionId}>
      <div role="status" className="min-w-28"><span>Publishing {value}%</span><Progress value={value} aria-label={`Publishing ${value}%`} /></div>
    </PublishActivityPopover>
  );
}
