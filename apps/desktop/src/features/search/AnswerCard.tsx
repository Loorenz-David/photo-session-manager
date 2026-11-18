import { motion } from 'motion/react';
import { ArrowUp, MessageCircleQuestion, Sparkles, X } from 'lucide-react';
import type { AskFound } from '@/services/search';

/** A18: an answer in words, kept on its search tab until dismissed. */
export type TabAnswer = { question: string; text: string | null; found: AskFound[]; host: string };

/**
 * A18 (owner 2026-11-18): AI's quick answer about the selected text, above the selection bar (nothing can be drawn
 * over the page). The values it found per field are one-tap "Put '<value>' in <Field>" chips; nothing is written
 * until one is tapped, and then exactly as "Put in" writes selected text (with Undo).
 */
export function AnswerCard({ answer, onPut, onDismiss }: { answer: TabAnswer; onPut: (found: AskFound) => void; onDismiss: () => void }) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.16, ease: 'easeOut' }}
      aria-label="AI answer"
      data-testid="ai-answer"
      className="mx-3 mb-2 flex max-h-44 shrink-0 flex-col gap-2 overflow-y-auto rounded-xl border bg-popover px-3 py-2.5 text-popover-foreground shadow-(--shadow-md)"
    >
      <div className="flex items-start gap-2">
        <span className="mt-0.5 flex size-6 shrink-0 items-center justify-center rounded-full bg-(--ai-muted) text-(--ai-foreground)" aria-hidden>
          <Sparkles className="size-3.5" />
        </span>
        <div className="min-w-0 flex-1">
          <p className="flex items-center gap-1 truncate text-[11px] text-muted-foreground" title={answer.question}>
            <MessageCircleQuestion className="size-3 shrink-0" aria-hidden /> {answer.question} · from {answer.host}
          </p>
          <p className="text-sm leading-snug" data-testid="ai-answer-text">
            {answer.text ?? 'AI gave no answer in words for that.'}
          </p>
        </div>
        <button type="button" aria-label="Dismiss answer" title="Dismiss" onClick={onDismiss} className="flex size-6 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground">
          <X className="size-3.5" />
        </button>
      </div>
      {answer.found.length > 0 && (
        <div className="flex flex-wrap items-center gap-1.5 pl-8" aria-label="Values found">
          {answer.found.map(found => (
            <button
              key={found.key}
              type="button"
              onClick={() => onPut(found)}
              title={found.quote ? `“${found.quote}”` : undefined}
              className="inline-flex h-7 max-w-full items-center gap-1 rounded-full border bg-background px-2.5 text-xs transition-colors hover:border-(--ai)/40 hover:bg-(--ai-muted)"
            >
              <ArrowUp className="size-3 shrink-0 rotate-45 text-(--ai-foreground)" aria-hidden />
              <span className="truncate">
                Put <span className="font-semibold">“{found.value}”</span> in {found.label}
              </span>
            </button>
          ))}
        </div>
      )}
    </motion.section>
  );
}
