import { useEffect, useState } from 'react';
import { DropdownMenu } from 'radix-ui';
import { MoreHorizontal, Trash2 } from 'lucide-react';
import { toast } from 'sonner';
import { ActionButton } from '@/components/ui/action-button';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { deleteSession, sessionDeletePreview, type SessionDeletePreview } from '@/services/sessions';

/**
 * Owner correction A17 (2026-11-18): "Delete session", for test items and mistakes. Session Manager only: the
 * session's files go to the macOS Trash and its records are removed; Shopify is not changed.
 */
export function SessionActionsMenu({ sessionId, onDeleted, className = 'row-menu-button' }: { sessionId: number; onDeleted: () => void; className?: string }) {
  const [confirming, setConfirming] = useState(false);
  return (
    <>
      <DropdownMenu.Root modal={false}>
        <DropdownMenu.Trigger className={className} aria-label="Session actions">
          <MoreHorizontal />
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content className="photo-menu" align="end" sideOffset={6}>
            <DropdownMenu.Item className="photo-menu-item" data-danger onSelect={() => setConfirming(true)}>
              <span>Delete session…</span>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
      {confirming && <DeleteSessionDialog sessionId={sessionId} onClose={() => setConfirming(false)} onDeleted={onDeleted} />}
    </>
  );
}

function photos(n: number): string {
  return n === 1 ? '1 photo' : `${n} photos`;
}

/** The confirm dialog: what is deleted, where the files go, and that Shopify is not changed. */
export function DeleteSessionDialog({ sessionId, onClose, onDeleted }: { sessionId: number; onClose: () => void; onDeleted: () => void }) {
  const [preview, setPreview] = useState<SessionDeletePreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let alive = true;
    sessionDeletePreview(sessionId)
      .then(p => alive && setPreview(p))
      .catch(e => alive && setError((e as Error).message));
    return () => {
      alive = false;
    };
  }, [sessionId]);
  const name = preview ? preview.item_type || preview.title : '';
  return (
    <Dialog open onOpenChange={open => !open && onClose()}>
      <DialogContent className="delete-session-dialog" showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>Delete {preview ? preview.sku : 'session'}?</DialogTitle>
          <DialogDescription>
            {preview ? `${name} · ${photos(preview.photos)}${preview.published ? ' · published' : ''}` : error ?? 'Loading…'}
          </DialogDescription>
        </DialogHeader>
        {preview && (
          <ul className="delete-session-facts">
            <li>Its photo files, Capture One's files of them and exported JPGs go to the macOS Trash. You can put them back from there.</li>
            <li>The session and its details are removed from Session Manager. This cannot be undone in the app.</li>
            <li>{preview.published ? 'The product in Shopify is not changed.' : 'Shopify is not changed.'}</li>
            {preview.archived && <li>Its files were already cleared after publishing; only its records are left.</li>}
            {preview.active && <li>New photos go to Unassigned from now on.</li>}
          </ul>
        )}
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <ActionButton
            variant="destructive"
            disabled={!preview}
            action={async () => {
              const done = await deleteSession(sessionId);
              toast.success(`Deleted ${done.sku}`, {
                description: done.files_trashed > 0 ? `${done.files_trashed === 1 ? '1 file' : `${done.files_trashed} files`} moved to the Trash` : 'No files were left on this Mac',
              });
              onClose();
              onDeleted();
            }}
            onError={e => toast.error((e as Error).message)}
          >
            <Trash2 /> Delete session
          </ActionButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
