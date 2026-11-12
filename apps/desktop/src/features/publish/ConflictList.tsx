import { useState } from 'react';
import { TriangleAlert } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { jsonText, priorOrigin } from '@/features/form/AiPopover';
import { resolveConflict, type ConflictAction } from '@/services/attributes';
import { getSessionView } from '@/services/photos';
import { waitsLine, type PreviewConflict } from '@/services/publish';

type ConflictListProps = {
  sessionId: number;
  conflicts: PreviewConflict[];
  /** A conflict was resolved: the caller reads the preview again. */
  onResolved: () => void;
};

/**
 * One item's conflicts in the publish confirmation (08B MC-A10), as a warning: per conflict the field, the previous
 * value and where it came from, the suggestion and its confidence, and Keep previous / Use suggestion / Edit (MC-A12).
 * Confirm stays available: only this item's publish waits.
 */
export function ConflictList({ sessionId, conflicts, onResolved }: ConflictListProps) {
  const [editing, setEditing] = useState<string | null>(null);
  const [text, setText] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const resolve = async (key: string, action: ConflictAction, value?: unknown) => {
    setBusy(true);
    try {
      const view = await getSessionView(sessionId);
      const itemId = view.session.item_id ?? view.item?.id;
      if (itemId == null) throw new Error('This session has no item');
      await resolveConflict(itemId, key, action, value);
      setEditing(null);
      setError(null);
      onResolved();
    } catch (failure) {
      setError((failure as Error).message ?? String(failure));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div role="group" aria-label="Conflicts" data-conflicts className="flex flex-col gap-2 rounded-md border border-amber-300 bg-amber-50 p-2 text-xs text-amber-950">
      {conflicts.map(conflict => (
        <div key={conflict.key} data-conflict={conflict.key} className="flex flex-col gap-1">
          <p className="flex items-center gap-1 font-semibold"><TriangleAlert className="size-3.5" aria-hidden /> {conflict.label}</p>
          <p>
            Previous: <span className="font-semibold">{jsonText(conflict.prior_value)}</span> · {priorOrigin(conflict.prior_provenance)}
          </p>
          <p>
            Suggested: <span className="font-semibold">{jsonText(conflict.suggestion)}</span>
            {conflict.confidence && <> · {conflict.confidence} confidence</>}
          </p>
          {editing === conflict.key ? (
            <form className="flex gap-1.5" onSubmit={event => { event.preventDefault(); if (text.trim() !== '') void resolve(conflict.key, 'edit', text); }}>
              <Input aria-label={`New value for ${conflict.label}`} value={text} autoFocus onChange={event => setText(event.target.value)} className="h-7 text-xs" />
              <Button type="submit" size="sm" disabled={busy || text.trim() === ''}>Save</Button>
              <Button type="button" size="sm" variant="ghost" onClick={() => setEditing(null)}>Cancel</Button>
            </form>
          ) : (
            <div className="flex flex-wrap gap-1.5">
              <Button type="button" size="sm" variant="outline" disabled={busy} onClick={() => void resolve(conflict.key, 'keep_previous')}>Keep previous</Button>
              <Button type="button" size="sm" variant="outline" disabled={busy} onClick={() => void resolve(conflict.key, 'use_suggestion')}>Use suggestion</Button>
              <Button type="button" size="sm" variant="outline" disabled={busy} onClick={() => { setEditing(conflict.key); setText(jsonText(conflict.suggestion)); }}>Edit</Button>
            </div>
          )}
        </div>
      ))}
      <p className="font-semibold" data-waits>{waitsLine(conflicts.length)}</p>
      {error && <p role="alert">{error}</p>}
    </div>
  );
}
