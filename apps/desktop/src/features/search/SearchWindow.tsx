import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { ArrowLeft, ArrowRight, Bookmark, LayoutGrid, Loader2, RotateCw, Search, Sparkles, X } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Toaster } from '@/components/ui/sonner';
import { cn } from '@/lib/utils';
import { getForm, type FormField } from '@/services/attributes';
import { getSessionView, type Photo, type SessionView } from '@/services/photos';
import { previewUrl } from '@/services/sessions';
import {
  askAi,
  browserStep,
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
type Selected = { text: string; url: string; title: string };

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
  useTheme();
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
  const [instruction, setInstruction] = useState('');
  const [asking, setAsking] = useState(false);
  const [other, setOther] = useState('');
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
  const place = useCallback(() => {
    const area = browserArea.current;
    if (!area) return;
    const rect = area.getBoundingClientRect();
    void placeBrowser(sessionId, { x: rect.left, y: rect.top, width: rect.width, height: rect.height }, mode === 'web').catch(() => undefined);
  }, [sessionId, mode]);
  useLayoutEffect(() => {
    place();
    const area = browserArea.current;
    if (!area || typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(() => place());
    observer.observe(area);
    window.addEventListener('resize', place);
    return () => { observer.disconnect(); window.removeEventListener('resize', place); };
  }, [place]);

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

  async function put(field: FormField) {
    if (!selected) return;
    try {
      await putText(sessionId, field.definition.key, selected.text);
      toast.success(`Put in ${field.definition.label}`);
    } catch (error) {
      toast.error((error as Error).message);
    }
  }

  async function ask() {
    if (!selected) return;
    setAsking(true);
    try {
      const answer = await askAi(sessionId, selected, instruction);
      if (answer.suggested.length > 0) {
        const names = answer.suggested.map(s => `${s.label}: ${s.value}`).join(' · ');
        toast.success(`AI suggested ${answer.suggested.length === 1 ? '1 field' : `${answer.suggested.length} fields`} — check ${answer.suggested.length === 1 ? 'it' : 'them'} in AI review`, { description: names });
      } else if (answer.not_written.length > 0) {
        toast.message('Nothing new to suggest', { description: answer.not_written.map(([label, why]) => `${label}: ${why}`).join(' · ') });
      } else {
        toast.message('AI found nothing for that in the selected text');
      }
    } catch (error) {
      toast.error(`Ask AI: ${(error as Error).message}`);
    } finally {
      setAsking(false);
    }
  }

  const note = lensNote(lens, waiting);
  const current = page?.url;

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
        <div className="flex shrink-0 items-center gap-1 border-b px-2 py-1.5">
          <Button variant="ghost" size="icon-sm" aria-label="Back" title="Back" onClick={() => void browserStep(sessionId, 'back').catch(() => undefined)}><ArrowLeft /></Button>
          <Button variant="ghost" size="icon-sm" aria-label="Forward" title="Forward" onClick={() => void browserStep(sessionId, 'forward').catch(() => undefined)}><ArrowRight /></Button>
          <Button variant="ghost" size="icon-sm" aria-label="Reload" title="Reload" onClick={() => void browserStep(sessionId, 'reload').catch(() => undefined)}>
            {page?.loading ? <Loader2 className="animate-spin" /> : <RotateCw />}
          </Button>
          <Button variant="outline" size="sm" onClick={() => void backToResults()}><LayoutGrid /> Results</Button>
          <div className="mx-2 min-w-0 flex-1 truncate text-xs" title={current}>
            {mode === 'list'
              ? <span className="text-muted-foreground">Matches for this photo</span>
              : page
                ? <><span className="font-semibold">{hostOf(page.url)}</span>{page.title ? <span className="text-muted-foreground"> — {page.title}</span> : null}</>
                : <span className="text-muted-foreground">…</span>}
          </div>
          <Button variant="outline" size="sm" disabled={mode !== 'web' || !page || !/^https?:/.test(page.url)} onClick={() => void saveCurrent()}><Bookmark /> Save page</Button>
          <Button variant="ghost" size="sm" title="Search Google again for the pinned photo" onClick={() => void searchGoogle(photoId)}><Search /> Google</Button>
        </div>

        {pages.length > 0 && (
          <div className="flex shrink-0 gap-1 overflow-x-auto border-b px-2 py-1" aria-label="Saved pages">
            {pages.map(saved => (
              <div key={saved.id} className={cn('flex max-w-[220px] shrink-0 items-center rounded-md border text-xs', mode === 'web' && current === saved.url ? 'border-primary bg-primary/10' : 'bg-muted/50')}>
                <button type="button" className="min-w-0 truncate px-2 py-1 text-left" title={saved.url} onClick={() => void open(saved.url)}>
                  <span className="font-semibold">{hostOf(saved.url)}</span>{saved.title ? ` · ${saved.title}` : ''}
                </button>
                <button type="button" aria-label={`Remove saved page ${hostOf(saved.url)}`} className="px-1 text-muted-foreground hover:text-foreground" onClick={() => void unsave(saved)}><X className="size-3" /></button>
              </div>
            ))}
          </div>
        )}

        {note && mode === 'web' && <div className="shrink-0 border-b bg-muted/50 px-3 py-1.5 text-xs" role="status">{note}</div>}

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
                      <button key={match.url} type="button" onClick={() => void open(match.url)} className="flex flex-col gap-1 rounded-md border p-2 text-left hover:bg-muted">
                        {match.thumbnail ? <img src={match.thumbnail} alt="" className="aspect-square w-full rounded object-cover" /> : <div className="aspect-square w-full rounded bg-muted" />}
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

        <div className="flex shrink-0 flex-col gap-2 border-t bg-muted/30 px-3 py-2" aria-label="Selected text">
          {selected ? (
            <>
              <p className="line-clamp-2 text-xs"><span className="font-semibold">Selected:</span> “{selected.text}” <span className="text-muted-foreground">— {hostOf(selected.url)}</span></p>
              <div className="flex flex-wrap items-center gap-1.5">
                {quick.map(field => (
                  <Button key={field.definition.key} size="sm" variant="secondary" onClick={() => void put(field)}>Put in {field.definition.label}</Button>
                ))}
                {rest.length > 0 && (
                  <select
                    aria-label="Put in another field"
                    className="h-7 rounded-md border bg-background px-2 text-xs"
                    value={other}
                    onChange={event => {
                      const field = rest.find(f => f.definition.key === event.target.value);
                      setOther('');
                      if (field) void put(field);
                    }}
                  >
                    <option value="">{quick.length > 0 ? 'Other field…' : 'Put in field…'}</option>
                    {rest.map(field => <option key={field.definition.key} value={field.definition.key}>{field.definition.label}</option>)}
                  </select>
                )}
                <div className="ml-auto flex min-w-[320px] flex-1 items-center gap-1.5">
                  <Input
                    aria-label="Instruction for AI"
                    className="h-7 text-xs"
                    placeholder="Ask AI… e.g. find the production year"
                    maxLength={300}
                    value={instruction}
                    onChange={event => setInstruction(event.target.value)}
                    onKeyDown={event => { if (event.key === 'Enter') void ask(); }}
                  />
                  <Button size="sm" disabled={asking} onClick={() => void ask()}>{asking ? <Loader2 className="animate-spin" /> : <Sparkles />} Ask AI</Button>
                </div>
              </div>
            </>
          ) : (
            <p className="text-xs text-muted-foreground">{refused ?? 'Select text on the page to put it in a field or ask AI about it.'}</p>
          )}
        </div>
      </main>
      <Toaster />
    </div>
  );
}
