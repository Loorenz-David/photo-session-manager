import {
  commands,
  type NoteDto_Serialize,
  type ObservationDto_Serialize,
  type RequestedDto_Serialize,
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
