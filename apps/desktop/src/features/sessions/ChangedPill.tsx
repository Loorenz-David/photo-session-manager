import { toast } from 'sonner';
import { ActionButton } from '@/components/ui/action-button';
import { keepPublished, type PublishedChanges } from '@/services/sessions';

/** The reasons a published session is open again, in words (A10). */
export function changeReasons(changes: PublishedChanges): string[] {
  const reasons: string[] = [];
  if (changes.new_photos > 0) reasons.push(changes.new_photos === 1 ? '1 new photo' : `${changes.new_photos} new photos`);
  if (changes.editor_edits > 0) {
    reasons.push(changes.editor_edits === 1 ? '1 photo edited in Capture One' : `${changes.editor_edits} photos edited in Capture One`);
  }
  if (changes.app_edits) reasons.push('edited in the app');
  if (reasons.length === 0) reasons.push(changes.unrecorded ? 'changed before this version noted why' : 'changed');
  return reasons;
}

/**
 * Owner correction A10 (2026-11-18): a published session that changed since publishing never reopens silently. It
 * says "Changed after publishing" with the reasons, and offers "Keep as published" (photos that arrived late are
 * denied; the session counts as published again; one undo step), or the person publishes again.
 */
export function ChangedPill({ sessionId, changes, onKept }: { sessionId: number; changes: PublishedChanges | null | undefined; onKept?: () => void }) {
  if (!changes) return null;
  const reasons = changeReasons(changes);
  const keepTitle = changes.new_photos > 0
    ? 'Keep the published listing as it is: the new photos are denied and nothing is published'
    : 'Keep the published listing as it is: nothing is published';
  return (
    <span className="changed-after-publish" data-testid={`changed-${sessionId}`}>
      <span className="pill changed-pill" title={reasons.join(' · ')}>
        Changed after publishing · {reasons.join(' · ')}
      </span>
      <ActionButton
        size="sm"
        variant="ghost"
        className="changed-keep"
        title={keepTitle}
        action={async () => {
          await keepPublished(sessionId);
          toast.success('Kept as published', { description: 'Undo in the session if that was a mistake.' });
          onKept?.();
        }}
        onError={error => toast.error((error as Error).message)}
      >
        Keep as published
      </ActionButton>
    </span>
  );
}
