import { useCallback, useEffect, useState } from 'react';
import { ActionButton } from '@/components/ui/action-button';
import { writerLookAgain, writerNote, type ProductNote as Note } from '@/services/writer';

/**
 * "What research saw" (book 08C §4, W1): the short factual note research's two photo looks wrote, one line per
 * observation with its photo; an observation the owner decided shows the decision and a "Your decision" mark. When the
 * photos changed since they were looked at (or were never looked at) it offers Look again, which runs the photo looks
 * only. Re-reads whenever `refresh` changes: the panel passes its research status, which the form re-reads on
 * `research_changed` for this session (one listener for both).
 */
export function ProductNote({ sessionId, refresh }: { sessionId: number; refresh?: unknown }) {
  const [note, setNote] = useState<Note | null>(null);
  const [refusal, setRefusal] = useState<string | null>(null);

  const reload = useCallback(async () => {
    try {
      setNote(await writerNote(sessionId));
    } catch {
      // Research not available (or the session has no item): the block stays hidden.
      setNote(null);
    }
  }, [sessionId]);

  useEffect(() => {
    void reload();
  }, [reload, refresh]);

  if (!note) return null;
  const stale = note.stale_main || note.stale_condition;
  const lookAgain = async () => {
    const answer = await writerLookAgain(sessionId);
    setRefusal(answer.queued ? null : (answer.reason?.sentence ?? null));
    await reload();
  };

  return (
    <section aria-label="What research saw" className="flex flex-col gap-2 border-t px-4 py-3 text-xs" data-product-note>
      <h4 className="font-semibold">What research saw</h4>
      {(stale || note.missing_main) && (
        <div role="status" className="flex flex-col items-start gap-1.5 rounded-md bg-(--ai-muted) px-2.5 py-2 text-(--ai-foreground)" data-note-banner>
          <span>{stale ? 'Photos changed since they were looked at' : 'No product note yet'}</span>
          <ActionButton size="sm" variant="secondary" action={lookAgain}>Look again</ActionButton>
        </div>
      )}
      {refusal && <p className="text-muted-foreground" data-note-refusal>{refusal}</p>}
      {note.observations.length > 0 && (
        <ul className="flex flex-col gap-1.5" aria-label="Observations">
          {note.observations.map(observation => (
            <li key={observation.ref} data-ref={observation.ref} data-owner={observation.owner || undefined} className="flex flex-col gap-0.5">
              <span>
                {observation.text}
                {observation.owner && (
                  <span className="ml-1.5 rounded-full bg-(--ai-muted) px-1.5 py-px text-[10px] text-(--ai-foreground)" data-your-decision>
                    Your decision
                  </span>
                )}
              </span>
              {observation.photo && <span className="text-[11px] text-muted-foreground">{observation.photo}</span>}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
