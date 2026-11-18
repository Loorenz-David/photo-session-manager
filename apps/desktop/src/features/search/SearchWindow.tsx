import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { ArrowLeft, ArrowRight, Bookmark, BookmarkCheck, Globe, LayoutGrid, Loader2, RotateCw, ScanSearch, X } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Toaster } from '@/components/ui/sonner';
import { cn } from '@/lib/utils';
import { getForm, type FormField } from '@/services/attributes';
import { getSessionView, type Photo, type SessionView } from '@/services/photos';
import { previewUrl } from '@/services/sessions';
import {
  askAi,
  browserStep,
  openAiReview,
  putBack,
  giveUpLens,
  goTo,
  lensResults,
  onSearchEvents,
  placeBrowser,
  putText,
  readSelection,
  removePage,
  researchMatches,
  savedPages,
  savePage,
  startLens,
  visualSearch,
  type LensEvent,
  type SavedPage,
  type VisualMatch,
} from '@/services/search';
import { Thumb } from '@/features/workspace/PhotoTile';
import { orientationTransform } from '@/features/workspace/StageImage';
import { displayOrder } from '@/features/workspace/order';
import { useTheme } from '@/features/shell/ThemeToggle';
import { SelectionBar, type AskOutcome, type Selected } from './SelectionBar';

/** How often the window asks the page what is selected (ms). */
export const SELECTION_POLL_MS = 700;
/** How long the window waits for Google's results before it shows its own list (ms). */
export const LENS_WAIT_MS = 30_000;

/** `#/search/<session>/<photo>`: the search window's route (A12). */
export function searchRoute(hash: string): { sessionId: number; photoId: number } | null {
  const match = hash.match(/^#\/search\/(\d+)\/(\d+)$/);
  return match ? { sessionId: Number(match[1]), photoId: Number(match[2]) } : null;
}

/** The host of a web address, for labels. */
export function hostOf(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, '');
  } catch {
    return url;
  }
}

const NOT_FILLABLE = new Set(['media_list', 'reference', 'rich_text', 'money', 'rating', 'tags']);
const FIRST = /manufactur|designer|maker|brand|producer/i;

/** The fields "Put in" offers: writable fields that take text, Manufacturer and Designer (and the like) first. */
export function putFields(fields: FormField[]): FormField[] {
  const usable = fields.filter(f => f.definition.writable && !NOT_FILLABLE.has(f.definition.core_type));
  return [...usable.filter(f => FIRST.test(f.definition.label)), ...usable.filter(f => !FIRST.test(f.definition.label))];
}

type Mode = 'web' | 'list';
type Page = { url: string; title: string; loading: boolean };

/**
 * A13: the browser area's place as sent to the app: the element's box in this page's CSS pixels,
 * which are logical points at any screen scale (never multiplied by `devicePixelRatio`), rounded
 * to whole points so the page webview never sits a fraction off.
 */
export function browserBounds(rect: { left: number; top: number; width: number; height: number }): { x: number; y: number; width: number; height: number } {
  const whole = (v: number) => (Number.isFinite(v) ? Math.round(v) : 0);
  return { x: whole(rect.left), y: whole(rect.top), width: Math.max(1, whole(rect.width)), height: Math.max(1, whole(rect.height)) };
}

function PinnedPhoto({ photo }: { photo: Photo }) {
  if (photo.adjusted_preview_path) {
    return <img src={previewUrl(photo.adjusted_preview_path)} alt="Pinned photo" className="max-h-full max-w-full object-contain" draggable={false} />;
  }
  if (photo.original_preview_path) {
    return (
      <img
        src={previewUrl(photo.original_preview_path)}
        alt="Pinned photo"
        className="max-h-full max-w-full object-contain"
        style={{ transform: orientationTransform(photo.orientation) }}
        draggable={false}
      />
    );
  }
  return <p className="text-xs text-muted-foreground">No preview yet</p>;
}

function lensNote(lens: LensEvent['state'] | null, waiting: boolean): string | null {
  if (lens === 'consent') return 'Google asks about cookies. Make your choice in the page — the search starts right after.';
  if (waiting || lens === 'opening' || lens === 'uploading') return 'Searching Google for this photo…';
  return null;
}

/**
 * The "Search this photo" window (owner correction A12): the photo pinned on the left, a browser on the right with
 * Google's results for it, saved pages as tabs, and what is selected on the page with its actions. The page itself
 * is a separate webview placed over the browser area; this page never reads it except for the selected text.
 */
export function SearchWindow({ sessionId, initialPhotoId }: { sessionId: number; initialPhotoId: number }) {
  const [theme] = useTheme();
  const [view, setView] = useState<SessionView | null>(null);
  const [fields, setFields] = useState<FormField[]>([]);
  const [photoId, setPhotoId] = useState(initialPhotoId);
  const [mode, setMode] = useState<Mode>('web');
  const [page, setPage] = useState<Page | null>(null);
  const [lens, setLens] = useState<LensEvent['state'] | null>(null);
  const [waiting, setWaiting] = useState(false);
  const [matches, setMatches] = useState<VisualMatch[] | null>(null);
  const [matchesOrigin, setMatchesOrigin] = useState<'research' | 'search' | null>(null);
  const [searching, setSearching] = useState(false);
  const [pages, setPages] = useState<SavedPage[]>([]);
  const [selected, setSelected] = useState<Selected | null>(null);
  const [refused, setRefused] = useState<string | null>(null);
  const browserArea = useRef<HTMLDivElement>(null);
  const searchedPhotos = useRef(new Set<number>());
  const photoIdRef = useRef(photoId);
  photoIdRef.current = photoId;

  const itemId = view?.item?.id;
  const photos = useMemo(() => {
    if (!view) return [];
    const order = displayOrder(view.photos);
    return [...order.main, ...order.unreviewed, ...order.condition];
  }, [view]);
  const pinned = photos.find(p => p.id === photoId) ?? view?.photos.find(p => p.id === photoId) ?? null;
  const offered = useMemo(() => putFields(fields), [fields]);
  const quick = offered.filter(f => FIRST.test(f.definition.label)).slice(0, 3);
  const rest = offered.filter(f => !quick.includes(f));

  useEffect(() => {
    let alive = true;
    getSessionView(sessionId)
      .then(next => {
        if (!alive) return;
        setView(next);
        document.title = `Search this photo — ${next.item ? `${next.item.item_type || next.item.title} · ${next.item.sku}` : ''}`;
        if (next.item) {
          void getForm(next.item.id).then(form => alive && setFields(form.groups.flatMap(g => g.fields))).catch(() => undefined);
          void savedPages(next.item.id).then(list => alive && setPages(list)).catch(() => undefined);
        }
      })
      .catch(error => toast.error(`Could not load the session: ${(error as Error).message}`));
    return () => { alive = false; };
  }, [sessionId]);

  // The browser area's place, so the page webview sits exactly over it (hidden while the list shows).
  // A13: CSS pixels of this page (logical points, never device pixels); the app adds where this
  // page sits in the window. Sent only when the place changed.
  const lastPlace = useRef('');
  const place = useCallback(() => {
    const area = browserArea.current;
    if (!area) return;
    const bounds = browserBounds(area.getBoundingClientRect());
    const visible = mode === 'web';
    const key = JSON.stringify([bounds, visible]);
    if (key === lastPlace.current) return;
    lastPlace.current = key;
    void placeBrowser(sessionId, bounds, visible).catch(() => { lastPlace.current = ''; });
  }, [sessionId, mode]);
  useLayoutEffect(() => {
    place();
    const area = browserArea.current;
    if (!area || typeof ResizeObserver === 'undefined') return;
    // The area moves when a bar above it appears, grows or goes (saved pages, the Google note,
    // the selection bar): watch every row of the browser column, not only the area.
    const observer = new ResizeObserver(() => place());
    observer.observe(area);
    const column = area.parentElement?.parentElement;
    if (column) for (const row of Array.from(column.children)) observer.observe(row);
    window.addEventListener('resize', place);
    return () => { observer.disconnect(); window.removeEventListener('resize', place); };
  }, [place, pages.length, lensNote(lens, waiting) !== null, selected !== null]);

  const showList = useCallback(async (forPhoto: number) => {
    setMode('list');
    setSearching(true);
    try {
      const known = await researchMatches(sessionId).catch(() => [] as VisualMatch[]);
      if (known.length > 0) {
        setMatches(known);
        setMatchesOrigin('research');
      } else if (!searchedPhotos.current.has(forPhoto)) {
        searchedPhotos.current.add(forPhoto);
        setMatches(await visualSearch(sessionId, forPhoto));
        setMatchesOrigin('search');
      } else {
        setMatches([]);
      }
    } catch (error) {
      setMatches([]);
      toast.error(`Could not search: ${(error as Error).message}`);
    } finally {
      setSearching(false);
    }
  }, [sessionId]);

  const searchGoogle = useCallback(async (forPhoto: number) => {
    setMode('web');
    setLens('opening');
    setWaiting(true);
    try {
      await startLens(sessionId, forPhoto);
    } catch (error) {
      setWaiting(false);
      toast.error(`Could not open Google: ${(error as Error).message}`);
      void showList(forPhoto);
    }
  }, [sessionId, showList]);

  // Google's route: events say where it stands; without results in time the window shows its own list.
  useEffect(() => {
    let off: (() => void) | undefined;
    let alive = true;
    onSearchEvents(sessionId, {
      page: event => setPage(current => ({
        url: event.url,
        title: event.title ?? (current && current.url === event.url ? current.title : ''),
        loading: event.loading,
      })),
      lens: event => {
        setLens(event.state);
        if (event.state === 'results' || event.state === 'consent') setWaiting(false);
        if (event.state === 'failed') {
          setWaiting(false);
          void showList(photoIdRef.current);
        }
      },
      photo: next => setPhotoId(next),
    })
      .then(unlisten => { if (alive) off = unlisten; else unlisten(); })
      .catch(() => undefined);
    return () => { alive = false; off?.(); };
  }, [sessionId, showList]);

  useEffect(() => {
    void searchGoogle(photoId);
  }, [photoId, searchGoogle]);

  useEffect(() => {
    if (!waiting) return;
    const timer = window.setTimeout(() => {
      setWaiting(false);
      void giveUpLens(sessionId, 'no results in 30 s').catch(() => undefined);
      void showList(photoIdRef.current);
    }, LENS_WAIT_MS);
    return () => window.clearTimeout(timer);
  }, [waiting, sessionId, showList]);

  // What is selected on the page, asked again and again while the page shows.
  useEffect(() => {
    if (mode !== 'web') return;
    let alive = true;
    const ask = () => {
      readSelection(sessionId)
        .then(answer => {
          if (!alive) return;
          setRefused(answer.refused);
          setSelected(current => {
            const next = answer.selection;
            if (!next) return null;
            return current && current.text === next.text && current.url === next.url ? current : next;
          });
        })
        .catch(() => undefined);
    };
    ask();
    const timer = window.setInterval(ask, SELECTION_POLL_MS);
    return () => { alive = false; window.clearInterval(timer); };
  }, [sessionId, mode]);

  async function open(url: string) {
    setMode('web');
    try {
      await goTo(sessionId, url);
    } catch (error) {
      toast.error((error as Error).message);
    }
  }

  async function backToResults() {
    if (await lensResults(sessionId).catch(() => false)) {
      setMode('web');
      return;
    }
    if (matches) setMode('list');
    else void searchGoogle(photoId);
  }

  async function saveCurrent() {
    if (!page || itemId === undefined) return;
    try {
      const saved = await savePage(itemId, page.url, page.title);
      setPages(current => (current.some(p => p.id === saved.id) ? current.map(p => (p.id === saved.id ? saved : p)) : [...current, saved]));
      toast.success('Page saved');
    } catch (error) {
      toast.error(`Could not save the page: ${(error as Error).message}`);
    }
  }

  async function unsave(saved: SavedPage) {
    try {
      await removePage(saved.id);
      setPages(current => current.filter(p => p.id !== saved.id));
    } catch (error) {
      toast.error((error as Error).message);
    }
  }

  /** "Put in <field>", confirmed by a toast whose Undo gives the field back what it held. */
  async function put(field: FormField, selection: Selected) {
    const { key, label } = field.definition;
    try {
      const done = await putText(sessionId, key, selection.text);
      toast.success(`Put in ${label}`, {
        description: selection.text.length > 80 ? `${selection.text.slice(0, 80)}…` : selection.text,
        action: {
          label: 'Undo',
          onClick: () => {
            putBack(sessionId, key, done.previous)
              .then(() => toast.message(done.previous === null ? `${label} is empty again` : `${label} has its previous value again`))
              .catch(error => toast.error(`Could not undo: ${(error as Error).message}`));
          },
        },
      });
    } catch (error) {
      toast.error((error as Error).message);
    }
  }

  /** "Ask AI…": the answer for the bar to show (null when the call failed; a toast says why). */
  async function ask(selection: Selected, instruction: string): Promise<AskOutcome | null> {
    try {
      const answer = await askAi(sessionId, selection, instruction);
      return { suggested: answer.suggested, notWritten: answer.not_written };
    } catch (error) {
      toast.error(`Ask AI: ${(error as Error).message}`);
      return null;
    }
  }

  const note = lensNote(lens, waiting);
  const current = page?.url;
  const currentSaved = mode === 'web' && current !== undefined && pages.some(p => p.url === current);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      <aside className="flex w-[380px] shrink-0 flex-col gap-3 border-r p-3">
        <div className="text-xs text-muted-foreground">
          {view?.item ? <><span className="font-semibold text-foreground">{view.item.item_type || view.item.title}</span> · {view.item.sku}</> : 'Loading…'}
        </div>
        <div className="flex min-h-0 flex-1 items-center justify-center overflow-hidden rounded-md bg-muted" data-testid="pinned-photo" data-photo-id={pinned?.id}>
          {pinned ? <PinnedPhoto photo={pinned} /> : null}
        </div>
        {photos.length > 1 && (
          <div className="flex shrink-0 gap-1.5 overflow-x-auto pb-1" aria-label="Other photos">
            {photos.map(photo => (
              <button
                key={photo.id}
                type="button"
                aria-label={`Pin photo ${photo.id}`}
                aria-pressed={photo.id === photoId}
                onClick={() => setPhotoId(photo.id)}
                className={cn('size-14 shrink-0 overflow-hidden rounded border-2 bg-muted [&_img]:size-full [&_img]:object-cover', photo.id === photoId ? 'border-primary' : 'border-transparent')}
              >
                <Thumb photo={photo} />
              </button>
            ))}
          </div>
        )}
      </aside>

      <main className="flex min-w-0 flex-1 flex-col">
        <div className="flex h-12 shrink-0 items-center gap-2 border-b px-2">
          <div className="flex shrink-0 items-center rounded-lg bg-muted/70 p-0.5">
            <Button variant="ghost" size="icon-sm" aria-label="Back" title="Back" onClick={() => void browserStep(sessionId, 'back').catch(() => undefined)}><ArrowLeft /></Button>
            <Button variant="ghost" size="icon-sm" aria-label="Forward" title="Forward" onClick={() => void browserStep(sessionId, 'forward').catch(() => undefined)}><ArrowRight /></Button>
            <Button variant="ghost" size="icon-sm" aria-label="Reload" title="Reload" onClick={() => void browserStep(sessionId, 'reload').catch(() => undefined)}><RotateCw /></Button>
          </div>
          <Button variant="secondary" size="sm" title="Back to the results for the pinned photo" onClick={() => void backToResults()}><LayoutGrid /> Results</Button>
          <div className="flex h-8 min-w-0 flex-1 items-center gap-2 rounded-full bg-muted/60 px-3 text-xs" title={current} data-testid="address">
            {page?.loading && mode === 'web' ? <Loader2 className="size-3.5 shrink-0 animate-spin text-muted-foreground" /> : <Globe className="size-3.5 shrink-0 text-muted-foreground" />}
            <span className="min-w-0 truncate">
              {mode === 'list'
                ? <span className="text-muted-foreground">Matches for this photo</span>
                : page
                  ? <><span className="font-semibold">{hostOf(page.url)}</span>{page.title ? <span className="text-muted-foreground"> — {page.title}</span> : null}</>
                  : <span className="text-muted-foreground">…</span>}
            </span>
          </div>
          {currentSaved ? (
            <Button variant="secondary" size="sm" disabled title="This page is saved"><BookmarkCheck /> Saved</Button>
          ) : (
            <Button variant="outline" size="sm" title="Keep this page as a tab for this item" disabled={mode !== 'web' || !page || !/^https?:/.test(page.url)} onClick={() => void saveCurrent()}><Bookmark /> Save page</Button>
          )}
          <Button variant="ghost" size="icon-sm" aria-label="Search Google again" title="Search Google again for the pinned photo" onClick={() => void searchGoogle(photoId)}><ScanSearch /></Button>
        </div>

        {pages.length > 0 && (
          <div className="flex h-10 shrink-0 items-center gap-1.5 overflow-x-auto border-b px-2" aria-label="Saved pages">
            <span className="shrink-0 px-1 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">Saved</span>
            {pages.map(saved => {
              const on = mode === 'web' && current === saved.url;
              return (
                <div
                  key={saved.id}
                  data-active={on || undefined}
                  className={cn(
                    'group flex h-7 max-w-[240px] shrink-0 items-center rounded-full border pr-0.5 pl-2.5 text-xs transition-colors',
                    on ? 'border-transparent bg-foreground text-background' : 'bg-background hover:bg-muted',
                  )}
                >
                  <button type="button" className="min-w-0 truncate text-left" title={`${saved.title ? `${saved.title}\n` : ''}${saved.url}`} onClick={() => void open(saved.url)}>
                    <span className="font-semibold">{hostOf(saved.url)}</span>
                    {saved.title ? <span className={on ? 'opacity-70' : 'text-muted-foreground'}> · {saved.title}</span> : null}
                  </button>
                  <button
                    type="button"
                    aria-label={`Remove saved page ${hostOf(saved.url)}`}
                    title="Remove from saved"
                    className="ml-1 flex size-5 shrink-0 items-center justify-center rounded-full opacity-60 group-hover:opacity-100 hover:bg-current/15"
                    onClick={() => void unsave(saved)}
                  >
                    <X className="size-3" />
                  </button>
                </div>
              );
            })}
          </div>
        )}

        {note && mode === 'web' && (
          <div className="flex h-9 shrink-0 items-center gap-2 border-b bg-(--ai-muted)/50 px-3 text-xs text-(--ai-foreground)" role="status">
            {lens === 'consent' ? null : <Loader2 className="size-3.5 shrink-0 animate-spin" />}
            {note}
          </div>
        )}

        <div className="relative min-h-0 flex-1">
          {/* The page webview is placed exactly over this area. */}
          <div ref={browserArea} className="absolute inset-0" data-testid="browser-area" />
          {mode === 'list' && (
            <div className="absolute inset-0 overflow-y-auto p-4" data-testid="match-list">
              {searching ? (
                <p className="flex items-center gap-2 text-sm text-muted-foreground"><Loader2 className="size-4 animate-spin" /> Looking for matches…</p>
              ) : (
                <>
                  <p className="mb-3 text-xs text-muted-foreground">
                    {matchesOrigin === 'research' ? 'Google could not be used here. These are the matches research found for this item.' : 'Google could not be used here. These are visual matches for the pinned photo.'}{' '}
                    <button type="button" className="underline" onClick={() => void searchGoogle(photoId)}>Try Google again</button>
                  </p>
                  {matches && matches.length === 0 && <p className="text-sm text-muted-foreground">No matches.</p>}
                  <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3">
                    {(matches ?? []).map(match => (
                      <button key={match.url} type="button" onClick={() => void open(match.url)} className="flex flex-col gap-1 rounded-lg border p-2 text-left shadow-(--shadow-sm) transition-colors hover:bg-muted">
                        {match.thumbnail ? <img src={match.thumbnail} alt="" className="aspect-square w-full rounded-md object-cover" /> : <div className="aspect-square w-full rounded-md bg-muted" />}
                        <span className="line-clamp-2 text-xs font-semibold">{match.title || hostOf(match.url)}</span>
                        <span className="truncate text-[11px] text-muted-foreground">{match.source || hostOf(match.url)}</span>
                      </button>
                    ))}
                  </div>
                </>
              )}
            </div>
          )}
        </div>

        <SelectionBar
          selected={selected}
          refused={refused}
          quick={quick}
          rest={rest}
          onPut={put}
          onAsk={ask}
          onOpenReview={() => void openAiReview(sessionId).catch(error => toast.error((error as Error).message))}
        />
      </main>
      {/* Toasts sit over the photo panel: anything drawn over the page area would be hidden under the page. */}
      <Toaster theme={theme} position="bottom-left" offset={12} visibleToasts={3} closeButton />
    </div>
  );
}
