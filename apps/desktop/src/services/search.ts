import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { commands, type AskAnswerDto_Serialize, type AskFoundDto, type PutTextDto_Serialize, type SavedPageDto, type SelectionAnswerDto_Serialize, type VisualMatchDto_Serialize } from './bindings';
import { unwrap } from './sessions';

/** Owner correction A12: the "Search this photo" window. A18: every search runs in its own tab (`tab`). */
export type VisualMatch = VisualMatchDto_Serialize;
export type SavedPage = SavedPageDto;
export type SelectionAnswer = SelectionAnswerDto_Serialize;
export type AskAnswer = AskAnswerDto_Serialize;
export type AskFound = AskFoundDto;
export type PutText = PutTextDto_Serialize;
/** A18: "Ask AI…" answers in words (`answer`) or sends suggestions to AI review (`suggest`). */
export type AskMode = 'answer' | 'suggest';
export type ZoomAction = 'in' | 'out' | 'reset';

/** Opens the session's search window pinned on the photo (or brings it forward and opens a new search tab for it). */
export async function openSearchWindow(sessionId: number, photoId: number): Promise<void> {
  unwrap(await commands.searchWindowOpen(sessionId, photoId));
}

/** Where the browser area is (CSS pixels of the window), which tab shows there, and whether it shows. */
export async function placeBrowser(sessionId: number, tab: number, rect: { x: number; y: number; width: number; height: number }, visible: boolean): Promise<void> {
  unwrap(await commands.searchWebPlace(sessionId, tab, rect.x, rect.y, rect.width, rect.height, visible));
}

export async function goTo(sessionId: number, tab: number, url: string): Promise<void> {
  unwrap(await commands.searchWebGo(sessionId, tab, url));
}

export async function browserStep(sessionId: number, tab: number, action: 'back' | 'forward' | 'reload'): Promise<void> {
  unwrap(await commands.searchWebStep(sessionId, tab, action));
}

/** A18: the tab's page zoom after `action` from `current`; answers the zoom applied. */
export async function zoomPage(sessionId: number, tab: number, current: number, action: ZoomAction): Promise<number> {
  return unwrap(await commands.searchWebZoom(sessionId, tab, current, action)) ?? 1;
}

/** A18: closes a search tab. */
export async function closeTab(sessionId: number, tab: number): Promise<void> {
  unwrap(await commands.searchTabClose(sessionId, tab));
}

/** Google Lens for the photo in a search tab (`search_lens` events follow). */
export async function startLens(sessionId: number, tab: number, photoId: number): Promise<void> {
  unwrap(await commands.searchLensStart(sessionId, tab, photoId));
}

export async function giveUpLens(sessionId: number, tab: number, why: string): Promise<void> {
  unwrap(await commands.searchLensGiveUp(sessionId, tab, why));
}

/** Back to Google's results page of the tab's search; false when there is none. */
export async function lensResults(sessionId: number, tab: number): Promise<boolean> {
  return unwrap(await commands.searchLensResults(sessionId, tab));
}

export async function readSelection(sessionId: number, tab: number): Promise<SelectionAnswer> {
  return unwrap(await commands.searchWebSelection(sessionId, tab));
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

/** "Put in <field>": the selected text into the field, as the person's own edit; `previous` is what the field held. */
export async function putText(sessionId: number, key: string, text: string): Promise<PutText> {
  return unwrap(await commands.searchPutText(sessionId, key, text));
}

/** A18: "Put '<value>' in <field>" from an answer in words: the same write as `putText`. */
export async function putValue(sessionId: number, key: string, valueJson: string): Promise<PutText> {
  return unwrap(await commands.searchPutValue(sessionId, key, valueJson));
}

/** A14: Undo of "Put in": the field gets back `previous` (its stored text), or is emptied when that is null. */
export async function putBack(sessionId: number, key: string, previous: string | null): Promise<void> {
  unwrap(await commands.searchPutBack(sessionId, key, previous));
}

/** A14: the main window comes forward on the session's AI review tab. */
export async function openAiReview(sessionId: number): Promise<void> {
  unwrap(await commands.searchOpenReview(sessionId));
}

/** A14: the main window's side of `openAiReview`. */
export async function onOpenAiReview(handler: (sessionId: number) => void): Promise<UnlistenFn> {
  return listen<{ session_id: number }>('search_open_review', event => handler(event.payload.session_id));
}

/** "Ask AI…": `suggest` sends suggestions to AI review; `answer` (A18) answers in words and offers the values found. */
export async function askAi(sessionId: number, selection: { text: string; url: string; title: string }, instruction: string, mode: AskMode = 'suggest'): Promise<AskAnswer> {
  return unwrap(await commands.searchAsk(sessionId, selection.text, selection.url, selection.title, instruction, mode));
}

export type PageEvent = { session_id: number; tab: number; url: string; title: string | null; loading: boolean };
export type LensEvent =
  | { session_id: number; tab: number; state: 'idle' | 'opening' | 'consent' | 'uploading' }
  | { session_id: number; tab: number; state: 'results'; url: string }
  | { session_id: number; tab: number; state: 'failed'; why: string };

/** A tab's page (address, title, loading), a tab's Google Lens route phase, a photo to search (from the main window) and the View menu's zoom keys, for one session. */
export async function onSearchEvents(
  sessionId: number,
  handlers: { page: (event: PageEvent) => void; lens: (event: LensEvent) => void; photo: (photoId: number) => void; zoom?: (action: ZoomAction) => void },
): Promise<UnlistenFn> {
  const offs = await Promise.all([
    listen<PageEvent>('search_web_page', event => { if (event.payload.session_id === sessionId) handlers.page(event.payload); }),
    listen<LensEvent>('search_lens', event => { if (event.payload.session_id === sessionId) handlers.lens(event.payload); }),
    listen<{ session_id: number; photo_id: number }>('search_photo', event => { if (event.payload.session_id === sessionId) handlers.photo(event.payload.photo_id); }),
    listen<{ session_id: number; action: ZoomAction }>('search_zoom', event => { if (event.payload.session_id === sessionId) handlers.zoom?.(event.payload.action); }),
  ]);
  return () => offs.forEach(off => off());
}
