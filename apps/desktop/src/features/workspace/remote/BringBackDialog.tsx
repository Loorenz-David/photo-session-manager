import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogTitle } from '@/components/ui/dialog';
import type { RemoteView } from '@/services/remote';

/** `About <X> MB`: the estimate rounded to a whole number, at least 1 (master plan archive §4 Words). */
export function megabytes(bytes: number): number {
  return Math.max(1, Math.round(bytes / 1_000_000));
}

export function bringBackQuestion(view: RemoteView): string {
  const n = view.pictures.length;
  const mb = megabytes(view.bring_back_bytes);
  return n === 1
    ? `Bring back 1 photo from Shopify to edit it? About ${mb} MB.`
    : `Bring back ${n} photos from Shopify to edit them? About ${mb} MB.`;
}

type Phase = { kind: 'ask' } | { kind: 'running' } | { kind: 'failed'; message: string };

/**
 * H4 (10H §6, §7): the question before the first photo edit of a remote product, then the progress
 * (`Bringing back <k> of <N> photos…`, from the progress events), then closed on success (the workspace reloads as an
 * ordinary session) or the reason and **Try again**. Opened while a bring back already runs, it shows its progress and
 * starts nothing (MC-H4.1).
 */
export function BringBackDialog({ open, view, onOpenChange, bringBack }: {
  open: boolean;
  view: RemoteView;
  onOpenChange: (open: boolean) => void;
  bringBack: () => Promise<void>;
}) {
  const [phase, setPhase] = useState<Phase>({ kind: 'ask' });
  const running = view.bring_back !== null;
  useEffect(() => {
    if (open) setPhase(running ? { kind: 'running' } : { kind: 'ask' });
    // Only when the dialog opens: a progress event while it is open must not reset a failure.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const start = () => {
    setPhase({ kind: 'running' });
    bringBack()
      .then(() => onOpenChange(false))
      .catch(error => setPhase({ kind: 'failed', message: (error as Error).message }));
  };

  const total = view.bring_back?.total || view.pictures.length;
  const done = view.bring_back?.done ?? 0;
  return (
    <Dialog open={open} onOpenChange={next => { if (!next && phase.kind !== 'running') onOpenChange(false); }}>
      <DialogContent showCloseButton={false}>
        <DialogTitle>Bring back the photos</DialogTitle>
        {phase.kind === 'ask' && <>
          <DialogDescription>{bringBackQuestion(view)}</DialogDescription>
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
            <Button onClick={start}>Bring back</Button>
          </DialogFooter>
        </>}
        {phase.kind === 'running' && <DialogDescription aria-live="polite">{`Bringing back ${done} of ${total} photos…`}</DialogDescription>}
        {phase.kind === 'failed' && <>
          <DialogDescription role="alert">{phase.message}</DialogDescription>
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
            <Button onClick={start}>Try again</Button>
          </DialogFooter>
        </>}
      </DialogContent>
    </Dialog>
  );
}
