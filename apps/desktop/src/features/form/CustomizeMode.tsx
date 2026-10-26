import { useEffect, useRef, useState } from 'react';
import { DndContext, KeyboardSensor, PointerSensor, closestCenter, useSensor, useSensors, type DragEndEvent } from '@dnd-kit/core';
import { SortableContext, arrayMove, sortableKeyboardCoordinates, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { GripVertical, Plus, X } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { SkeletonFields } from '@/components/pending/SkeletonFields';
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { FIXED_KEYS, PRODUCT_DESCRIPTION, getPreset, setPreset, type Definition, type Group, type PresetEntry } from '@/services/attributes';
import { kindOf } from './kinds';
import { INPUTS } from './MetafieldInput';
import { TextareaInput } from './inputs/TextareaInput';
import { isEmptyValue } from './useFormState';
import { STEP_LABEL } from './StepTabs';

/** Kinds without a default value: the damage report is per session, references are never written. */
const NO_DEFAULT = new Set(['files', 'unsupported']);

type RowProps = {
  entry: PresetEntry;
  definition: Definition | undefined;
  onDefault: (value: unknown) => void;
  onRemove: (() => void) | null;
};

function Row({ entry, definition, onDefault, onRemove }: RowProps) {
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition } = useSortable({ id: entry.key });
  const label = definition?.label ?? entry.key;
  const kind = definition && definition.writable ? kindOf(definition) : 'unsupported';
  const Input = definition?.key === PRODUCT_DESCRIPTION ? TextareaInput : INPUTS[kind];
  return (
    <li
      ref={setNodeRef}
      data-layout-key={entry.key}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className="grid grid-cols-[24px_1fr_1.3fr_28px] items-start gap-3 border-b bg-background py-3"
    >
      <button ref={setActivatorNodeRef} type="button" aria-label={`Reorder ${label}`} className="mt-0.5 cursor-grab text-muted-foreground" {...attributes} {...listeners}>
        <GripVertical className="size-4" />
      </button>
      <div className="min-w-0">
        <p className="text-sm font-semibold">{label}</p>
        <p className="truncate text-[11px] text-muted-foreground">
          {entry.key} · {definition?.core_type ?? 'unknown'}
        </p>
      </div>
      <div aria-label={`Default value of ${label}`} role="group">
        {definition && !NO_DEFAULT.has(kind) ? (
          <Input id={`default-${entry.key}`} label={`Default ${label}`} kind={kind} definition={definition} value={entry.default} onChange={onDefault} onBlur={() => undefined} placeholder="No default" />
        ) : (
          <span className="text-xs text-muted-foreground">—</span>
        )}
      </div>
      {onRemove ? (
        <Button variant="ghost" size="icon-sm" aria-label={`Remove ${label}`} onClick={onRemove}>
          <X />
        </Button>
      ) : (
        <span />
      )}
    </li>
  );
}

type CustomizeModeProps = {
  itemType: string;
  group: Group;
  definitions: Definition[];
  onDone: () => void;
};

/**
 * Customize fields (ch08 §Groups and presets): the layout of one step for one item type. Reorder by drag, remove
 * (never Title, Description, Tags), add from every definition, a default value per field. The layout is stored
 * with `presets_set` on Done, or when the mode is left another way (step change, form closed).
 */
export function CustomizeMode({ itemType, group, definitions, onDone }: CustomizeModeProps) {
  const [layout, setLayout] = useState<PresetEntry[] | null>(null);
  const dirty = useRef(false);
  const latest = useRef<PresetEntry[] | null>(null);
  latest.current = layout;
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }), useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }));

  useEffect(() => {
    let live = true;
    getPreset(itemType, group)
      .then(preset => live && setLayout(preset))
      .catch(error => toast.error(`Could not load the layout: ${(error as Error).message}`));
    return () => {
      live = false;
    };
  }, [itemType, group]);

  const save = async () => {
    if (!dirty.current || !latest.current) return;
    dirty.current = false;
    try {
      await setPreset(itemType, group, latest.current);
    } catch (error) {
      dirty.current = true;
      toast.error(`Could not save the layout: ${(error as Error).message}`);
      throw error;
    }
  };
  const saveRef = useRef(save);
  saveRef.current = save;
  // Leaving the mode without Done still keeps the layout.
  useEffect(() => () => void saveRef.current().catch(() => undefined), []);

  function change(next: PresetEntry[]) {
    dirty.current = true;
    setLayout(next);
  }

  if (!layout) return <div className="p-6"><SkeletonFields n={4} /></div>;

  const byKey = new Map(definitions.map(definition => [definition.key, definition]));
  const addable = definitions.filter(definition => !layout.some(entry => entry.key === definition.key));

  function onDragEnd(event: DragEndEvent) {
    if (!event.over || event.active.id === event.over.id || !layout) return;
    const from = layout.findIndex(entry => entry.key === event.active.id);
    const to = layout.findIndex(entry => entry.key === event.over!.id);
    if (from !== -1 && to !== -1) change(arrayMove(layout, from, to));
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex-1 overflow-y-auto px-6 py-5">
        <h3 className="text-base font-semibold">
          {STEP_LABEL[group]} fields for {itemType || 'this item type'}
        </h3>
        <p className="text-xs text-muted-foreground">Drag to reorder. Default values pre-fill every new {itemType || 'item of this type'}.</p>
        <Command className="mt-4 rounded-lg border">
          <CommandInput placeholder="Add a field" aria-label="Add a field" />
          <CommandList className="max-h-48">
            <CommandEmpty>No other fields.</CommandEmpty>
            {addable.map(definition => (
              <CommandItem key={definition.key} value={`${definition.label} ${definition.key}`} onSelect={() => change([...layout, { key: definition.key }])}>
                <Plus className="size-3.5" /> {definition.label} <span className="text-[11px] text-muted-foreground">{definition.key}</span>
              </CommandItem>
            ))}
          </CommandList>
        </Command>
        <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
          <SortableContext items={layout.map(entry => entry.key)} strategy={verticalListSortingStrategy}>
            <ul aria-label="Fields" className="mt-4">
              {layout.map((entry, index) => (
                <Row
                  key={entry.key}
                  entry={entry}
                  definition={byKey.get(entry.key)}
                  onDefault={value => change(layout.map((e, i) => (i === index ? (isEmptyValue(value) ? { key: e.key } : { key: e.key, default: value }) : e)))}
                  onRemove={FIXED_KEYS.includes(entry.key) ? null : () => change(layout.filter((_, i) => i !== index))}
                />
              ))}
            </ul>
          </SortableContext>
        </DndContext>
      </div>
      <footer className="flex items-center justify-between border-t px-5 py-3">
        <span className="text-xs text-muted-foreground">Saved for every {itemType || 'item of this type'}</span>
        <ActionButton action={async () => { await save(); onDone(); }}>Done</ActionButton>
      </footer>
    </div>
  );
}
