import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { writerInstructionReset, writerPreview, type WriterInstruction as Instruction, type WriterPreview, type WriterSettings } from '@/services/writer';

function message(error: unknown): string {
  return error instanceof Error ? error.message : (error as { message?: string }).message ?? String(error);
}

/** JSON text shown indented (left as it is when it is not JSON). */
function pretty(text: string): string {
  try {
    return JSON.stringify(JSON.parse(text), null, 2);
  } catch {
    return text;
  }
}

function words(text: string): number {
  return text.split(/\s+/).filter(word => /[\p{L}\p{N}]/u.test(word)).length;
}

/**
 * Settings → Writer → Instruction (08C §8): the instruction of one language in an editor, **Preview what is sent** (the
 * exact request for the built-in example item under the saved settings), the earlier versions with **Use this**, and
 * **Reset to default**. A version put back is saved only by Save.
 */
export function WriterInstruction({ language, instruction, onChange, onReset }: {
  language: string;
  instruction: Instruction;
  /** The editor's text (saved by Save). */
  onChange: (text: string) => void;
  /** The settings the reset saved. */
  onReset: (saved: WriterSettings) => void;
}) {
  const [preview, setPreview] = useState<WriterPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const text = instruction.text ?? instruction.effective_instruction;

  const showPreview = async () => {
    setBusy(true);
    try {
      setPreview(await writerPreview(language));
      setError(null);
    } catch (failure) {
      setError(message(failure));
    } finally {
      setBusy(false);
    }
  };
  const reset = async () => {
    setBusy(true);
    try {
      onReset(await writerInstructionReset(language));
      setError(null);
    } catch (failure) {
      setError(message(failure));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div role="group" aria-label="Instruction" className="flex flex-col gap-2">
      <h3>Instruction</h3>
      <label htmlFor="writer-instruction">Instruction ({language})</label>
      <Textarea id="writer-instruction" rows={16} value={text} onChange={event => onChange(event.target.value)} />
      <p className="health-detail" data-instruction-state>
        {words(text)} words · {instruction.is_default && instruction.text === null ? 'the shipped instruction' : 'your instruction'}
      </p>
      <div className="flex gap-2">
        <Button type="button" variant="outline" disabled={busy} onClick={() => void showPreview()}>Preview what is sent</Button>
        <Button type="button" variant="outline" disabled={busy} onClick={() => void reset()}>Reset to default</Button>
      </div>
      <p className="health-detail">The preview shows the saved settings: save first to see a change in it.</p>
      {error && <p role="alert">{error}</p>}

      <h4>Earlier versions</h4>
      {instruction.versions.length === 0 ? <p className="health-detail">None yet.</p> : (
        <ul aria-label="Earlier versions" className="flex flex-col gap-1">
          {instruction.versions.map((version, index) => (
            <li key={`${version.saved_at}-${index}`} className="flex items-center gap-2">
              <span className="health-detail">{new Date(version.saved_at).toLocaleString()}</span>
              <span className="truncate">{version.text.slice(0, 80)}</span>
              <Button type="button" variant="ghost" size="sm" className="ml-auto" aria-label={`Use the version of ${new Date(version.saved_at).toLocaleString()}`}
                onClick={() => onChange(version.text)}>Use this</Button>
            </li>
          ))}
        </ul>
      )}

      <Dialog open={preview !== null} onOpenChange={open => { if (!open) setPreview(null); }}>
        <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-3xl">
          <DialogTitle>What is sent</DialogTitle>
          <DialogDescription>The request a write sends in {language}, for a built-in example item. Nothing was sent.</DialogDescription>
          {preview && <>
            <h4>Instructions</h4>
            <pre data-preview="instructions" className="whitespace-pre-wrap text-xs">{preview.instructions}</pre>
            <h4>Input</h4>
            <pre data-preview="input" className="whitespace-pre-wrap text-xs">{pretty(preview.input)}</pre>
            <h4>Schema</h4>
            <pre data-preview="schema" className="whitespace-pre-wrap text-xs">{pretty(preview.schema)}</pre>
          </>}
        </DialogContent>
      </Dialog>
    </div>
  );
}
