import { useEffect, useState } from 'react';
import { Command, CommandGroup, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { ITEM_SEARCH_DEBOUNCE_MS } from '@/features/sessions/NewSessionDialog';
import { useDebounced } from '@/lib/keyboard';
import { searchItems, type Item } from '@/services/items';
import { listSessions } from '@/services/sessions';
import { acceptsPhotos, type SessionRow } from './SessionsPanel';

type PickerProps<T> = {
  open: boolean;
  title: string;
  onOpenChange: (open: boolean) => void;
  onPick: (picked: T) => void;
};

/** "Move to session…" / "Copy to session…": pick a target session; published sessions are listed but disabled (MC-2). */
export function SessionPickerDialog({ open, title, onOpenChange, onPick, excludeSessionId }: PickerProps<SessionRow> & { excludeSessionId: number }) {
  const [query, setQuery] = useState('');
  const [rows, setRows] = useState<SessionRow[]>([]);
  const debounced = useDebounced(query.trim(), ITEM_SEARCH_DEBOUNCE_MS);

  useEffect(() => {
    if (open) setQuery('');
  }, [open]);

  useEffect(() => {
    if (!open) return;
    let alive = true;
    listSessions({ filter: 'all', query: debounced || null })
      .then(list => alive && setRows(list.groups.flatMap(group => group.rows).filter(row => row.id !== excludeSessionId)))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [open, debounced, excludeSessionId]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="new-session-dialog" showCloseButton={false}>
        <DialogTitle className="dialog-eyebrow">{title}</DialogTitle>
        <DialogDescription className="sr-only">Search sessions by title or SKU.</DialogDescription>
        <Command shouldFilter={false} className="new-session-command">
          <CommandInput value={query} onValueChange={setQuery} placeholder="Title or SKU" aria-label="Search sessions by title or SKU" autoFocus />
          <CommandList className="new-session-results">
            <CommandGroup heading="Sessions">
              {rows.map(row => (
                <CommandItem
                  key={row.id}
                  value={`session-${row.id}`}
                  disabled={!acceptsPhotos(row)}
                  onSelect={() => onPick(row)}
                  className="result-row"
                >
                  <span className="result-text">
                    <strong>{row.item_type || row.title}</strong>
                    <small>
                      {row.sku}
                      {acceptsPhotos(row) ? '' : ' · published, cannot receive photos'}
                    </small>
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </DialogContent>
    </Dialog>
  );
}

/** "Assign product" / "Assign to product…": the New session dialog's item search, picking an existing item. */
export function ItemPickerDialog({ open, title, onOpenChange, onPick }: PickerProps<Item>) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<Item[]>([]);
  const debounced = useDebounced(query.trim(), ITEM_SEARCH_DEBOUNCE_MS);

  useEffect(() => {
    if (!open) return;
    setQuery('');
    setResults([]);
  }, [open]);

  useEffect(() => {
    if (!open || !debounced) {
      setResults([]);
      return;
    }
    let alive = true;
    searchItems(debounced)
      .then(found => alive && setResults(found))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [open, debounced]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="new-session-dialog" showCloseButton={false}>
        <DialogTitle className="dialog-eyebrow">{title}</DialogTitle>
        <DialogDescription className="sr-only">Search items by title or SKU.</DialogDescription>
        <Command shouldFilter={false} className="new-session-command">
          <CommandInput value={query} onValueChange={setQuery} placeholder="Title or SKU" aria-label="Search items by title or SKU" autoFocus />
          {debounced !== '' && (
            <CommandList className="new-session-results">
              <CommandGroup heading={`${results.length} found`}>
                {results.map(item => (
                  <CommandItem key={item.id} value={`item-${item.id}`} onSelect={() => onPick(item)} className="result-row">
                    <span className="result-thumb" aria-hidden>
                      {(item.item_type || item.title).slice(0, 1).toUpperCase()}
                    </span>
                    <span className="result-text">
                      <strong>{item.title}</strong>
                      <small>
                        {item.sku}
                        {item.item_type ? ` · ${item.item_type}` : ''}
                      </small>
                    </span>
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          )}
        </Command>
      </DialogContent>
    </Dialog>
  );
}
