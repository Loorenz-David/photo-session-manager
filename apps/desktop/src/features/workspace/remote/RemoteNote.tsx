import { Button } from '@/components/ui/button';
import type { RemoteView } from '@/services/remote';

/**
 * H4 (10H §6, MC-H3.2): the line under the header of a product whose photos are on Shopify, with **Bring back now**
 * (the way to edit in Capture One, which cannot start inside the app); while an adopted product's first read runs,
 * "Checking Shopify…"; when that read failed, the sentence and **Try again**.
 */
export function RemoteNote({ view, onBringBack, onTryAgain }: { view: RemoteView; onBringBack: () => void; onTryAgain: () => void }) {
  if (view.state === 'archived' || view.state === 'shop_only') {
    return (
      <div className="remote-note" role="note">
        <span>Photos are on Shopify. Editing a photo brings them back to this Mac first.</span>
        <Button variant="outline" size="sm" onClick={onBringBack}>Bring back now</Button>
      </div>
    );
  }
  if (view.state === 'unknown') {
    return view.check_failed ? (
      <div className="remote-note" role="note">
        <span>Could not check Shopify for this product's photos.</span>
        <Button variant="outline" size="sm" onClick={onTryAgain}>Try again</Button>
      </div>
    ) : (
      <div className="remote-note" role="note" aria-busy="true">
        <span>Checking Shopify…</span>
      </div>
    );
  }
  return null;
}
