import { commands, type NoteDto_Serialize, type ObservationDto_Serialize, type RequestedDto_Serialize } from './bindings';
import { unwrap } from './sessions';

/** W1: what research saw on the session's photos, as the owner's decisions show it, and whether the photos changed. */
export type ProductNote = NoteDto_Serialize;
export type NoteObservation = ObservationDto_Serialize;
/** Whether a request queued a run, and why not (`reason.sentence` is shown). */
export type Requested = RequestedDto_Serialize;

export async function writerNote(sessionId: number): Promise<ProductNote> {
  return unwrap(await commands.writerNote(sessionId));
}
/** W1: Look again — the photo looks run again over the changed photos (no listing search, no new suggestions). */
export async function writerLookAgain(sessionId: number): Promise<Requested> {
  return unwrap(await commands.writerLookAgain(sessionId));
}
