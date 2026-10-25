import type { Definition } from '@/services/attributes';

/** How a definition renders (evidence design_handoff §Field kind map; COMPONENTS.md §MetafieldInput kinds). */
export type Kind =
  | 'text'
  | 'textarea'
  | 'choice'
  | 'select'
  | 'radio'
  | 'severity'
  | 'multi'
  | 'multiselect'
  | 'tags'
  | 'int'
  | 'decimal'
  | 'money'
  | 'dimension'
  | 'weight'
  | 'boolean'
  | 'date'
  | 'color'
  | 'rating'
  | 'url'
  | 'files'
  | 'unsupported';

/** MC-9 severity set: exactly these five strings, case-sensitive, any order, no others. */
export const SEVERITY_SET: readonly string[] = ['N/A', 'None', 'Minor', 'Moderate', 'Major'];

export function isSeveritySet(choices: readonly string[]): boolean {
  return choices.length === SEVERITY_SET.length && new Set(choices).size === SEVERITY_SET.length && choices.every(c => SEVERITY_SET.includes(c));
}

/** Options too many or too long for pills: more than 4, or more than 48 characters joined. */
function crowded(choices: readonly string[]): boolean {
  return choices.length > 4 || choices.join('').length > 48;
}

const DIRECT: Record<string, Kind> = {
  text: 'text',
  textarea: 'textarea',
  rich_text: 'textarea',
  integer: 'int',
  decimal: 'decimal',
  money: 'money',
  dimension: 'dimension',
  weight: 'weight',
  boolean: 'boolean',
  date: 'date',
  color: 'color',
  rating: 'rating',
  url: 'url',
  tags: 'tags',
  media_list: 'files',
};

/** The input kind of a core-typed definition. Unknown and `reference` types are not editable here. */
export function kindOf(def: Pick<Definition, 'core_type' | 'choices'>): Kind {
  const choices = def.choices;
  if (def.core_type === 'choice') {
    if (choices.length === 0) return 'text';
    if (isSeveritySet(choices)) return 'severity';
    if (choices.some(choice => choice.length > 32)) return 'radio';
    return crowded(choices) ? 'select' : 'choice';
  }
  if (def.core_type === 'multi_choice') {
    if (choices.length === 0) return 'tags';
    return crowded(choices) ? 'multiselect' : 'multi';
  }
  return DIRECT[def.core_type] ?? 'unsupported';
}
