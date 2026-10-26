import { commands, type ItemDto_Serialize } from './bindings';
import { unwrap } from './sessions';

export type Item = ItemDto_Serialize;

/** Local items whose SKU or title contains the query. */
export async function searchItems(query: string): Promise<Item[]> {
  return unwrap(await commands.itemsSearch(query));
}

export async function searchItemsShop(query: string): Promise<Item[]> {
  return unwrap(await commands.itemsSearchShop(query));
}

/** MC-22 proposal `<sku_prefix>-<NNNN>`, or null when no prefix is set. */
export async function nextSku(): Promise<string | null> {
  return unwrap(await commands.itemsNextSku());
}

/** Creates a local item (its type is filled in later, in the form). */
export async function createItem(args: { sku: string; title: string }): Promise<Item> {
  return unwrap(await commands.itemCreate(args.sku, args.title, '', null));
}

export async function adoptItem(externalId: string): Promise<Item> {
  return unwrap(await commands.itemAdopt(externalId));
}
