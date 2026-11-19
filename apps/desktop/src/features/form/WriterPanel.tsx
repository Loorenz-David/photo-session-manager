import type { WriterStatus } from '@/services/writer';

/** The research panel's money: USD with two decimals, from micro-dollars. */
export function money(microUsd: number): string {
  return `$${(microUsd / 1_000_000).toFixed(2)}`;
}

function plural(n: number, one: string, many: string): string {
  return n === 1 ? `1 ${one}` : `${n} ${many}`;
}

/**
 * The writer's block in the research panel (book 08C §11): for the latest writer run, what it used (fields, the photo
 * note, the titles of matching listings), each text's reason and facts, the post-check per attempt and whether it
 * rewrote once, the open findings and questions, and the cost. Nothing when the session has no writer run.
 */
export function WriterPanel({ status }: { status: WriterStatus | null }) {
  const run = status?.run;
  if (!status || !run) return null;
  const { inputs } = status;
  const rewritten = status.regenerated.length > 0 || status.attempts.length > 1;
  return (
    <section aria-label="Writer" data-writer-panel className="flex flex-col gap-2 border-t px-4 py-3 text-xs">
      <h4 className="font-semibold">Writer</h4>
      <p className="text-muted-foreground" data-writer-inputs>
        Used {plural(inputs.fields, 'field', 'fields')} · {plural(inputs.note, 'observation', 'observations')} · {plural(inputs.listings.length, 'listing title', 'listing titles')}
      </p>
      {inputs.listings.length > 0 && (
        <ul className="list-disc pl-4 text-muted-foreground" aria-label="Listing titles">
          {inputs.listings.map(title => <li key={title}>{title}</li>)}
        </ul>
      )}
      {status.texts.length > 0 && (
        <ul className="flex flex-col gap-1.5" aria-label="Texts">
          {status.texts.map(text => (
            <li key={text.key} data-writer-text={text.key}>
              <span className="font-medium">{text.label}</span>
              {text.reason && <span className="text-muted-foreground">: {text.reason}</span>}
              {text.facts.length > 0 && <span className="block text-muted-foreground">Facts: {text.facts.join(', ')}</span>}
              {text.flagged && <span className="block text-[color-mix(in_oklch,var(--warn),var(--color-text)_45%)]">Check: {text.hits.map(h => h.detail).join('; ')}</span>}
            </li>
          ))}
        </ul>
      )}
      {status.attempts.some(a => a.hits.length > 0) && (
        <ul className="flex flex-col gap-1" aria-label="Checks">
          {status.attempts.filter(a => a.hits.length > 0).map(attempt => (
            <li key={attempt.attempt} data-writer-attempt={attempt.attempt}>
              Check {attempt.attempt}: {attempt.hits.map(h => `${h.text}: ${h.detail}`).join('; ')}
            </li>
          ))}
        </ul>
      )}
      {rewritten && <p data-rewritten>Rewritten once</p>}
      {status.findings.length > 0 && (
        <ul className="flex flex-col gap-1" aria-label="Findings and questions">
          {status.findings.map(finding => (
            <li key={finding.id} data-writer-finding={finding.id}>
              {finding.kind === 'question' ? 'Question' : 'Note'}: {finding.text}
            </li>
          ))}
        </ul>
      )}
      <p data-writer-cost>Cost {money(status.cost_micro_usd)}</p>
    </section>
  );
}
