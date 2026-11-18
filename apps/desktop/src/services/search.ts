import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { commands, type AskAnswerDto, type AttributeValueDto_Serialize, type SavedPageDto, type SelectionAnswerDto_Serialize, type VisualMatchDto_Serialize } from './bindings';
import { unwrap } from './sessions';

/** Owner correction A12: the "Search this photo" window. */
export type VisualMatch = VisualMatchDto_Serialize;
export type SavedPage = SavedPageDto;
export type SelectionAnswer = SelectionAnswerDto_Serialize;
export type AskAnswer = AskAnswerDto;

/** Opens the session's search window pinned on the photo (or brings it forward on that photo). */
export async function openSearchWindow(sessionId: number, photoId: number): Promise<void> {
  unwrap(await commands.searchWindowOpen(sessionId, photoId));
}

/** Where the browser area is (CSS pixels of the window) and whether the page shows. */
export async function placeBrowser(sessionId: number, rect: { x: number; y: number; width: number; height: number }, visible: boolean): Promise<void> {
  unwrap(await commands.searchWebPlace(sessionId, rect.x, rect.y, rect.width, rect.height, visible));
}

export async function goTo(sessionId: number, url: string): Promise<void> {
  unwrap(await commands.searchWebGo(sessionId, url));
}

export async function browserStep(sessionId: number, action: 'back' | 'forward' | 'reload'): Promise<void> {
  unwrap(await commands.searchWebStep(sessionId, action));
}

/** Google Lens for the photo in the window's browser (`search_lens` events follow). */
export async function startLens(sessionId: number, photoId: number): Promise<void> {
  unwrap(await commands.searchLensStart(sessionId, photoId));
}

export async function giveUpLens(sessionId: number, why: string): Promise<void> {
  unwrap(await commands.searchLensGiveUp(sessionId, why));
}

/** Back to Google's results page; false when there is none. */
export async function lensResults(sessionId: number): Promise<boolean> {
  return unwrap(await commands.searchLensResults(sessionId));
}

export async function readSelection(sessionId: number): Promise<SelectionAnswer> {
  return unwrap(await commands.searchWebSelection(sessionId));
}

/** The matches research already found for the session (no cost). */
export async function researchMatches(sessionId: number): Promise<VisualMatch[]> {
  return unwrap(await commands.searchMatches(sessionId));
}

/** One new visual search of the photo (a counted research call). */
export async function visualSearch(sessionId: number, photoId: number): Promise<VisualMatch[]> {
  return unwrap(await commands.searchVisual(sessionId, photoId));
}

export async function savedPages(itemId: number): Promise<SavedPage[]> {
  return unwrap(await commands.searchPages(itemId));
}

export async function savePage(itemId: number, url: string, title: string): Promise<SavedPage> {
  return unwrap(await commands.searchPageSave(itemId, url, title));
}

export async function removePage(pageId: number): Promise<void> {
  unwrap(await commands.searchPageRemove(pageId));
}

/** "Put in <field>": the selected text into the field, as the person's own edit. */
export async function putText(sessionId: number, key: string, text: string): Promise<AttributeValueDto_Serialize> {
  return unwrap(await commands.searchPutText(sessionId, key, text));
}

/** "Ask AI…": the answer becomes AI suggestions (AI review tab). */
export async function askAi(sessionId: number, selection: { text: string; url: string; title: string }, instruction: string): Promise<AskAnswer> {
  return unwrap(await commands.searchAsk(sessionId, selection.text, selection.url, selection.title, instruction));
}

export type PageEvent = { session_id: number; url: string; title: string | null; loading: boolean };
export type LensEvent =
  | { session_id: number; state: 'idle' | 'opening' | 'consent' | 'uploading' }
  | { session_id: number; state: 'results'; url: string }
  | { session_id: number; state: 'failed'; why: string };

/** The page webview's page (address, title, loading) and the Google Lens route's phase, for one session. */
export async function onSearchEvents(sessionId: number, handlers: { page: (event: PageEvent) => void; lens: (event: LensEvent) => void; photo: (photoId: number) => void }): Promise<UnlistenFn> {
  const offs = await Promise.all([
    listen<PageEvent>('search_web_page', event => { if (event.payload.session_id === sessionId) handlers.page(event.payload); }),
    listen<LensEvent>('search_lens', event => { if (event.payload.session_id === sessionId) handlers.lens(event.payload); }),
    listen<{ session_id: number; photo_id: number }>('search_photo', event => { if (event.payload.session_id === sessionId) handlers.photo(event.payload.photo_id); }),
  ]);
  return () => offs.forEach(off => off());
}
