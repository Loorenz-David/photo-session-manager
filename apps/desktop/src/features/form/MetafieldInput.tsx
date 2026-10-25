import type { ComponentType } from 'react';
import { PRODUCT_DESCRIPTION, type AttributeValue, type Definition, type FormField } from '@/services/attributes';
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

/** The verbatim text of an invalid stored value: a JSON string without its quotes, anything else as stored. */
function verbatim(value: AttributeValue): string {
  try {
    const parsed = JSON.parse(value.value);
    return typeof parsed === 'string' ? parsed : value.value;
  } catch {
    return value.value;
  }
}

type MetafieldInputProps = {
  field: FormField;
  value: unknown;
  /** The photographer edited the field in this form (the edit is human-owned; markers of the stored row go). */
  edited: boolean;
  onChange: (value: unknown) => void;
  onBlur: () => void;
  placeholder?: string;
  conditionPhotos?: Photo[];
  onLooksRight: (field: FormField) => void;
  onClearAi: (field: FormField) => void;
  onSeeResearch: (key: string) => void;
};

/**
 * One generated field (ch08 §Field kinds): label, the AI pill for a value with provenance `ai`, the Default
 * marker, the invalid marker with the verbatim value (MC-9), and the input of the definition's kind.
 */
export function MetafieldInput({ field, value, edited, onChange, onBlur, placeholder, conditionPhotos, onLooksRight, onClearAi, onSeeResearch }: MetafieldInputProps) {
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
      </div>
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
