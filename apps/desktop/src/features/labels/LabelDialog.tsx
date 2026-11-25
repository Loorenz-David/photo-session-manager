import { Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { openSettingsSection } from '@/features/settings/SettingsNav';
import type { LabelRunResult } from '@/services/labels';
import { LabelCheckView } from './LabelCheckView';
import { LabelDoneView } from './LabelDoneView';
import { CLOSE, OPENING_IN_PREVIEW, OPEN_PRICING_SETTINGS, doneTitle } from './labelWords';
import { useLabelDialog } from './useLabelDialog';

/**
 * L3 (09C §6), L4 (09C §14.2): the label dialog for one product (⋯ → Generate label / Reprint label) or a batch
 * (Generate labels for N). It opens at once with a busy state while the products are checked, shows the check and the
 * preview, generates on Generate PDF, then saves and opens every file in Preview by itself (it cannot be closed while
 * generating or saving: the close keys, Escape and a click outside do nothing). Everything generated → it closes with a
 * toast; otherwise it stays on the result. `onClose` gets the run's result when there is one (the Labels box keeps the
 * failed products selected and refreshes, MC-L1.9).
 */
export function LabelDialog({ productIds, onClose }: { productIds: string[]; onClose: (result: LabelRunResult | null) => void }) {
  const dialog = useLabelDialog(productIds, onClose);
  const { state } = dialog;
  const single = productIds.length === 1;
  const generating = state.phase === 'generating';
  const busy = generating || state.phase === 'saving';
  const title = state.phase === 'done'
    ? doneTitle(state.result, single, state.check.unresolved.length)
    : single ? 'Generate label' : `Generate labels for ${productIds.length}`;
  return (
    <Dialog open onOpenChange={open => !open && dialog.leave()}>
      <DialogContent
        className="label-dialog sm:max-w-3xl"
        showCloseButton={false}
        onEscapeKeyDown={event => busy && event.preventDefault()}
        onPointerDownOutside={event => busy && event.preventDefault()}
        onInteractOutside={event => busy && event.preventDefault()}
        aria-busy={state.phase === 'checking' || busy}
      >
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription className="sr-only">Price labels from Pricing</DialogDescription>
        </DialogHeader>
        {state.phase === 'checking' && (
          <div className="label-busy" role="status" aria-busy="true">
            <Loader2 className="animate-spin" aria-hidden /> Checking…
          </div>
        )}
        {state.phase === 'error' && (
          <div className="label-dialog-error-view">
            <p role="alert">{state.words}</p>
            <div className="label-dialog-buttons">
              {state.failure.code === 'labels_off' && (
                <Button
                  variant="outline"
                  onClick={() => {
                    openSettingsSection('pricing');
                    dialog.close(null);
                  }}
                >
                  {OPEN_PRICING_SETTINGS}
                </Button>
              )}
              <Button onClick={() => dialog.close(null)}>{CLOSE}</Button>
            </div>
          </div>
        )}
        {(state.phase === 'check' || state.phase === 'generating' || state.phase === 'saving') && (
          <LabelCheckView
            check={state.check}
            single={single}
            generating={busy}
            busyWords={state.phase === 'saving' ? OPENING_IN_PREVIEW : undefined}
            failure={state.phase === 'check' ? state.failure : null}
            items={dialog.items}
            shown={dialog.shown}
            preview={dialog.preview}
            onStep={dialog.step}
            onShow={dialog.show}
            onGenerate={dialog.generate}
            onLeave={dialog.leave}
          />
        )}
        {state.phase === 'done' && (
          <LabelDoneView
            check={state.check}
            result={state.result}
            single={single}
            downloads={dialog.downloads}
            onRetry={dialog.retry}
            onClose={() => dialog.close(state.result)}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}
