import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';

export function FailurePopover({ reason, onRetry, onClose }: { reason: string; onRetry: () => Promise<unknown> | void; onClose: () => void }) {
  return <section role="dialog" aria-label="Publish failure" className="fixed right-6 top-20 z-50 rounded-lg border bg-background p-4 shadow-lg">
    <h2>Publish failed</h2><p>{reason}</p><p>Nothing was overwritten</p>
    <ActionButton action={async () => { await onRetry(); }}>Retry</ActionButton><Button variant="ghost" onClick={onClose}>Close</Button>
  </section>;
}
