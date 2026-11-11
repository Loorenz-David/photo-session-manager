import type { ComponentType } from 'react';
import { BookmarkPlus, Pin, X } from 'lucide-react';
import { PRODUCT_DESCRIPTION, PRODUCT_TITLE, type AttributeValue, type Definition, type FormField } from '@/services/attributes';
import type { Photo } from '@/services/photos';
import { AiPopover } from './AiPopover';
import { BooleanInput } from './inputs/BooleanInput';
import { ChoicePills } from './inputs/ChoicePills';
import { ColorInput } from './inputs/ColorInput';
import { DateInput } from './inputs/DateInput';
import { DecimalInput } from './inputs/DecimalInput';
import { DescriptionInput } from './inputs/DescriptionInput';
import { DimensionInput } from './inputs/DimensionInput';
import { FilesPicker } from './inputs/FilesPicker';
import { IntInput } from './inputs/IntInput';
import { MultiPills } from './inputs/MultiPills';
import { MultiSelect } from './inputs/MultiSelect';
import { RadioCards } from './inputs/RadioCards';
import { RatingInput } from './inputs/RatingInput';
import { SelectInput } from './inputs/SelectInput';
import { SeverityScale } from './inputs/SeverityScale';
import { TagsInput } from './inputs/TagsInput';
import { TextareaInput } from './inputs/TextareaInput';
import { TextInput } from './inputs/TextInput';
import { Unsupported } from './inputs/Unsupported';
import { UrlInput } from './inputs/UrlInput';
import { kindOf, type Kind } from './kinds';

/** What every input component receives: the value in its core form and a change callback (MC-9 forms). */
export type FieldInputProps = {
  id: string;
  label: string;
  kind: Kind;
  definition: Definition;
  value: unknown;
  onChange: (value: unknown) => void;
  onBlur: () => void;
  placeholder?: string;
  /** `files`: the session's condition photos in condition order, and the ids `form_get` reports selected. */
  conditionPhotos?: Photo[];
  selected?: number[] | null;
};

/** `validations` of a definition (JSON text) as an object; `{}` when absent or unreadable. */
export function validationsOf(definition: Definition): Record<string, unknown> {
  try {
    const parsed = JSON.parse(definition.validations || '{}');
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

/** The input component of each kind (COMPONENTS.md §MetafieldInput kinds → shadcn). */
export const INPUTS: Record<Kind, ComponentType<FieldInputProps>> = {
  text: TextInput,
  textarea: TextareaInput,
  choice: ChoicePills,
  select: SelectInput,
  radio: RadioCards,
  severity: SeverityScale,
  multi: MultiPills,
  multiselect: MultiSelect,
  tags: TagsInput,
  int: IntInput,
  decimal: DecimalInput,
  money: DecimalInput,
  dimension: DimensionInput,
  weight: DimensionInput,
  boolean: BooleanInput,
  date: DateInput,
  color: ColorInput,
  rating: RatingInput,
  url: UrlInput,
  files: FilesPicker,
  unsupported: Unsupported,
};

function isEmpty(value: unknown): boolean {
  return value === null || value === undefined || (typeof value === 'string' && value.trim() === '') || (Array.isArray(value) && value.length === 0);
}

/** The verbatim text of an invalid stored value: a JSON string without its quotes, anything else as stored. */
function verbatim(value: AttributeValue): string {
  try {
    const parsed = JSON.parse(value.value);
    return typeof parsed === 'string' ? parsed : value.value;
  } catch {
    return value.value;
  }
}

/** P67: a saved value or default as one short line (pick buttons, the Default hint). */
export function valueText(value: unknown): string {
  if (value === null || value === undefined) return '';
  if (typeof value === 'string') return value.replace(/\s+/g, ' ').trim();
  if (Array.isArray(value)) return value.map(valueText).join(', ');
  if (typeof value === 'object') {
    const record = value as Record<string, unknown>;
    if ('value' in record && 'unit' in record) return `${valueText(record.value)} ${valueText(record.unit)}`;
    if ('amount' in record && 'currency' in record) return `${valueText(record.amount)} ${valueText(record.currency)}`;
    return Object.values(record).map(valueText).join(' ');
  }
  return String(value);
}

/** P67: kinds without defaults or saved values (the damage report is per session, references are never written) and the title (per item). */
export function takesPresets(field: FormField): boolean {
  const kind = field.definition.writable ? kindOf(field.definition) : 'unsupported';
  return field.definition.key !== PRODUCT_TITLE && kind !== 'files' && kind !== 'unsupported';
}

/** P67 (changelog 46 (4)): what a field can do with its defaults and saved values. */
export type FieldPresets = {
  /** The field's default for the item's type (parsed), else null. */
  defaultValue: unknown;
  /** Saved values (parsed), most recent first. */
  saved: unknown[];
  onSaveDefault: (value: unknown) => void;
  onSaveValue: (value: unknown) => void;
  onPick: (value: unknown) => void;
  onRemoveSaved: (value: unknown) => void;
};

type MetafieldInputProps = {
  field: FormField;
  value: unknown;
  /** The photographer edited the field in this form (the edit is human-owned; markers of the stored row go). */
  edited: boolean;
  onChange: (value: unknown) => void;
  onBlur: () => void;
  placeholder?: string;
  conditionPhotos?: Photo[];
  onLooksRight: (field: FormField) => Promise<unknown>;
  onClearAi: (field: FormField) => Promise<unknown>;
  onSeeResearch: (key: string) => void;
  presets?: FieldPresets;
};

/**
 * One generated field (ch08 §Field kinds): label, the AI pill for a value with provenance `ai`, the Default
 * marker, the invalid marker with the verbatim value (MC-9), and the input of the definition's kind.
 */
export function MetafieldInput({ field, value, edited, onChange, onBlur, placeholder, conditionPhotos, onLooksRight, onClearAi, onSeeResearch, presets }: MetafieldInputProps) {
  const { definition } = field;
  const stored = field.value;
  const kind = definition.writable ? kindOf(definition) : 'unsupported';
  const Input = definition.key === PRODUCT_DESCRIPTION ? DescriptionInput : INPUTS[kind];
  const id = `field-${definition.key}`;
  const showAi = !edited && stored?.provenance === 'ai';
  const invalid = !edited && stored !== null && !stored.valid;

  return (
    <div className="flex flex-col gap-2" data-field={definition.key} data-kind={kind}>
      <div className="flex items-center gap-2">
        <label htmlFor={id} className="text-sm font-semibold">
          {definition.label}
        </label>
        {showAi && stored && (
          <AiPopover value={stored} label={definition.label} onLooksRight={() => onLooksRight(field)} onClear={() => onClearAi(field)} onSeeResearch={() => onSeeResearch(definition.key)} />
        )}
        {!edited && field.is_default && <span className="rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground">Default</span>}
        {!edited && stored?.provenance === 'integration' && <span className="rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground" data-from-integration>from Shopify</span>}
        {presets && takesPresets(field) && (
          <span className="ml-auto flex items-center gap-1">
            <button type="button" className="flex items-center gap-1 rounded px-1.5 py-0.5 text-[11px] text-muted-foreground hover:bg-muted disabled:opacity-40" aria-label={`Save ${definition.label} as default`} disabled={isEmpty(value)} onClick={() => presets.onSaveDefault(value)}>
              <Pin className="size-3" /> Save as default
            </button>
            <button type="button" className="flex items-center gap-1 rounded px-1.5 py-0.5 text-[11px] text-muted-foreground hover:bg-muted disabled:opacity-40" aria-label={`Save ${definition.label} value`} disabled={isEmpty(value)} onClick={() => presets.onSaveValue(value)}>
              <BookmarkPlus className="size-3" /> Save value
            </button>
          </span>
        )}
      </div>
      {presets && takesPresets(field) && presets.saved.length > 0 && (
        <ul className="flex flex-wrap gap-1.5" aria-label={`Saved values of ${definition.label}`}>
          {presets.saved.map(saved => (
            <li key={JSON.stringify(saved)} className="flex items-center rounded-full border text-xs">
              <button type="button" className="px-2 py-0.5" aria-label={`Use ${valueText(saved)}`} onClick={() => presets.onPick(saved)}>
                {valueText(saved)}
              </button>
              <button type="button" className="pr-1.5 text-muted-foreground" aria-label={`Remove saved ${valueText(saved)}`} onClick={() => presets.onRemoveSaved(saved)}>
                <X className="size-3" />
              </button>
            </li>
          ))}
        </ul>
      )}
      {invalid && stored && (
        <p className="rounded-md bg-destructive/10 px-2.5 py-1.5 text-xs text-destructive" data-invalid>
          <span className="font-semibold">Invalid</span> · stored as received: <span data-verbatim>{verbatim(stored)}</span>
        </p>
      )}
      <Input
        id={id}
        label={definition.label}
        kind={kind}
        definition={definition}
        value={value}
        onChange={onChange}
        onBlur={onBlur}
        placeholder={placeholder}
        conditionPhotos={conditionPhotos}
        selected={field.selected}
      />
    </div>
  );
}
