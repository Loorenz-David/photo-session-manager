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

export async function createItem(args: { sku: string; title: string; itemType: string }): Promise<Item> {
  return unwrap(await commands.itemCreate(args.sku, args.title, args.itemType, null));
}

export async function listItemTypes(): Promise<string[]> {
  return unwrap(await commands.itemTypesList());
}

export async function setItemType(itemId: number, itemType: string): Promise<Item> {
  return unwrap(await commands.itemTypeSet(itemId, itemType));
}

export async function refreshItem(itemId: number): Promise<Item> {
  return unwrap(await commands.itemRefresh(itemId));
}

export async function adoptItem(externalId: string): Promise<Item> {
  return unwrap(await commands.itemAdopt(externalId));
}
