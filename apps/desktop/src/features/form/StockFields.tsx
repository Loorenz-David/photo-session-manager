import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Input } from '@/components/ui/input';
import { itemStockGet, itemStockSet, stockLocations, type StockChoiceDto, type StockLocationsDto } from '@/services/stock';

/** Shown when no location is chosen: the product is created without stock tracking. */
export const CHOOSE_LOCATION = 'Choose a location to track stock';

/** The location drop-down and quantity of a stock choice (item form and Settings → Shopify). */
export function StockPicker({ idPrefix, locations, value, onChange }: {
  idPrefix: string;
  locations: StockLocationsDto['locations'];
  value: { locationId: string; quantity: string };
  onChange: (value: { locationId: string; quantity: string }) => void;
}) {
  return <>
    <div className="flex flex-col gap-1">
      <label htmlFor={`${idPrefix}-location`}>Location</label>
      <select id={`${idPrefix}-location`} value={value.locationId} onChange={e => onChange({ ...value, locationId: e.target.value })}>
        <option value="">No location</option>
        {locations.map(location => <option key={location.id} value={location.id}>{location.name}</option>)}
      </select>
    </div>
    <div className="flex flex-col gap-1">
      <label htmlFor={`${idPrefix}-quantity`}>Quantity</label>
      <Input id={`${idPrefix}-quantity`} type="number" min={0} step={1} value={value.quantity} onChange={e => onChange({ ...value, quantity: e.target.value })} />
    </div>
  </>;
}

/** A whole number of 0 or more, else null. */
export function parseQuantity(text: string): number | null {
  const n = Number(text);
  return text.trim() !== '' && Number.isInteger(n) && n >= 0 ? n : null;
}

/**
 * Changelog 45 (P65): Location and Quantity of a product the app will create (Product step, new items only), prefilled from
 * the remembered default. Each change is saved for the item; the create uses it and remembers it as the next default.
 */
export function StockFields({ itemId }: { itemId: number }) {
  const [locations, setLocations] = useState<StockLocationsDto | null>(null);
  const [unavailable, setUnavailable] = useState(false);
  const [value, setValue] = useState<{ locationId: string; quantity: string } | null>(null);
  useEffect(() => {
    let alive = true;
    Promise.all([stockLocations(), itemStockGet(itemId)]).then(([found, stock]) => {
      if (!alive) return;
      if (!stock.new_item) { setUnavailable(true); return; }
      setLocations(found);
      setValue({ locationId: stock.choice?.location_id ?? '', quantity: String(stock.choice?.quantity ?? 1) });
    }).catch(() => { if (alive) setUnavailable(true); });
    return () => { alive = false; };
  }, [itemId]);
  if (unavailable || !locations || !value) return null;

  async function change(next: { locationId: string; quantity: string }) {
    setValue(next);
    const quantity = parseQuantity(next.quantity);
    const location = locations?.locations.find(l => l.id === next.locationId);
    if (next.locationId !== '' && quantity === null) return;
    const choice: StockChoiceDto | null = location && quantity !== null
      ? { location_id: location.id, location_name: location.name, quantity }
      : null;
    try { await itemStockSet(itemId, choice); } catch (error) { toast.error(`Could not save the stock: ${(error as Error).message}`); }
  }

  return <section aria-label="Stock" className="flex flex-col gap-3">
    <h3 className="text-sm font-semibold">Stock</h3>
    {locations.missing_access
      ? <p role="status" className="text-sm text-muted-foreground">{locations.missing_access}</p>
      : <>
        <StockPicker idPrefix="form-stock" locations={locations.locations} value={value} onChange={next => void change(next)} />
        {value.locationId === '' && <p role="status" className="text-sm text-muted-foreground">{CHOOSE_LOCATION}</p>}
      </>}
  </section>;
}
