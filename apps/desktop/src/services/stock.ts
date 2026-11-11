import { commands, type ItemStockDto, type StockChoiceDto, type StockLocationsDto } from './bindings';
import { unwrap } from './sessions';

export type { ItemStockDto, StockChoiceDto, StockLocationsDto };

/** Changelog 45 (P65): the shop's active locations (read once per app run), or why they cannot be read. */
export async function stockLocations(): Promise<StockLocationsDto> {
  return unwrap(await commands.stockLocations());
}

/** Settings → Shopify: the remembered location and quantity for new products (null when none). */
export async function stockDefaultGet(): Promise<StockChoiceDto | null> {
  return unwrap(await commands.stockDefaultGet());
}

export async function stockDefaultSet(choice: StockChoiceDto): Promise<StockChoiceDto | null> {
  return unwrap(await commands.stockDefaultSet(choice));
}

/** Whether the item will be created by a publish, and its stock: its own choice, else the default (null: none). */
export async function itemStockGet(itemId: number): Promise<ItemStockDto> {
  return unwrap(await commands.itemStockGet(itemId));
}

export async function itemStockSet(itemId: number, choice: StockChoiceDto | null): Promise<void> {
  unwrap(await commands.itemStockSet(itemId, choice));
}
