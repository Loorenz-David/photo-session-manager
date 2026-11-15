import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { SkeletonRows } from '@/components/pending/SkeletonRows';
import { useAction, usePageData } from '@/lib/pending';
import { previewUrl } from '@/services/sessions';
import { onResearchChanged } from '@/services/events';
import { SUGGESTIONS_CHANGED, confirmedText, publishConfirm, publishPreview, shownOf, withheldLine, type PreviewDto } from '@/services/publish';
import { ConflictList } from './ConflictList';

/** A refused confirmation: `needs_review` carries the backend's own naming of the photos ("Re-frame first: Main 02", MC-27 §Revisions); the popover stays open. */
function publishFailureText(error: unknown): string {
  const message = (error as Error).message;
  return (error as { code?: string }).code === 'needs_review' ? message : `Could not publish: ${message}`;
}

/**
 * The publish confirmation surface (07A MC-10). `modal` renders it as a dialog of its own, for callers that sit inside
 * another modal (the batch queue's form dialog): mounted in that dialog's tree, it stacks as the top layer.
 */
export function PublishPopover({ sessionIds, onClose, onConfirmed, modal = false }: { sessionIds: number[]; onClose: () => void; onConfirmed?: () => void; modal?: boolean }) {
  const key = sessionIds.join(',');
  const { data: preview, error, reload } = usePageData<PreviewDto>(() => publishPreview(sessionIds), [key]);
  useEffect(() => {
    let alive = true;
    let stop: (() => void) | undefined;
    onResearchChanged(id => { if (sessionIds.includes(id)) void reload(); })
      .then(unlisten => { if (alive) stop = unlisten; else unlisten(); })
      .catch(() => undefined);
    return () => { alive = false; stop?.(); };
    // `key` is the selection identity; reload is stable in usePageData.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, reload]);
  // MC-A10: the suggestions changed while the photographer was confirming; the popover says so and rebuilds.
  const [changed, setChanged] = useState<string | null>(null);
  useEffect(() => { if (error) toast.error(`Could not preview publish: ${(error as Error).message}`); }, [error]);
  const count = preview?.items.reduce((n, item) => n + item.main.length + item.condition.length, 0) ?? 0;
  const confirm = useAction(async () => {
    if (!preview) return;
    try {
      await publishConfirm(sessionIds, confirmedText(preview), shownOf(preview));
    } catch (failure) {
      if ((failure as { code?: string }).code !== SUGGESTIONS_CHANGED) throw failure;
      setChanged((failure as Error).message);
      await reload();
      return;
    }
    (onConfirmed ?? onClose)();
  }, error => toast.error(publishFailureText(error)));
  const body = !preview ? <SkeletonRows n={3} /> : <>
      {preview.items.map(item => <div key={item.session_id} data-session={item.session_id} className="space-y-2 py-2">
        <p>Main images</p><div className="flex gap-1">{item.main_thumbnails.map((path, index) => path ? <img key={item.main[index]} src={previewUrl(path)} alt={`Main ${index + 1}`} className="size-12 object-cover" /> : <span key={item.main[index]}>Main {index + 1}</span>)}</div>
        <p>Condition images</p><div className="flex gap-1">{item.condition_thumbnails.map((path, index) => path ? <img key={item.condition[index]} src={previewUrl(path)} alt={`Condition ${index + 1}`} className="size-12 object-cover" /> : <span key={item.condition[index]}>Condition {index + 1}</span>)}</div>
        <p>Product details {item.details_filled}/{item.details_total}</p>
        {item.unreviewed > 0 && <p>{item.unreviewed} unreviewed photos will be published after the main photos</p>}
        {item.announced_removals.length > 0 && <p>{item.announced_removals.length} gallery removals</p>}
        {item.create_item && <p>A new product will be created</p>}
        {item.researching && <p>Research is still running for this item. Its suggestions are not in this publish unless you wait.</p>}
        {item.n_withheld > 0 && <p data-withheld>{withheldLine(item.n_withheld)}</p>}
        {item.conflicts.length > 0 && <ConflictList sessionId={item.session_id} conflicts={item.conflicts} onResolved={() => { setChanged(null); void reload(); }} />}
      </div>)}
      {changed && <p role="alert">{changed}</p>}
      <p data-testid="publish-summary">{preview.summary}</p>
      <div className="flex justify-end gap-2 pt-3"><Button variant="outline" disabled={confirm.busy} onClick={onClose}>Cancel</Button><Button disabled={confirm.busy} aria-busy={confirm.busy ? 'true' : undefined} onClick={() => void confirm.run()}>Publish {count} images{confirm.busy ? '…' : ''}</Button></div>
    </>;
  if (modal) return <Dialog open onOpenChange={open => !open && !confirm.busy && onClose()}>
    <DialogContent showCloseButton={false} className="max-h-[85vh] overflow-y-auto">
      <DialogTitle>Review publish</DialogTitle>
      <DialogDescription className="sr-only">What will be published for the confirmed sessions</DialogDescription>
      <div>{body}</div>
    </DialogContent>
  </Dialog>;
  return <section role="dialog" aria-label="Review publish" className="absolute right-4 top-14 z-50 w-80 rounded-lg border bg-background p-4 shadow-lg">
    <h2 className="font-semibold">Review publish</h2>
    {body}
  </section>;
}
