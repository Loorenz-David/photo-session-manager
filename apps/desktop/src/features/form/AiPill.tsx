import { forwardRef, type ComponentProps } from 'react';
import { cn } from '@/lib/utils';

/** Bars per confidence (evidence design_handoff §AI): high 3, medium 2, low 1. */
export const CONFIDENCE_BARS: Record<string, number> = { high: 3, medium: 2, low: 1 };

/** Three confidence bars; `filled` of them are on. */
export function ConfidenceBars({ filled }: { filled: number }) {
  return (
    <span className="inline-flex items-end gap-px" aria-hidden>
      {[1, 2, 3].map(n => (
        <span key={n} data-bar data-filled={n <= filled} className={cn('w-[3px] rounded-[1px]', n <= filled ? 'bg-current' : 'bg-current/25')} style={{ height: 3 + n * 2 }} />
      ))}
    </span>
  );
}

type AiPillProps = { confidence: string | null } & ComponentProps<'button'>;

/** The pill of a field the AI agent filled: "AI" with 3/2/1 bars; low confidence reads "Check". */
export const AiPill = forwardRef<HTMLButtonElement, AiPillProps>(function AiPill({ confidence, className, ...props }, ref) {
  const filled = CONFIDENCE_BARS[confidence ?? ''] ?? 1;
  const low = filled === 1;
  return (
    <button
      ref={ref}
      type="button"
      data-ai-pill
      aria-label={`AI suggestion, ${confidence ?? 'unknown'} confidence`}
      className={cn('inline-flex items-center gap-1 rounded-full bg-(--ai-muted) px-2 py-0.5 text-[11px] font-semibold text-(--ai-foreground)', className)}
      {...props}
    >
      <span>{low ? 'Check' : 'AI'}</span>
      <ConfidenceBars filled={filled} />
    </button>
  );
});
