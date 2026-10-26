import { useEffect } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { SkeletonRows } from '@/components/pending/SkeletonRows';
import { useAction, usePageData } from '@/lib/pending';
import { previewUrl } from '@/services/sessions';
import { publishConfirm, publishPreview, type PreviewDto } from '@/services/publish';

/**
 * The publish confirmation surface (07A MC-10). `modal` renders it as a dialog of its own, for callers that sit inside
 * another modal (the batch queue's form dialog): mounted in that dialog's tree, it stacks as the top layer.
 */
export function PublishPopover({ sessionIds, onClose, onConfirmed, modal = false }: { sessionIds: number[]; onClose: () => void; onConfirmed?: () => void; modal?: boolean }) {
  const key = sessionIds.join(',');
  const { data: preview, error } = usePageData<PreviewDto>(() => publishPreview(sessionIds), [key]);
  useEffect(() => { if (error) toast.error(`Could not preview publish: ${(error as Error).message}`); }, [error]);
  const count = preview?.items.reduce((n, item) => n + item.main.length + item.condition.length, 0) ?? 0;
  const confirm = useAction(async () => {
    if (!preview) return;
    await publishConfirm(sessionIds, preview.summary);
    (onConfirmed ?? onClose)();
  }, error => toast.error(`Could not publish: ${(error as Error).message}`));
  const body = !preview ? <SkeletonRows n={3} /> : <>
      {preview.items.map(item => <div key={item.session_id} className="space-y-2 py-2">
        <p>Main images</p><div className="flex gap-1">{item.main_thumbnails.map((path, index) => path ? <img key={item.main[index]} src={previewUrl(path)} alt={`Main ${index + 1}`} className="size-12 object-cover" /> : <span key={item.main[index]}>Main {index + 1}</span>)}</div>
        <p>Condition images</p><div className="flex gap-1">{item.condition_thumbnails.map((path, index) => path ? <img key={item.condition[index]} src={previewUrl(path)} alt={`Condition ${index + 1}`} className="size-12 object-cover" /> : <span key={item.condition[index]}>Condition {index + 1}</span>)}</div>
        <p>Product details {item.details_filled}/{item.details_total}</p>
        {item.unreviewed > 0 && <p>{item.unreviewed} unreviewed photos will not be published</p>}
        {item.announced_removals.length > 0 && <p>{item.announced_removals.length} gallery removals</p>}
        {item.create_item && <p>A new product will be created</p>}
      </div>)}
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
