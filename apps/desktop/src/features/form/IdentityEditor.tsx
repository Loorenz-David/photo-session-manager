import { useState, type FormEvent } from 'react';
import { Barcode, Pencil } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { updateIdentity } from '@/services/attributes';
import type { Item } from '@/services/items';

type IdentityEditorProps = { item: Item; onSaved: (item: Item) => void };

/** SKU | barcode in the form header; the pencil opens an editor (`item_update_identity`, 05B MC-22). */
export function IdentityEditor({ item, onSaved }: IdentityEditorProps) {
  const [open, setOpen] = useState(false);
  const [sku, setSku] = useState(item.sku);
  const [barcode, setBarcode] = useState(item.barcode ?? '');
  const [error, setError] = useState<string | null>(null);

  function onOpenChange(next: boolean) {
    setOpen(next);
    if (next) {
      setSku(item.sku);
      setBarcode(item.barcode ?? '');
      setError(null);
    }
  }

  async function save(event: FormEvent) {
    event.preventDefault();
    const nextSku = sku.trim();
    const nextBarcode = barcode.trim();
    try {
      const saved = await updateIdentity(item.id, nextSku && nextSku !== item.sku ? nextSku : null, nextBarcode && nextBarcode !== (item.barcode ?? '') ? nextBarcode : null);
      setOpen(false);
      onSaved(saved);
    } catch (failure) {
      const code = (failure as { code?: string; data?: Record<string, string> }).code;
      setError(code === 'sku_collision' ? `SKU already used by ${(failure as { data?: Record<string, string> }).data?.existing_sku ?? 'another item'}` : (failure as Error).message);
    }
  }

  return (
    <Popover open={open} onOpenChange={onOpenChange}>
      <PopoverTrigger asChild>
        <button type="button" aria-label="Edit SKU and barcode" className="inline-flex items-center gap-2 rounded-md px-1.5 py-0.5 text-xs hover:bg-muted">
          <span className="font-bold">{item.sku}</span>
          {item.barcode && (
            <>
              <span className="text-muted-foreground">|</span>
              <span className="inline-flex items-center gap-1 text-muted-foreground">
                <Barcode className="size-3.5" />
                {item.barcode}
              </span>
            </>
          )}
          <Pencil className="size-3 text-muted-foreground" />
        </button>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-72">
        <form className="flex flex-col gap-2" onSubmit={event => void save(event)}>
          <label className="text-xs font-semibold" htmlFor="identity-sku">
            SKU
          </label>
          <Input id="identity-sku" value={sku} onChange={event => setSku(event.target.value)} />
          <label className="text-xs font-semibold" htmlFor="identity-barcode">
            Barcode
          </label>
          <Input id="identity-barcode" value={barcode} onChange={event => setBarcode(event.target.value)} />
          {error && (
            <p role="alert" className="text-xs text-destructive">
              {error}
            </p>
          )}
          <Button type="submit" size="sm" className="self-end">
            Save
          </Button>
        </form>
      </PopoverContent>
    </Popover>
  );
}
