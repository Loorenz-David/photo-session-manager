import { useEffect, useId, useMemo, useRef, useState } from 'react';
import { Check, ChevronDown } from 'lucide-react';
import { Popover, PopoverAnchor, PopoverContent } from '@/components/ui/popover';
import { listItemTypes } from '@/services/items';

export function useItemTypes(active: boolean) {
  const [types, setTypes] = useState<string[] | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    if (!active) return;
    let alive = true;
    setTypes(null);
    setFailed(false);
    listItemTypes().then(value => {
      if (alive) setTypes(value);
    }).catch(error => {
      if (alive && (error as { code?: string }).code !== 'no_integration') setFailed(true);
    });
    return () => { alive = false; };
  }, [active]);
  return { types, failed };
}

/** The types shown for `typed`: all of them while the field holds a listed type or nothing, else those containing it. */
export function matchingTypes(types: string[], typed: string | null | undefined): string[] {
  const needle = (typed ?? "").trim().toLowerCase();
  if (!needle || types.some(type => type.toLowerCase() === needle)) return types;
  return types.filter(type => type.toLowerCase().includes(needle));
}

/**
 * The item type field (owner 2026-11-16): typing searches, the chevron opens the app's own list (the browser's
 * suggestion list was unreadable and gave no sign it could be opened). Arrow keys move in the list, Return picks.
 */
export function ItemTypePicker({ value, onChange, onBlur, onEnter, onChoose, fromIntegration, types, failed, id, hints }: {
  value: string; onChange: (value: string) => void; onBlur?: () => void; types: string[] | null; failed: boolean; id: string;
  /** P67: Return confirms the typed or suggested type. */
  onEnter?: () => void;
  /** A type picked from the list. */
  onChoose?: (value: string) => void;
  /** P67: the type is the one read from the linked product. */
  fromIntegration?: boolean;
  /** A short note shown next to a type in the list (e.g. its template). */
  hints?: Record<string, string>;
}) {
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const input = useRef<HTMLInputElement>(null);
  const anchor = useRef<HTMLDivElement>(null);
  const listId = `${useId()}-list`;
  const shown = useMemo(() => matchingTypes(types ?? [], value), [types, value]);
  const canOpen = shown.length > 0;
  useEffect(() => { if (active >= shown.length) setActive(shown.length - 1); }, [active, shown.length]);

  const choose = (type: string) => {
    onChange(type);
    onChoose?.(type);
    setOpen(false);
    setActive(-1);
  };
  const openList = () => {
    if (!canOpen) return;
    setOpen(true);
    setActive(Math.max(0, shown.indexOf(value)));
  };

  return <div className="flex flex-col gap-1">
    <label htmlFor={id}>Item type{fromIntegration && <span className="ml-1.5 rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground" data-from-integration>from Shopify</span>}</label>
    <Popover open={open && canOpen} onOpenChange={setOpen}>
      <PopoverAnchor asChild>
        <div ref={anchor} className="type-picker">
          <input ref={input} id={id} role="combobox" aria-label="Item type" aria-expanded={open && canOpen} aria-controls={listId}
            aria-autocomplete="list" aria-activedescendant={open && active >= 0 ? `${listId}-${active}` : undefined}
            value={value ?? ""} autoComplete="off" spellCheck={false}
            placeholder={types?.length ? 'Choose a shop type' : 'Type an item type'}
            onChange={event => { onChange(event.target.value); setOpen(true); setActive(0); }}
            onClick={() => { if (!open) openList(); }}
            onBlur={event => {
              // Focus moving into the list is not leaving the field.
              if (anchor.current?.contains(event.relatedTarget as Node | null)) return;
              setOpen(false);
              onBlur?.();
            }}
            onKeyDown={event => {
              if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
                event.preventDefault();
                if (!open) { openList(); return; }
                const step = event.key === 'ArrowDown' ? 1 : -1;
                setActive(current => (current + step + shown.length) % Math.max(shown.length, 1));
              } else if (event.key === 'Enter') {
                if (open && canOpen && active >= 0 && shown[active]) { event.preventDefault(); choose(shown[active]); return; }
                if (onEnter) { event.preventDefault(); onEnter(); }
              } else if (event.key === 'Escape' && open) {
                event.preventDefault();
                event.stopPropagation();
                setOpen(false);
              }
            }} />
          {types && types.length > 0 && <button type="button" tabIndex={-1} className="type-picker-toggle" aria-label="Show item types"
            onMouseDown={event => event.preventDefault()}
            onClick={() => { if (open) setOpen(false); else { openList(); input.current?.focus(); } }}>
            <ChevronDown aria-hidden />
          </button>}
        </div>
      </PopoverAnchor>
      <PopoverContent align="start" className="type-picker-list" onOpenAutoFocus={event => event.preventDefault()}
        onCloseAutoFocus={event => event.preventDefault()}
        onInteractOutside={event => { if (anchor.current?.contains(event.target as Node)) event.preventDefault(); }}>
        <div id={listId} role="listbox" aria-label="Item types">
          {shown.map((type, index) => <div key={type} id={`${listId}-${index}`} role="option" aria-selected={type === value}
            data-active={index === active || undefined} className="type-picker-option"
            onMouseDown={event => event.preventDefault()} onMouseEnter={() => setActive(index)} onClick={() => choose(type)}>
            <span className="type-picker-check">{type === value && <Check aria-hidden />}</span>
            <span className="type-picker-name">{type}</span>
            {hints?.[type] && <span className="type-picker-hint">{hints[type]}</span>}
          </div>)}
        </div>
      </PopoverContent>
    </Popover>
    {failed && <small>Couldn't load the shop's types — type one</small>}
  </div>;
}
