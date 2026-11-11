import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { ArrowLeft, ArrowRight, ClipboardList, SlidersHorizontal, Sparkles, Wand2, X } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { SkeletonFields } from '@/components/pending/SkeletonFields';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { Progress } from '@/components/ui/progress';
import { cn } from '@/lib/utils';
import {
  PRODUCT_TITLE,
  acceptAllSuggestions,
  addSavedValue,
  clearAttribute,
  fillDefaults,
  listDefinitions,
  readShopValues,
  removeSavedValue,
  setAttributeValue,
  setFieldDefault,
  type Definition,
  type FormField,
  type FormRead,
  type Group,
} from '@/services/attributes';
import { setItemType, type Item } from '@/services/items';
import type { Photo } from '@/services/photos';
import { displayOrder } from '@/features/workspace/order';
import { CustomizeMode } from './CustomizeMode';
import { IdentityEditor } from './IdentityEditor';
import { ItemTypePicker, useItemTypes } from './ItemTypePicker';
import { MediaColumn } from './MediaColumn';
import { MetafieldInput, type FieldPresets } from './MetafieldInput';
import { ResearchPanel } from './ResearchPanel';
import { STEP_LABEL, StepTabs } from './StepTabs';
import { StockFields } from './StockFields';
import { useFormState, type SaveState } from './useFormState';

const STEPS: Group[] = ['details', 'condition', 'product'];

const SAVE_LABEL: Record<SaveState, string> = { idle: '', saving: 'Saving…', saved: 'Saved', error: 'Not saved' };

/** P67 (changelog 46 (3)): shown when the linked product could not be read; the form stays editable. */
export const READ_FAILED = "Couldn't read this product from Shopify";

/** P67 (changelog 46 (6)): where the photographer is in a form (step and scroll), carried to the next form of a batch. */
export type FormPosition = { step: Group; scroll: number };

function parseJson(text: string | null): unknown {
  if (text === null) return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

/** The stored row's value in its core form, for "Looks right" (the same value, now human-owned). */
function rawValue(field: FormField): unknown {
  try {
    return JSON.parse(field.value!.value);
  } catch {
    return field.value!.value;
  }
}

type FormDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  item: Item;
  /** The session's visible photos (media column, damage-report picker). */
  photos: Photo[];
  /** SKU or barcode changed: the workspace reloads its view. */
  onItemChanged: () => void;
  /** Filled by the publish phase (P15). */
  publishSlot?: ReactNode;
  queueStrip?: ReactNode;
  queueConfirm?: () => void;
  /** P67: the batch's read of this item's linked product (read once for the batch); absent → the form reads it itself. */
  remoteRead?: Promise<FormRead | undefined>;
  /** P67: open on this step and scroll (a batch's previous form); absent → Details, top. */
  position?: FormPosition | null;
  /** P67: the step and scroll as the photographer moves (a batch remembers it). */
  onPosition?: (position: FormPosition) => void;
};

/** The product details form (NOTES §Screens 6–7): a dialog over the workspace. */
export function FormDialog({ open, onOpenChange, ...rest }: FormDialogProps) {
  const closeBody = useRef<(() => void) | null>(null);
  const changeOpen = (next: boolean) => {
    if (next) onOpenChange(true);
    else if (closeBody.current) closeBody.current();
    else onOpenChange(false);
  };
  return (
    <Dialog open={open} onOpenChange={changeOpen}>
      {/* P67 (changelog 46 (5)): focus is not put in the Item type picker on open (see FormBody). */}
      <DialogContent showCloseButton={false} onOpenAutoFocus={event => event.preventDefault()} className="flex h-[min(820px,92vh)] w-[min(1120px,96vw)] max-w-none flex-col gap-0 overflow-hidden p-0 sm:max-w-none">
        {open && <FormBody close={() => onOpenChange(false)} registerClose={close => { closeBody.current = close; }} {...rest} />}
      </DialogContent>
    </Dialog>
  );
}

type FormBodyProps = Omit<FormDialogProps, 'open' | 'onOpenChange'> & { close: () => void; registerClose: (close: () => void) => void };

function FormBody({ item, photos, onItemChanged, publishSlot, queueStrip, queueConfirm, close, registerClose, remoteRead, position, onPosition }: FormBodyProps) {
  const state = useFormState(item.id);
  const { form } = state;
  const [step, setStep] = useState<Group>(position?.step ?? 'details');
  const linked = item.origin === 'linked';
  const [reading, setReading] = useState(linked);
  const [readFailed, setReadFailed] = useState(false);
  const fieldsSection = useRef<HTMLElement | null>(null);
  const restoredScroll = useRef(!position || position.scroll === 0);
  const [itemType, setLocalItemType] = useState(item.item_type);
  const [savedType, setSavedType] = useState(item.item_type);
  const { types, failed: typesFailed } = useItemTypes(true);
  const [customizing, setCustomizing] = useState(false);
  const [definitions, setDefinitions] = useState<Definition[]>([]);
  const [research, setResearch] = useState<{ open: boolean; focus: string | null }>({ open: false, focus: null });
  const actionQueue = useRef<Promise<unknown>>(Promise.resolve());
  const closing = useRef(false);

  // P67 (changelog 46 (3)): a linked item's form shows the product's values. The product is read once (a batch
  // passes its own read); the fields wait in a quiet loading state until then, so they never show blank and jump.
  useEffect(() => {
    if (!linked) return;
    let alive = true;
    const read = remoteRead ?? readShopValues([item.id]).then(results => results[0]);
    read
      .then(async result => {
        if (!alive) return;
        if (!result || result.state === 'failed') {
          setReadFailed(true);
          return;
        }
        const changed = result.item_type !== item.item_type || result.title !== item.title;
        if (result.item_type !== item.item_type) {
          setLocalItemType(result.item_type);
          setSavedType(result.item_type);
        }
        await state.reload();
        if (changed) onItemChanged();
      })
      .catch(() => alive && setReadFailed(true))
      .finally(() => alive && setReading(false));
    return () => { alive = false; };
  }, [item.id]);

  // P67 (changelog 46 (6)): report where the photographer is, and open where the previous batch form was.
  useEffect(() => {
    onPosition?.({ step, scroll: fieldsSection.current?.scrollTop ?? 0 });
  }, [step]);
  useEffect(() => {
    if (restoredScroll.current || !form || reading || !fieldsSection.current || !position) return;
    restoredScroll.current = true;
    fieldsSection.current.scrollTop = position.scroll;
  }, [form, reading]);

  /**
   * P67 (changelog 46 (5)): a type is saved only when the photographer confirms it (leaving the picker, or
   * Return), never while a suggestion is merely shown in the field.
   */
  async function commitType(value: string) {
    if (value === savedType) return;
    if (types?.length && !types.includes(value)) return;
    try {
      await state.flush();
      await setItemType(item.id, value);
      setSavedType(value);
      await state.reload();
      onItemChanged();
    } catch (error) { toast.error(`Could not save item type: ${(error as Error).message}`); }
  }

  useEffect(() => {
    if (!customizing) return;
    listDefinitions()
      .then(setDefinitions)
      .catch(error => toast.error(`Could not load the fields: ${(error as Error).message}`));
  }, [customizing]);

  const conditionPhotos = useMemo(() => displayOrder(photos).condition, [photos]);
  const aiFields = state.fields.filter(field => field.value?.provenance === 'ai');
  const group = form?.groups.find(g => g.group === step);
  const filled = form?.groups.reduce((sum, g) => sum + g.filled, 0) ?? 0;
  const total = form?.groups.reduce((sum, g) => sum + g.total, 0) ?? 0;
  const stepIndex = STEPS.indexOf(step);

  const act = (action: () => Promise<unknown>, failure: string): Promise<unknown> => {
    const next = actionQueue.current.then(async () => {
      try {
        await action();
      } catch (error) {
        toast.error(`${failure}: ${(error as Error).message}`);
      } finally {
        await state.reload();
      }
    });
    actionQueue.current = next;
    return next;
  };

  /** P67 (changelog 46 (4)): Save as default, Save value and the saved values of one field, per item type. */
  function presetsOf(field: FormField): FieldPresets {
    const key = field.definition.key;
    const typeName = savedType || 'items without a type';
    return {
      defaultValue: parseJson(field.default),
      saved: field.saved.map(parseJson),
      onSaveDefault: value => void act(async () => {
        await setFieldDefault(savedType, key, value);
        toast.success(`Saved as the default ${field.definition.label} for ${typeName}`);
      }, 'Could not save the default'),
      onSaveValue: value => void act(() => addSavedValue(savedType, key, value), 'Could not save the value'),
      onPick: value => {
        state.edit(key, value);
        void blurField(key);
      },
      onRemoveSaved: value => void act(() => removeSavedValue(savedType, key, value), 'Could not remove the saved value'),
    };
  }
  const hasDefaults = !!form?.groups.some(g => g.fields.some(f => f.default !== null));

  async function saveAndClose() {
    if (closing.current) return;
    closing.current = true;
    const titleEdited = state.isEdited(PRODUCT_TITLE);
    await state.flush();
    if (itemType !== savedType && (!types?.length || types.includes(itemType))) await commitType(itemType);
    if (titleEdited) onItemChanged();
    close();
  }
  registerClose(() => { void saveAndClose(); });

  /** The Title is the item's title (ch05): once it is saved, the workspace reloads so its header follows. */
  async function blurField(key: string) {
    const titleEdited = key === PRODUCT_TITLE && state.isEdited(PRODUCT_TITLE);
    await state.blur();
    if (titleEdited) onItemChanged();
  }

  function finishCustomizing() {
    setCustomizing(false);
    void state.reload();
  }

  return (
    <>
      <header className="flex items-center gap-3 border-b px-5 py-3">
        <span className="grid size-8 place-content-center rounded-lg border" aria-hidden>
          <ClipboardList className="size-4" />
        </span>
        <DialogTitle className="text-lg font-bold">{itemType || item.title || 'Product details'}</DialogTitle>
        <ItemTypePicker id="form-item-type" value={itemType} types={types} failed={typesFailed}
          fromIntegration={!!form?.item_type_from_integration && itemType === savedType}
          onChange={setLocalItemType}
          onEnter={() => {
            if (!types?.length || types.includes(itemType)) void commitType(itemType);
          }}
          onBlur={() => {
            if (!types?.length || types.includes(itemType)) void commitType(itemType);
            else setLocalItemType(savedType);
          }} />
        <IdentityEditor item={item} onSaved={onItemChanged} />
        <DialogDescription className="sr-only">Product details of {item.sku}</DialogDescription>
        <span className="text-xs text-muted-foreground" role="status">
          {SAVE_LABEL[state.saveState]}
        </span>
        <div className="ml-auto flex items-center gap-2">
          {aiFields.length > 0 && (
            <Button variant="secondary" size="sm" className="text-(--ai-foreground)" onClick={() => setResearch(current => ({ open: !current.open, focus: null }))}>
              <Sparkles /> Research
            </Button>
          )}
          <Button variant={customizing ? 'secondary' : 'ghost'} size="sm" aria-pressed={customizing} onClick={() => (customizing ? finishCustomizing() : setCustomizing(true))}>
            <SlidersHorizontal /> Customize fields
          </Button>
          <Button variant="ghost" size="icon-sm" aria-label="Close" onClick={() => void saveAndClose()}>
            <X />
          </Button>
        </div>
      </header>

      {queueStrip}
      {form && <StepTabs groups={form.groups} step={step} onStep={setStep} />}

      {customizing ? (
        <CustomizeMode key={step} itemType={itemType} group={step} definitions={definitions} onDone={finishCustomizing} />
      ) : (
        <>
          <div className="flex min-h-0 flex-1">
            <MediaColumn photos={photos} step={step} />
            <section ref={fieldsSection} onScroll={event => onPosition?.({ step, scroll: event.currentTarget.scrollTop })} className="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto px-6 py-5" aria-label={`${STEP_LABEL[step]} fields`}>
              {readFailed && (
                <p role="alert" className="rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive">
                  {READ_FAILED}
                </p>
              )}
              {(!form || reading) && <SkeletonFields n={5} />}
              {!reading && group && group.fields.length === 0 && (
                <p className="text-sm text-muted-foreground">No fields in this step. Use Customize fields to add some.</p>
              )}
              {!reading && group?.fields.map(field => (
                <MetafieldInput
                  key={field.definition.key}
                  field={field}
                  value={state.valueOf(field)}
                  edited={state.isEdited(field.definition.key)}
                  onChange={value => state.edit(field.definition.key, value)}
                  onBlur={() => void blurField(field.definition.key)}
                  conditionPhotos={conditionPhotos}
                  onLooksRight={f => act(() => setAttributeValue(item.id, f.definition.key, rawValue(f)), 'Could not accept')}
                  onClearAi={f => act(() => clearAttribute(item.id, f.definition.key), 'Could not clear')}
                  onSeeResearch={key => setResearch({ open: true, focus: key })}
                  presets={presetsOf(field)}
                />
              ))}
              {form && !reading && step === 'product' && <StockFields itemId={item.id} />}
            </section>
            {research.open && (
              <ResearchPanel
                fields={aiFields}
                focusKey={research.focus}
                onFocus={focus => setResearch(current => ({ ...current, focus }))}
                onAcceptAll={() => act(() => acceptAllSuggestions(item.id), 'Could not accept')}
                onClose={() => setResearch({ open: false, focus: null })}
              />
            )}
          </div>
          <footer className="flex items-center gap-3 border-t px-5 py-3">
            <Progress value={total ? (filled / total) * 100 : 0} className="w-16" aria-label="Filled" />
            <span className="text-xs text-muted-foreground">
              {filled} of {total} filled
            </span>
            <div className="ml-auto flex items-center gap-2">
              {hasDefaults && !reading && (
                <ActionButton variant="ghost" action={() => act(async () => {
                  await state.flush();
                  const filled = await fillDefaults(item.id);
                  toast.success(filled === 1 ? 'Filled 1 field with its default' : `Filled ${filled} fields with their defaults`);
                }, 'Could not fill defaults')}>
                  <Wand2 /> Fill defaults
                </ActionButton>
              )}
              <ActionButton variant="outline" action={saveAndClose}>
                Save &amp; close
              </ActionButton>
              {stepIndex > 0 && (
                <Button variant="ghost" onClick={() => setStep(STEPS[stepIndex - 1])}>
                  <ArrowLeft /> {STEP_LABEL[STEPS[stepIndex - 1]]}
                </Button>
              )}
              {stepIndex < STEPS.length - 1 && (
                <Button variant="outline" onClick={() => setStep(STEPS[stepIndex + 1])}>
                  {STEP_LABEL[STEPS[stepIndex + 1]]} <ArrowRight />
                </Button>
              )}
              <div data-slot="publish" className={cn(!publishSlot && 'hidden')}>
                {publishSlot}
              </div>
              {queueConfirm && <ActionButton variant="outline" action={async () => { await state.flush(); queueConfirm(); }}>Confirm &amp; next</ActionButton>}
            </div>
          </footer>
        </>
      )}
    </>
  );
}
