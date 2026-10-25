import {
  commands,
  type AttributeGroupDto,
  type AttributeValueDto_Serialize,
  type DefinitionDto_Serialize,
  type FormDto_Serialize,
  type FormFieldDto_Serialize,
  type FormGroupDto_Serialize,
  type ItemDto_Serialize,
} from './bindings';
import { unwrap } from './sessions';

export type Definition = DefinitionDto_Serialize;
export type AttributeValue = AttributeValueDto_Serialize;
export type Form = FormDto_Serialize;
export type FormGroup = FormGroupDto_Serialize;
export type FormField = FormFieldDto_Serialize;
export type Group = AttributeGroupDto;
/** One field of a step layout (Customize fields); `default` is the value itself, absent when the field has none. */
export type PresetEntry = { key: string; default?: unknown };

/** The built-in Product fields (ch08 §Groups and presets: never removable). */
export const PRODUCT_TITLE = 'product.title';
export const PRODUCT_DESCRIPTION = 'product.description';
export const PRODUCT_TAGS = 'product.tags';
export const FIXED_KEYS: readonly string[] = [PRODUCT_TITLE, PRODUCT_DESCRIPTION, PRODUCT_TAGS];

/** The item's form: Details, Condition, Product with ordered fields, values and filled/total counts. */
export async function getForm(itemId: number): Promise<Form> {
  return unwrap(await commands.formGet(itemId));
}

/** Every active definition (built-in Product fields, then the integration's). */
export async function listDefinitions(): Promise<Definition[]> {
  return unwrap(await commands.attributesDefinitionsList());
}

/**
 * A human edit of one field: `value` is the value in its core form (string, number, array, object), sent as
 * JSON text. The backend stores it canonical, or verbatim with `valid = false` when it does not fit (MC-9).
 */
export async function setAttributeValue(itemId: number, key: string, value: unknown): Promise<AttributeValue> {
  return unwrap(await commands.attributeValuesSet(itemId, key, JSON.stringify(value)));
}

/** Clears the field (an AI suggestion, or any value the photographer empties). */
export async function clearAttribute(itemId: number, key: string): Promise<void> {
  unwrap(await commands.attributeAiClear(itemId, key));
}

/** "Accept all": suggested AI values except low-confidence ones become accepted. Returns how many. */
export async function acceptAllSuggestions(itemId: number): Promise<number> {
  return unwrap(await commands.attributeAiAcceptAll(itemId));
}

/** The stored layout of one step for one item type, else every definition of the step without defaults. */
export async function getPreset(itemType: string, group: Group): Promise<PresetEntry[]> {
  const preset = unwrap(await commands.presetsGet(itemType, group));
  return preset.layout.map(field => (field.default == null ? { key: field.key } : { key: field.key, default: JSON.parse(field.default) }));
}

/** Customize fields: stores the layout of one step for one item type (presets pre-fill new sessions). */
export async function setPreset(itemType: string, group: Group, layout: PresetEntry[]): Promise<void> {
  unwrap(
    await commands.presetsSet({
      item_type: itemType,
      group,
      layout: layout.map(entry => ({ key: entry.key, default: entry.default === undefined ? null : JSON.stringify(entry.default) })),
    }),
  );
}

/** SKU / barcode edit from the form header (05B MC-22); `null` leaves that part unchanged. */
export async function updateIdentity(itemId: number, sku: string | null, barcode: string | null): Promise<ItemDto_Serialize> {
  return unwrap(await commands.itemUpdateIdentity(itemId, sku, barcode));
}
