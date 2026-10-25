import { useState } from 'react';
import { Sparkles } from 'lucide-react';
import { Textarea } from '@/components/ui/textarea';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import type { FieldInputProps } from '../MetafieldInput';

type Mode = 'ai' | 'manual';

/**
 * The product description: "AI agent writes it" (default; the agent is post-MVP, so this is a placeholder that
 * writes nothing) or "Write manually" (saved as typed, published as written). A stored description opens in
 * manual mode.
 */
export function DescriptionInput({ id, label, value, onChange, onBlur }: FieldInputProps) {
  const text = typeof value === 'string' ? value : '';
  const [mode, setMode] = useState<Mode>(text ? 'manual' : 'ai');
  return (
    <div className="flex flex-col gap-2">
      <ToggleGroup type="single" variant="outline" size="sm" aria-label={`${label} mode`} value={mode} onValueChange={next => next && setMode(next as Mode)}>
        <ToggleGroupItem value="ai">
          <Sparkles className="size-3.5" /> AI agent writes it
        </ToggleGroupItem>
        <ToggleGroupItem value="manual">Write manually</ToggleGroupItem>
      </ToggleGroup>
      {mode === 'ai' ? (
        <p className="rounded-lg bg-(--ai-muted) px-3 py-2 text-xs text-(--ai-foreground)">
          The AI agent writes the description after publishing. It is not available yet; choose “Write manually” to write it yourself.
        </p>
      ) : (
        <Textarea id={id} rows={6} value={text} onChange={event => onChange(event.target.value)} onBlur={onBlur} />
      )}
    </div>
  );
}
