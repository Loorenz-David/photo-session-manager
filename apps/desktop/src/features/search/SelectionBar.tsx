import { useEffect, useId, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import { motion } from 'motion/react';
import { ArrowLeft, ArrowUp, Check, ChevronRight, Loader2, Quote, Search, Sparkles, TextSelect, TriangleAlert, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import type { FormField } from '@/services/attributes';

export type Selected = { text: string; url: string; title: string };

/** What "Ask AI…" came back with, as the bar shows it. */
export type AskOutcome = { suggested: { label: string; value: string }[]; notWritten: [string, string][] };

type Mode = 'actions' | 'fields' | 'ask' | 'asking' | 'done';

/** Characters from which the bar also says how long the selection is. */
const COUNT_FROM = 120;

function hostOf(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, '');
  } catch {
    return url;
  }
}

/** The fields the "Other field…" list shows for `typed`. */
export function matchingFields(fields: FormField[], typed: string): FormField[] {
  const needle = typed.trim().toLowerCase();
  return needle ? fields.filter(f => f.definition.label.toLowerCase().includes(needle)) : fields;
}

const enter = { initial: { opacity: 0, y: 6 }, animate: { opacity: 1, y: 0 }, transition: { duration: 0.16, ease: 'easeOut' as const } };

/**
 * A14: what is selected on the page and its actions, in one floating bar under the page (the page is a separate
 * native view over the browser area, so nothing of the app can be drawn over it: the bar, its field list and the
 * AI prompt all stay inside this band, which has the same height in every state so the page never jumps).
 */
export function SelectionBar({ selected, refused, quick, rest, onPut, onAsk, onOpenReview }: {
  selected: Selected | null;
  refused: string | null;
  quick: FormField[];
  rest: FormField[];
  onPut: (field: FormField, selection: Selected) => Promise<void>;
  onAsk: (selection: Selected, instruction: string) => Promise<AskOutcome | null>;
  onOpenReview: () => void;
}) {
  const [mode, setMode] = useState<Mode>('actions');
  const [dismissed, setDismissed] = useState<Selected | null>(null);
  // The selection the field list or the AI prompt works on: kept while the person types, even if the page's own
  // selection changes or goes meanwhile.
  const [held, setHeld] = useState<Selected | null>(null);
  const [instruction, setInstruction] = useState('');
  const [typed, setTyped] = useState('');
  const [active, setActive] = useState(0);
  const [outcome, setOutcome] = useState<AskOutcome | null>(null);
  const listId = useId();
  const promptRef = useRef<HTMLInputElement>(null);
  const findRef = useRef<HTMLInputElement>(null);
  const holding = mode === 'fields' || mode === 'ask' || mode === 'asking';
  const current = holding ? held : selected;

  // Another selection on the page: the bar shows it (an AI answer on show gives way to it; typing is never cut off).
  const selectionKey = selected ? `${selected.url}\n${selected.text}` : '';
  useEffect(() => {
    if (!selectionKey) return;
    setMode(now => (now === 'done' ? 'actions' : now));
  }, [selectionKey]);
  useEffect(() => {
    if (mode === 'ask') promptRef.current?.focus();
    if (mode === 'fields') findRef.current?.focus();
  }, [mode]);

  const fields = useMemo(() => matchingFields(rest, typed), [rest, typed]);
  const isDismissed = selected !== null && dismissed !== null && dismissed.text === selected.text && dismissed.url === selected.url;
  const showing = holding ? held !== null : selected !== null && !isDismissed;

  function hold(next: Mode) {
    setHeld(selected);
    setTyped('');
    setActive(0);
    setMode(next);
  }

  async function put(field: FormField) {
    const selection = current;
    setMode('actions');
    if (selection) await onPut(field, selection);
  }

  async function send() {
    if (!instruction.trim() || mode === 'asking' || !held) return;
    setMode('asking');
    const answer = await onAsk(held, instruction.trim());
    if (answer) {
      setOutcome(answer);
      setMode('done');
      setInstruction('');
    } else {
      setMode('ask');
    }
  }

  function onPromptKey(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === 'Enter') {
      event.preventDefault();
      void send();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      setMode('actions');
    }
  }

  function onFindKey(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
      event.preventDefault();
      setActive(i => Math.min(i + 1, Math.max(fields.length - 1, 0)));
    } else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
      event.preventDefault();
      setActive(i => Math.max(i - 1, 0));
    } else if (event.key === 'Enter') {
      event.preventDefault();
      const field = fields[Math.min(active, fields.length - 1)];
      if (field) void put(field);
    } else if (event.key === 'Escape') {
      event.preventDefault();
      setMode('actions');
    }
  }

  const doneLine = outcome
    ? outcome.suggested.length > 0
      ? `${outcome.suggested.length === 1 ? '1 suggestion' : `${outcome.suggested.length} suggestions`} sent to AI review`
      : outcome.notWritten.length > 0
        ? 'Nothing new to suggest'
        : 'AI found nothing for that in the selection'
    : '';
  const doneDetail = outcome
    ? outcome.suggested.length > 0
      ? outcome.suggested.map(s => `${s.label}: ${s.value}`).join(' · ')
      : outcome.notWritten.map(([label, why]) => `${label}: ${why}`).join(' · ')
    : '';

  return (
    <div className="flex h-16 shrink-0 items-center justify-center border-t bg-(--stage)/40 px-3" aria-label="Selected text">
      {mode === 'done' || mode === 'asking' ? null : !showing ? (
        <motion.div
          key={refused ? 'refused' : 'hint'}
          {...enter}
          role="status"
          className={cn(
            'inline-flex max-w-full items-center gap-2 truncate rounded-full border px-3 py-1 text-xs shadow-(--shadow-sm)',
            refused ? 'border-amber-500/40 bg-amber-50 text-amber-900 dark:bg-amber-950/40 dark:text-amber-200' : 'bg-background/90 text-muted-foreground',
          )}
        >
          {refused ? <TriangleAlert className="size-3.5 shrink-0" /> : <TextSelect className="size-3.5 shrink-0" />}
          <span className="truncate">{refused ?? 'Select text on the page to put it in a field or ask AI about it'}</span>
        </motion.div>
      ) : null}

      {(showing || mode === 'done' || mode === 'asking') && (
        <motion.div
          key={mode === 'done' || mode === 'asking' ? mode : `bar-${mode}`}
          {...enter}
          data-testid="selection-bar"
          data-mode={mode}
          className="flex h-12 w-full max-w-[1120px] items-center gap-1.5 rounded-xl border bg-popover pr-1.5 pl-3 text-popover-foreground shadow-(--shadow-md)"
        >
          {mode === 'actions' && selected && (
            <>
              <Quote className="size-3.5 shrink-0 text-muted-foreground" aria-hidden />
              <p className="min-w-0 flex-1 truncate text-sm" title={selected.text} data-testid="selection-snippet">
                {selected.text}
              </p>
              <span className="hidden shrink-0 text-xs text-muted-foreground xl:inline">
                {hostOf(selected.url)}{selected.text.length >= COUNT_FROM ? ` · ${selected.text.length.toLocaleString('en')} characters` : ''}
              </span>
              <span className="mx-1 h-6 w-px shrink-0 bg-border" aria-hidden />
              {quick.map(field => (
                <Button key={field.definition.key} size="sm" variant="secondary" aria-label={`Put in ${field.definition.label}`} title={`Put the selection in ${field.definition.label}`} onClick={() => void put(field)}>
                  <ArrowUp className="rotate-45" /> {field.definition.label}
                </Button>
              ))}
              {rest.length > 0 && (
                <Button size="sm" variant="ghost" title="Put the selection in another field" onClick={() => hold('fields')}>
                  {quick.length > 0 ? 'Other field…' : 'Put in field…'} <ChevronRight />
                </Button>
              )}
              <Button size="sm" className="bg-(--ai-muted) text-(--ai-foreground) hover:bg-[color-mix(in_oklch,var(--ai-muted),var(--ai)_18%)]" title="Ask AI about the selection" onClick={() => hold('ask')}>
                <Sparkles /> Ask AI
              </Button>
              <Button size="icon-sm" variant="ghost" aria-label="Dismiss selection" title="Dismiss" onClick={() => setDismissed(selected)}>
                <X />
              </Button>
            </>
          )}

          {mode === 'fields' && (
            <>
              <Button size="icon-sm" variant="ghost" aria-label="Back" title="Back (Esc)" onClick={() => setMode('actions')}>
                <ArrowLeft />
              </Button>
              <div className="flex h-8 w-52 shrink-0 items-center gap-1.5 rounded-lg border bg-background px-2">
                <Search className="size-3.5 shrink-0 text-muted-foreground" aria-hidden />
                <input
                  ref={findRef}
                  role="combobox"
                  aria-expanded
                  aria-controls={listId}
                  aria-activedescendant={fields[active] ? `${listId}-${active}` : undefined}
                  aria-label="Find a field"
                  placeholder="Find a field…"
                  className="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
                  value={typed}
                  onChange={event => { setTyped(event.target.value); setActive(0); }}
                  onKeyDown={onFindKey}
                />
              </div>
              <div id={listId} role="listbox" aria-label="Fields" aria-orientation="horizontal" className="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto py-1">
                {fields.length === 0 && <span className="px-2 text-xs text-muted-foreground">No field matches</span>}
                {fields.map((field, index) => (
                  <button
                    key={field.definition.key}
                    id={`${listId}-${index}`}
                    type="button"
                    role="option"
                    aria-selected={index === active}
                    onMouseEnter={() => setActive(index)}
                    onClick={() => void put(field)}
                    className={cn(
                      'h-7 shrink-0 rounded-full border px-3 text-xs whitespace-nowrap transition-colors',
                      index === active ? 'border-foreground/30 bg-foreground text-background' : 'bg-muted/60 hover:bg-muted',
                    )}
                  >
                    {field.definition.label}
                  </button>
                ))}
              </div>
            </>
          )}

          {mode === 'ask' && (
            <>
              <Sparkles className="size-4 shrink-0 text-(--ai)" aria-hidden />
              <input
                ref={promptRef}
                aria-label="Instruction for AI"
                maxLength={300}
                placeholder="What should AI find in the selection? e.g. the production year"
                className="h-8 min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
                value={instruction}
                onChange={event => setInstruction(event.target.value)}
                onKeyDown={onPromptKey}
              />
              <span className="hidden shrink-0 text-[11px] text-muted-foreground lg:inline">Return to send · Esc to cancel</span>
              <Button size="sm" className="bg-(--ai) text-white hover:bg-(--ai-700)" aria-label="Send to AI" disabled={!instruction.trim()} onClick={() => void send()}>
                <Sparkles /> Ask
              </Button>
              <Button size="icon-sm" variant="ghost" aria-label="Cancel" title="Cancel (Esc)" onClick={() => setMode('actions')}>
                <X />
              </Button>
            </>
          )}

          {mode === 'asking' && (
            <p className="flex min-w-0 flex-1 items-center gap-2 text-sm text-(--ai-foreground)" role="status">
              <Loader2 className="size-4 shrink-0 animate-spin" /> Asking AI…
            </p>
          )}

          {mode === 'done' && (
            <>
              <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-(--ai-muted) text-(--ai-foreground)" aria-hidden>
                <Check className="size-3.5" />
              </span>
              <p className="min-w-0 flex-1 truncate text-sm" role="status" title={doneDetail}>
                <span className="font-medium">{doneLine}</span>
                {doneDetail && <span className="text-muted-foreground"> — {doneDetail}</span>}
              </p>
              {outcome && outcome.suggested.length > 0 && (
                <Button size="sm" className="bg-(--ai-muted) text-(--ai-foreground) hover:bg-[color-mix(in_oklch,var(--ai-muted),var(--ai)_18%)]" onClick={onOpenReview}>
                  Open AI review <ChevronRight />
                </Button>
              )}
              <Button size="icon-sm" variant="ghost" aria-label="Close" title="Close" onClick={() => { setOutcome(null); setMode('actions'); }}>
                <X />
              </Button>
            </>
          )}
        </motion.div>
      )}
    </div>
  );
}
