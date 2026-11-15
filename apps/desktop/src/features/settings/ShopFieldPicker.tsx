import { useState } from 'react';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import type { Definition } from '@/services/attributes';

/** Shown under a chosen key that is not one of the shop's field keys. */
export const NOT_A_SHOP_FIELD = 'Not one of your shop’s fields';
/** Shown instead of the list when no shop is linked (08A §2: research settings never need a shop to load). */
export const CONNECT_A_SHOP = 'Connect a shop to choose fields';
/** Shown under a field the screen preselected for the person (Designer and maker, R11·U6). */
export const SUGGESTED_NOT_SAVED = 'suggested — not saved yet';
export const RESEARCH_NEVER_FILLS = 'Research never fills this kind of field';

/** The linked shop's product fields: every active definition that the integration brings (not the built-in Product fields). */
export function shopFieldsOf(definitions: Definition[]): Definition[] {
  return definitions.filter(definition => definition.integration !== 'core');
}

/**
 * A filterable, scrolling list of the linked shop's product fields, each by label and key with a check box (R11·U6).
 * A chosen key the shop does not have is still listed, marked, so it can be unchecked.
 */
export function ShopFieldPicker({ label, definitions, chosen, shopDomain, suggested = [], eligible, onChange }: {
  label: string;
  definitions: Definition[];
  chosen: string[];
  shopDomain: string | null;
  suggested?: string[];
  eligible?: (definition: Definition) => boolean;
  onChange: (keys: string[]) => void;
}) {
  const [filter, setFilter] = useState('');
  const allShop = shopDomain ? shopFieldsOf(definitions) : [];
  const shop = allShop.filter(d => !eligible || eligible(d) || chosen.includes(d.key));
  const known = new Set(allShop.map(d => d.key));
  const unknown = chosen.filter(key => !known.has(key)).map(key => ({ key, label: key }));
  const needle = filter.trim().toLowerCase();
  const rows = [...shop.map(d => ({ key: d.key, label: d.label, unsupported: !!eligible && !eligible(d) })), ...unknown.map(d => ({ ...d, unsupported: false }))]
    .filter(row => needle === '' || `${row.label} ${row.key}`.toLowerCase().includes(needle));
  const toggle = (key: string, on: boolean) => onChange(on ? [...chosen.filter(k => k !== key), key] : chosen.filter(k => k !== key));

  return (
    <div className="flex flex-col gap-2">
      {allShop.length === 0 && <p className="health-detail">{CONNECT_A_SHOP}</p>}
      {allShop.length > 0 && <Input aria-label={`Filter ${label}`} placeholder="Filter fields" value={filter} onChange={event => setFilter(event.target.value)} />}
      {rows.length > 0 && (
        <ul aria-label={label} className="max-h-56 overflow-y-auto rounded-md border p-2">
          {rows.map(row => (
            <li key={row.key} data-field={row.key} className="py-1">
              <label className="flex items-start gap-2">
                <Checkbox aria-label={`${row.label} (${row.key})`} checked={chosen.includes(row.key)} onCheckedChange={value => toggle(row.key, value === true)} />
                <span>
                  <strong>{row.label}</strong> <small className="health-detail">{row.key}</small>
                  {!known.has(row.key) && <span className="block text-xs text-destructive" data-not-a-shop-field>{NOT_A_SHOP_FIELD}</span>}
                  {row.unsupported && <span className="block text-xs text-destructive">{RESEARCH_NEVER_FILLS}</span>}
                  {suggested.includes(row.key) && chosen.includes(row.key) && <span className="block text-xs" data-suggested>{SUGGESTED_NOT_SAVED}</span>}
                </span>
              </label>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
