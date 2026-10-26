import { useEffect, useRef, useState } from 'react';
import { Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Command, CommandGroup, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { useDebounced } from '@/lib/keyboard';
import { adoptItem, createItem, nextSku, searchItems, type Item } from '@/services/items';
import { createSession, ServiceError } from '@/services/sessions';

export const ITEM_SEARCH_DEBOUNCE_MS = 200;
export const SKU_CHECK_DEBOUNCE_MS = 250;

type NewSessionDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Called with the session to open once it exists. */
  onOpened: (sessionId: number) => void;
};

type SkuCheck = { sku: string; conflict: Item | null };

/** A search hit whose origin is not `local` comes from (or is linked to) the shop. */
function inShop(item: Item): boolean {
  return item.origin !== 'local';
}

function conflictText(check: SkuCheck): string | null {
  if (!check.conflict) return null;
  return inShop(check.conflict)
    ? `${check.sku} is already in the shop`
    : `${check.sku} is already used by “${check.conflict.title}”`;
}

function errorText(error: unknown): string {
  if (error instanceof ServiceError && error.code === 'sku_collision') {
    return `That SKU collides with ${error.data.existing_sku ?? 'another item'}.`;
  }
  return (error as Error).message || 'Something went wrong.';
}

/** New session (ch04 S-B, S-D; 05B MC-22): find an item, or create a local one with a proposed, editable SKU. */
export function NewSessionDialog({ open, onOpenChange, onOpened }: NewSessionDialogProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<Item[]>([]);
  const [searchedFor, setSearchedFor] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [sku, setSku] = useState('');
  const [skuCheck, setSkuCheck] = useState<SkuCheck | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const skuInput = useRef<HTMLInputElement>(null);

  const title = query.trim();
  const debouncedTitle = useDebounced(title, ITEM_SEARCH_DEBOUNCE_MS);
  const typedSku = sku.trim();
  const debouncedSku = useDebounced(typedSku, SKU_CHECK_DEBOUNCE_MS);

  // Fresh state and a fresh SKU proposal each time the dialog opens.
  useEffect(() => {
    if (!open) return;
    setQuery('');
    setResults([]);
    setSearchedFor(null);
    setCreating(false);
    setSku('');
    setSkuCheck(null);
    setBusy(false);
    setError(null);
    let alive = true;
    nextSku()
      .then(proposal => alive && setSku(current => current || (proposal ?? '')))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [open]);

  useEffect(() => {
    if (!open || !debouncedTitle) {
      setResults([]);
      setSearchedFor(null);
      return;
    }
    let alive = true;
    searchItems(debouncedTitle)
      .then(found => {
        if (!alive) return;
        setResults(found);
        setSearchedFor(debouncedTitle);
      })
      .catch(searchError => alive && setError(errorText(searchError)));
    return () => {
      alive = false;
    };
  }, [open, debouncedTitle]);

  // The SKU in the box is checked against local items and (when configured) the shop before Create enables.
  useEffect(() => {
    if (!open || !creating || !debouncedSku) return;
    let alive = true;
    searchItems(debouncedSku)
      .then(found => alive && setSkuCheck({ sku: debouncedSku, conflict: found.find(item => item.sku === debouncedSku) ?? null }))
      .catch(checkError => alive && setError(errorText(checkError)));
    return () => {
      alive = false;
    };
  }, [open, creating, debouncedSku]);

  useEffect(() => {
    if (creating) skuInput.current?.focus();
  }, [creating]);

  const searchDone = title !== '' && searchedFor === title;
  const exactTitle = results.some(item => item.title.trim().toLowerCase() === title.toLowerCase());
  const showCreate = searchDone && !exactTitle;
  const checked = skuCheck !== null && skuCheck.sku === typedSku;
  const conflict = checked ? conflictText(skuCheck) : null;
  const canCreate = creating && typedSku !== '' && checked && !skuCheck.conflict && !busy;

  async function run(action: () => Promise<number>) {
    setBusy(true);
    setError(null);
    try {
      onOpened(await action());
    } catch (runError) {
      setError(errorText(runError));
    } finally {
      setBusy(false);
    }
  }

  function openExisting(item: Item) {
    void run(async () => {
      const local = item.id === 0 && item.external_id ? await adoptItem(item.external_id) : item;
      return (await createSession(local.id)).id;
    });
  }

  function createAndOpen() {
    if (!canCreate) return;
    void run(async () => {
      const item = await createItem({ sku: typedSku, title });
      return (await createSession(item.id)).id;
    });
  }

  const shopCount = results.filter(inShop).length;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="new-session-dialog" showCloseButton={false}>
        <DialogTitle className="dialog-eyebrow">New session</DialogTitle>
        <DialogDescription className="sr-only">Search items by title or SKU, or create a new item.</DialogDescription>
        <Command shouldFilter={false} className="new-session-command">
          <CommandInput
            value={query}
            onValueChange={value => {
              setQuery(value);
              setCreating(false);
              setError(null);
            }}
            placeholder="Title or SKU"
            aria-label="Search items by title or SKU"
            autoFocus
          />
          {title !== '' && (
            <CommandList className="new-session-results">
              <CommandGroup heading={searchDone ? `${results.length} found${shopCount ? ` · ${shopCount} in the shop` : ''}` : 'Searching…'}>
                {results.map(item => (
                  <CommandItem key={item.external_id ?? item.id} value={`item-${item.external_id ?? item.id}`} onSelect={() => openExisting(item)} disabled={busy} className="result-row">
                    <span className="result-thumb" aria-hidden>
                      {(item.item_type || item.title).slice(0, 1).toUpperCase()}
                    </span>
                    <span className="result-text">
                      <strong>{item.title}</strong>
                      <small>
                        {item.sku}
                        {item.item_type ? ` · ${item.item_type}` : ''} · {inShop(item) ? 'in the shop' : 'local item'}
                      </small>
                    </span>
                  </CommandItem>
                ))}
                {showCreate && (
                  <CommandItem value="create" onSelect={() => setCreating(true)} disabled={busy} className="result-row">
                    <span className="result-thumb" aria-hidden>
                      <Plus />
                    </span>
                    <span className="result-text">
                      <strong>Create “{title}”</strong>
                      <small>New item · saved locally; created on the shop when you publish</small>
                    </span>
                  </CommandItem>
                )}
              </CommandGroup>
            </CommandList>
          )}
        </Command>

        {creating && (
          <form
            className="sku-entry"
            onSubmit={event => {
              event.preventDefault();
              createAndOpen();
            }}
          >
            <label htmlFor="new-item-sku">SKU</label>
            <div className="sku-line">
              <input
                id="new-item-sku"
                ref={skuInput}
                value={sku}
                onChange={event => {
                  setSku(event.target.value);
                  setError(null);
                }}
                placeholder="Required"
                autoComplete="off"
                spellCheck={false}
                aria-invalid={conflict ? true : undefined}
              />
              <Button type="submit" disabled={!canCreate}>
                Create session
              </Button>
            </div>
            {conflict && <p className="sku-message">{conflict}</p>}
          </form>
        )}
        {error && (
          <p className="sku-message" role="alert">
            {error}
          </p>
        )}
      </DialogContent>
    </Dialog>
  );
}
