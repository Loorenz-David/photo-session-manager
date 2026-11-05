import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { commands, type HistoryOutcomeDto_Serialize, type HistoryStateDto_Serialize } from './bindings';
import { unwrap } from './sessions';

export type HistoryState = HistoryStateDto_Serialize;
export type HistoryOutcome = HistoryOutcomeDto_Serialize;
export type HistoryChanged = HistoryState & { session_id: number };
export type EditMenuEvent = { action: 'undo' | 'redo' };

export async function historyGet(sessionId: number): Promise<HistoryState> {
  return unwrap(await commands.historyGet(sessionId));
}
export async function historyUndo(sessionId: number): Promise<HistoryOutcome> {
  return unwrap(await commands.historyUndo(sessionId));
}
export async function historyRedo(sessionId: number): Promise<HistoryOutcome> {
  return unwrap(await commands.historyRedo(sessionId));
}
export async function editMenuSet(undo: string | null, redo: string | null): Promise<void> {
  unwrap(await commands.editMenuSet(undo, redo));
}
export async function onHistoryChanged(handler: (state: HistoryChanged) => void): Promise<UnlistenFn> {
  return listen<HistoryChanged>('history_changed', event => handler(event.payload));
}
export async function onEditMenu(handler: (event: EditMenuEvent) => void): Promise<UnlistenFn> {
  return listen<EditMenuEvent>('edit_menu', event => handler(event.payload));
}
