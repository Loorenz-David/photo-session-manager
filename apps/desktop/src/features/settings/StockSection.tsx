import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { StockPicker, parseQuantity } from '@/features/form/StockFields';
import { stockDefaultGet, stockDefaultSet, stockLocations, type StockLocationsDto } from '@/services/stock';

/** Shown when the shop's locations cannot be read (the shop is unreachable). */
export const LOCATIONS_UNREADABLE = "Couldn't read the shop's locations";

/**
 * Settings → Shopify (changelog 45, P65): the location and quantity a new product starts with. The last create's values
 * become this default; it can be changed here.
 */
export function StockSection() {
  const [locations, setLocations] = useState<StockLocationsDto | null>(null);
  const [failed, setFailed] = useState(false);
  const [value, setValue] = useState<{ locationId: string; quantity: string }>({ locationId: '', quantity: '1' });
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let alive = true;
    Promise.all([stockLocations(), stockDefaultGet()]).then(([found, stock]) => {
      if (!alive) return;
      setLocations(found);
      if (stock) setValue({ locationId: stock.location_id, quantity: String(stock.quantity) });
    }).catch(() => { if (alive) setFailed(true); });
    return () => { alive = false; };
  }, []);
  const save = async () => {
    const location = locations?.locations.find(l => l.id === value.locationId);
    const quantity = parseQuantity(value.quantity);
    if (!location || quantity === null) { toast.error('Choose a location and a quantity of 0 or more'); return; }
    setBusy(true);
    try {
      await stockDefaultSet({ location_id: location.id, location_name: location.name, quantity });
      toast.success(`New products start with ${quantity} at ${location.name}`);
    } catch (error) {
      toast.error(`Not saved: ${(error as Error).message ?? String(error)}`);
    } finally { setBusy(false); }
  };
  return <section className="health-section flex max-w-xl flex-col gap-4" aria-label="Stock for new products">
    <h2>Stock for new products</h2>
    {failed && <p role="status">{LOCATIONS_UNREADABLE}</p>}
    {locations?.missing_access && <p role="status">{locations.missing_access}</p>}
    {locations && !locations.missing_access && <>
      <StockPicker idPrefix="settings-stock" locations={locations.locations} value={value} onChange={setValue} />
      <div><Button type="button" variant="outline" disabled={busy} onClick={() => void save()}>Save stock default</Button></div>
    </>}
  </section>;
}
