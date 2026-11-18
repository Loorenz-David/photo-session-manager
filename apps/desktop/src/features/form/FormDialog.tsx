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
  acceptSuggestion,
  addSavedValue,
  clearAttribute,
  fillDefaults,
  listDefinitions,
  readShopValues,
  removeSavedValue,
  resolveConflict,
  setFieldDefault,
  type Definition,
  type FormField,
  type FormRead,
  type Group,
} from '@/services/attributes';
import { setItemType, type Item } from '@/services/items';
import type { Photo } from '@/services/photos';
import { onResearchChanged } from '@/services/events';
import { researchStatus, runResearch, type ResearchStatus } from '@/services/research';
import { displayOrder } from '@/features/workspace/order';
import { CustomizeMode } from './CustomizeMode';
import { IdentityEditor } from './IdentityEditor';
import { ItemTypePicker, useItemTypes } from './ItemTypePicker';
import { MediaColumn } from './MediaColumn';
import { MetafieldInput, type FieldPresets } from './MetafieldInput';
import type { ConflictActions } from './AiPopover';
import { AiReviewTab, type ReviewEntry } from './AiReviewTab';
import { ResearchPanel } from './ResearchPanel';
import { ReviewList } from './ReviewList';
import { REVIEW_LABEL, REVIEW_STEP, STEP_LABEL, StepTabs, type Step } from './StepTabs';
import { StockFields } from './StockFields';
import { isEmptyValue, storedValue, useFormState, type SaveState } from './useFormState';

const STEPS: Group[] = ['details', 'condition', 'product'];

const SAVE_LABEL: Record<SaveState, string> = { idle: '', saving: 'Saving…', saved: 'Saved', error: 'Not saved' };

/** P67 (changelog 46 (3)): shown when the linked product could not be read; the form stays editable. */
export const READ_FAILED = "Couldn't read this product from Shopify";

/** P67 (changelog 46 (6)): where the photographer is in a form (step and scroll), carried to the next form of a batch. */
export type FormPosition = { step: Step; scroll: number };

function parseJson(text: string | null): unknown {
  if (text === null) return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
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
  /** The item's session: its research status and runs (08A §1). Absent → no research state is read. */
  sessionId?: number;
  /** A19: opened without a position (Details button, D), the form lands on AI review when suggestions wait. */
  reviewFirst?: boolean;
  /** A19: the review queue: "Next to review" on the AI review tab once every suggestion is checked. */
  review?: { left: number; onNext: () => void };
};

/** "n to review" (08B MC-A10, derived): the form's fields holding a live `ai` + `suggested` value, at every confidence. */
export function toReview(fields: FormField[]): number {
  return fields.filter(field => field.value?.provenance === 'ai' && field.value.ai_status === 'suggested').length;
}

/**
 * The session's research status, re-read on `research_changed` for the session. Null while research is not available
 * (before the engine is wired every `research_*` command answers `research_unavailable`): the form works as before.
 */
function useResearchStatus(sessionId: number | undefined, onChanged: () => void) {
  const [status, setStatus] = useState<ResearchStatus | null>(null);
  const changed = useRef(onChanged);
  changed.current = onChanged;
  const latest = useRef(0);
  const reload = useMemo(() => async () => {
    if (sessionId === undefined) return;
    const mine = ++latest.current;
    try {
      const next = await researchStatus(sessionId);
      if (mine === latest.current) setStatus(next);
    } catch {
      if (mine === latest.current) setStatus(null);
    }
  }, [sessionId]);
  useEffect(() => {
    if (sessionId === undefined) return;
    let alive = true;
    let off: (() => void) | undefined;
    void reload();
    // Without the event (no Tauri runtime) the panel still re-reads the status when it opens.
    Promise.resolve()
      .then(() => onResearchChanged(changedId => {
        if (changedId !== sessionId) return;
        void reload();
        changed.current();
      }))
      .then(unlisten => { if (alive) off = unlisten; else unlisten(); })
      .catch(() => undefined);
    return () => { alive = false; off?.(); };
  }, [sessionId, reload]);
  return { status, reload };
}

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

function FormBody({ item, photos, onItemChanged, publishSlot, queueStrip, queueConfirm, close, registerClose, remoteRead, position, onPosition, sessionId, reviewFirst = false, review }: FormBodyProps) {
  const state = useFormState(item.id);
  const { form } = state;
  const [step, setStep] = useState<Step>(position?.step ?? 'details');
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
  const researchState = useResearchStatus(sessionId, () => void state.reload());
  /** A6: the field chosen in the "to review" list; `n` changes on every choice so the same field opens again. */
  const [reveal, setReveal] = useState<{ key: string | null; n: number }>({ key: null, n: 0 });
  const actionQueue = useRef<Promise<unknown>>(Promise.resolve());
  /**
   * Edits of fields in conflict (08B MC-A10, R5 projection N1). On a held publish item the first save of a pending key
   * resolves it, so these are never written on the typing debounce: only once, on Enter, leaving the field, or Save &
   * close, through the conflict's Edit.
   */
  const [conflictDrafts, setConflictDrafts] = useState<Record<string, unknown>>({});
  const conflictDraftsRef = useRef(conflictDrafts);
  conflictDraftsRef.current = conflictDrafts;
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
  // A6: the "to review" list, in step and form order; Accept all's count as in the research panel (MC-A12).
  const reviewEntries: ReviewEntry[] = (form?.groups ?? []).flatMap(g =>
    g.fields.filter(field => field.value?.provenance === 'ai' && field.value.ai_status === 'suggested').map(field => ({ field, group: g.group })));
  const acceptable = reviewEntries.filter(entry => entry.field.value!.ai_confidence !== 'low' && !entry.field.value!.conflict).length;
  // A19: opened from Details or D (no position), the form lands on AI review when suggestions wait: once, on its
  // first read, so moving between steps afterwards is the owner's.
  const landed = useRef(!reviewFirst || Boolean(position));
  const waitingSuggestions = reviewEntries.length;
  useEffect(() => {
    if (landed.current || !form) return;
    landed.current = true;
    if (waitingSuggestions > 0) setStep(REVIEW_STEP);
  }, [form, waitingSuggestions]);
  /** Opens the research panel; it re-reads the status on open in case an event was missed. */
  const openResearch = (focus: string | null) => {
    setResearch({ open: true, focus });
    void researchState.reload();
  };
  const group = form?.groups.find(g => g.group === step);
  const filled = form?.groups.reduce((sum, g) => sum + g.filled, 0) ?? 0;
  const total = form?.groups.reduce((sum, g) => sum + g.total, 0) ?? 0;
  const reviewing = step === REVIEW_STEP;
  // AI review is not one of the steps: Back / Next are not shown on it.
  const stepIndex = step === REVIEW_STEP ? -1 : STEPS.indexOf(step);
  /** A11: goes to a field in its own step, scrolls to it and opens its AI popover (A6's reveal). */
  const showField = (entry: ReviewEntry) => {
    setCustomizing(false);
    setStep(entry.group);
    setReveal(current => ({ key: entry.field.definition.key, n: current.n + 1 }));
  };

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
    await Promise.all(state.fields.filter(field => field.definition.key in conflictDraftsRef.current).map(commitConflict));
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

  /** Writes a conflict field's edit once (Edit with the typed value; emptied → Clear); an unchanged edit is dropped. */
  function commitConflict(field: FormField): Promise<unknown> {
    const key = field.definition.key;
    if (!(key in conflictDraftsRef.current)) return Promise.resolve();
    const value = conflictDraftsRef.current[key];
    const rest = { ...conflictDraftsRef.current };
    delete rest[key];
    conflictDraftsRef.current = rest;
    setConflictDrafts(rest);
    if (JSON.stringify(value) === JSON.stringify(storedValue(field))) return Promise.resolve();
    return act(() => (isEmptyValue(value) ? clearAttribute(item.id, key) : resolveConflict(item.id, key, 'edit', value)), 'Could not save');
  }

  function conflictActions(field: FormField): ConflictActions | undefined {
    // MC-A1 conflict, decided by the backend (`AttributeValueDto.conflict`; R7·U1).
    if (!field.value?.conflict) return undefined;
    const key = field.definition.key;
    return {
      onKeepPrevious: () => act(() => resolveConflict(item.id, key, 'keep_previous'), 'Could not keep the previous value'),
      onUseSuggestion: () => act(() => resolveConflict(item.id, key, 'use_suggestion'), 'Could not use the suggestion'),
      onEdit: () => {
        setConflictDrafts(current => ({ ...current, [key]: state.valueOf(field) }));
        window.setTimeout(() => document.getElementById(`field-${key}`)?.focus(), 0);
      },
    };
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
          <ReviewList
            count={reviewEntries.length}
            onOpen={() => {
              setCustomizing(false);
              setStep(REVIEW_STEP);
            }}
          />
          {(aiFields.length > 0 || researchState.status !== null) && (
            <Button variant="secondary" size="sm" className="text-(--ai-foreground)" onClick={() => (research.open ? setResearch({ open: false, focus: null }) : openResearch(null))}>
              <Sparkles /> Research
            </Button>
          )}
          <Button variant={customizing ? 'secondary' : 'ghost'} size="sm" aria-pressed={customizing} onClick={() => {
            if (customizing) return finishCustomizing();
            // Fields are customized per step; from AI review, start on Details.
            if (reviewing) setStep('details');
            setCustomizing(true);
          }}>
            <SlidersHorizontal /> Customize fields
          </Button>
          <Button variant="ghost" size="icon-sm" aria-label="Close" onClick={() => void saveAndClose()}>
            <X />
          </Button>
        </div>
      </header>

      {queueStrip}
      {form && <StepTabs groups={form.groups} step={step} onStep={next => { if (next === REVIEW_STEP) setCustomizing(false); setStep(next); }} toReview={reviewEntries.length} />}

      {customizing && step !== REVIEW_STEP ? (
        <CustomizeMode key={step} itemType={itemType} group={step} definitions={definitions} onDone={finishCustomizing} />
      ) : (
        <>
          <div className="flex min-h-0 flex-1">
            <MediaColumn photos={photos} step={step === REVIEW_STEP ? 'details' : step} />
            <section ref={fieldsSection} onScroll={event => onPosition?.({ step, scroll: event.currentTarget.scrollTop })} className="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto px-6 py-5" aria-label={step === REVIEW_STEP ? REVIEW_LABEL : `${STEP_LABEL[step]} fields`}>
              {readFailed && (
                <p role="alert" className="rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive">
                  {READ_FAILED}
                </p>
              )}
              {(!form || reading) && <SkeletonFields n={5} />}
              {form && !reading && reviewing && (
                <AiReviewTab
                  entries={reviewEntries}
                  acceptable={acceptable}
                  status={researchState.status}
                  anyAi={aiFields.length > 0}
                  onLooksRight={f => act(() => acceptSuggestion(item.id, f.definition.key), 'Could not accept')}
                  onClear={f => act(() => clearAttribute(item.id, f.definition.key), 'Could not clear')}
                  onKeepPrevious={f => act(() => resolveConflict(item.id, f.definition.key, 'keep_previous'), 'Could not keep the previous value')}
                  conflictActions={entry => {
                    const actions = conflictActions(entry.field);
                    // Edit types the value in the field itself, on its own step.
                    return actions && { ...actions, onEdit: () => { setStep(entry.group); actions.onEdit(); } };
                  }}
                  onSeeResearch={key => openResearch(key)}
                  onShowField={showField}
                  onAcceptAll={() => act(() => acceptAllSuggestions(item.id), 'Could not accept')}
                  nextToReview={review && { left: review.left, onNext: () => { close(); review.onNext(); } }}
                />
              )}
              {!reading && group && group.fields.length === 0 && (
                <p className="text-sm text-muted-foreground">No fields in this step. Use Customize fields to add some.</p>
              )}
              {!reading && group?.fields.map(field => {
                const key = field.definition.key;
                const conflict = conflictActions(field);
                const editingConflict = key in conflictDrafts;
                const routed = conflict !== undefined || editingConflict;
                return (
                <div key={key} className="contents" onKeyDown={routed ? event => {
                  if (event.key === 'Enter' && event.target instanceof HTMLInputElement) {
                    event.preventDefault();
                    void commitConflict(field);
                  }
                } : undefined}>
                <MetafieldInput
                  field={field}
                  value={editingConflict ? conflictDrafts[key] : state.valueOf(field)}
                  edited={editingConflict || state.isEdited(key)}
                  onChange={routed ? value => setConflictDrafts(current => ({ ...current, [key]: value })) : value => state.edit(key, value)}
                  onBlur={routed ? () => void commitConflict(field) : () => void blurField(key)}
                  conflict={conflict}
                  conditionPhotos={conditionPhotos}
                  onLooksRight={f => act(() => acceptSuggestion(item.id, f.definition.key), 'Could not accept')}
                  onClearAi={f => act(() => clearAttribute(item.id, f.definition.key), 'Could not clear')}
                  onSeeResearch={key => openResearch(key)}
                  presets={presetsOf(field)}
                  reveal={reveal.key === key ? reveal.n : undefined}
                />
                </div>
                );
              })}
              {form && !reading && step === 'product' && <StockFields itemId={item.id} />}
            </section>
            {research.open && (
              <ResearchPanel
                status={researchState.status}
                fields={state.fields}
                onRun={async () => {
                  if (sessionId === undefined) return;
                  try {
                    await runResearch(sessionId);
                  } catch (error) {
                    toast.error(`Could not start research: ${(error as Error).message}`);
                  }
                  await researchState.reload();
                }}
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
              {stepIndex >= 0 && stepIndex < STEPS.length - 1 && (
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
