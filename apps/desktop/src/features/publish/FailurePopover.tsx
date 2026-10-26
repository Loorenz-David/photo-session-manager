import { Button } from '@/components/ui/button';

export function FailurePopover({ reason, onRetry, onClose }: { reason: string; onRetry: () => void; onClose: () => void }) {
  return <section role="dialog" aria-label="Publish failure" className="fixed right-6 top-20 z-50 rounded-lg border bg-background p-4 shadow-lg">
    <h2>Publish failed</h2><p>{reason}</p><p>Nothing was overwritten</p>
    <Button onClick={onRetry}>Retry</Button><Button variant="ghost" onClick={onClose}>Close</Button>
  </section>;
}
