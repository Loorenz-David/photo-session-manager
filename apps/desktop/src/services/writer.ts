import { open, save } from '@tauri-apps/plugin-dialog';
import {
  commands,
  type NoteDto_Serialize,
  type ObservationDto_Serialize,
  type RequestedDto_Serialize,
  type WriterPreviewDto,
  type WriterSettingsDto_Serialize,
  type WriterStatusDto_Serialize,
} from './bindings';
import { unwrap } from './sessions';

/** W1: what research saw on the session's photos, as the owner's decisions show it, and whether the photos changed. */
export type ProductNote = NoteDto_Serialize;
export type NoteObservation = ObservationDto_Serialize;
/** Whether a request queued a run, and why not (`reason.sentence` is shown). */
export type Requested = RequestedDto_Serialize;
/** W3: the writer's state for a session: its latest run, texts, checks, open findings and questions, cost. */
export type WriterStatus = WriterStatusDto_Serialize;
export type WriterText = WriterStatus['texts'][number];
export type WriterFinding = WriterStatus['findings'][number];
/** W3: why the writer is asked to write (`answer` is internal to the engine). */
export type WriterOrigin = 'next_to_review' | 'all_checked' | 'write_texts';

export async function writerNote(sessionId: number): Promise<ProductNote> {
  return unwrap(await commands.writerNote(sessionId));
}
/** W1: Look again — the photo looks run again over the changed photos (no listing search, no new suggestions). */
export async function writerLookAgain(sessionId: number): Promise<Requested> {
  return unwrap(await commands.writerLookAgain(sessionId));
}
/** W3: the writer's state for the session (writer block in AI review, writer block in the research panel). */
export async function writerStatus(sessionId: number): Promise<WriterStatus> {
  return unwrap(await commands.writerStatus(sessionId));
}
/** W3: asks the writer to write the session's texts. */
export async function writerRequest(sessionId: number, origin: WriterOrigin): Promise<Requested> {
  return unwrap(await commands.writerRequest(sessionId, origin));
}
/** W3: answers an open writer question or finding; the texts are written again. */
export async function writerFindingAnswer(findingId: number, answer: string): Promise<Requested> {
  return unwrap(await commands.writerFindingAnswer(findingId, answer));
}
/** W3: dismisses an open writer note or question. */
export async function writerFindingDismiss(findingId: number): Promise<void> {
  unwrap(await commands.writerFindingDismiss(findingId));
}

/** W4: Settings → Writer, every field present, with the safeguard counts per finding type. */
export type WriterSettings = WriterSettingsDto_Serialize;
export type WriterInstruction = WriterSettings['instructions'][string];
export type WriterExample = WriterSettings['examples'][string][number];
/** W4: the exact request a write would send (`input` and `schema` as JSON text). */
export type WriterPreview = WriterPreviewDto;

/** W4: Settings → Writer as saved. */
export async function writerSettingsGet(): Promise<WriterSettings> {
  return unwrap(await commands.writerSettingsGet());
}
/** W4: saves the whole writer slot; an instruction whose text changes keeps the replaced text among the earlier versions. */
export async function writerSettingsSave(value: WriterSettings): Promise<WriterSettings> {
  return unwrap(await commands.writerSettingsSave(value));
}
/**
 * A23 **Export…**: asks where to save, then writes the saved writer settings there (only the writer settings, without
 * earlier instruction versions). Resolves to false when the person cancels.
 */
export async function writerSettingsExport(): Promise<boolean> {
  const path = await save({ title: 'Export writer settings', defaultPath: 'Writer settings.json',
    filters: [{ name: 'Writer settings', extensions: ['json'] }] });
  if (typeof path !== 'string') return false;
  unwrap(await commands.writerSettingsExport(path));
  return true;
}
/**
 * A23 **Import…**: asks for an exported file, then saves its writer settings (each changed instruction keeps the
 * replaced text among the earlier versions). Resolves to null when the person cancels.
 */
export async function writerSettingsImport(): Promise<WriterSettings | null> {
  const path = await open({ multiple: false, directory: false, title: 'Import writer settings',
    filters: [{ name: 'Writer settings', extensions: ['json'] }] });
  if (typeof path !== 'string') return null;
  return unwrap(await commands.writerSettingsImport(path));
}
/** W4: Reset to default — the language's instruction returns to the shipped one. */
export async function writerInstructionReset(language: string): Promise<WriterSettings> {
  return unwrap(await commands.writerInstructionReset(language));
}
/** W4: Preview what is sent — the request for the built-in example item under the saved settings. */
export async function writerPreview(language: string): Promise<WriterPreview> {
  return unwrap(await commands.writerPreview(language));
}
