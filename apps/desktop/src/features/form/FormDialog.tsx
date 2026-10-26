import { useEffect, useMemo, useState, type ReactNode } from 'react';
import { ArrowLeft, ArrowRight, ClipboardList, SlidersHorizontal, Sparkles, X } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { SkeletonFields } from '@/components/pending/SkeletonFields';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { Progress } from '@/components/ui/progress';
import { cn } from '@/lib/utils';
import { useAction } from '@/lib/pending';
import {
  PRODUCT_TITLE,
  acceptAllSuggestions,
  clearAttribute,
  listDefinitions,
  setAttributeValue,
  type Definition,
  type FormField,
  type Group,
} from '@/services/attributes';
import type { Item } from '@/services/items';
import type { Photo } from '@/services/photos';
import { displayOrder } from '@/features/workspace/order';
import { CustomizeMode } from './CustomizeMode';
import { IdentityEditor } from './IdentityEditor';
import { MediaColumn } from './MediaColumn';
import { MetafieldInput } from './MetafieldInput';
import { ResearchPanel } from './ResearchPanel';
import { STEP_LABEL, StepTabs } from './StepTabs';
import { useFormState, type SaveState } from './useFormState';

const STEPS: Group[] = ['details', 'condition', 'product'];

const SAVE_LABEL: Record<SaveState, string> = { idle: '', saving: 'Saving…', saved: 'Saved', error: 'Not saved' };

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
};

/** The product details form (NOTES §Screens 6–7): a dialog over the workspace. */
export function FormDialog({ open, onOpenChange, ...rest }: FormDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton={false} className="flex h-[min(820px,92vh)] w-[min(1120px,96vw)] max-w-none flex-col gap-0 overflow-hidden p-0 sm:max-w-none">
        {open && <FormBody close={() => onOpenChange(false)} {...rest} />}
      </DialogContent>
    </Dialog>
  );
}

type FormBodyProps = Omit<FormDialogProps, 'open' | 'onOpenChange'> & { close: () => void };

function FormBody({ item, photos, onItemChanged, publishSlot, queueStrip, queueConfirm, close }: FormBodyProps) {
  const state = useFormState(item.id);
  const { form } = state;
  const [step, setStep] = useState<Group>('details');
  const [customizing, setCustomizing] = useState(false);
  const [definitions, setDefinitions] = useState<Definition[]>([]);
  const [research, setResearch] = useState<{ open: boolean; focus: string | null }>({ open: false, focus: null });

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

  const act = useAction(async (action: () => Promise<unknown>, failure: string) => {
    try {
      await action();
    } catch (error) {
      throw new Error(`${failure}: ${(error as Error).message}`);
    }
    await state.reload();
  });

  async function saveAndClose() {
    const titleEdited = state.isEdited(PRODUCT_TITLE);
    await state.flush();
    if (titleEdited) onItemChanged();
    close();
  }

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
        <DialogTitle className="text-lg font-bold">{item.item_type || item.title || 'Product details'}</DialogTitle>
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
          <Button variant="ghost" size="icon-sm" aria-label="Close" onClick={close}>
            <X />
          </Button>
        </div>
      </header>

      {queueStrip}
      {form && <StepTabs groups={form.groups} step={step} onStep={setStep} />}

      {customizing ? (
        <CustomizeMode key={step} itemType={item.item_type} group={step} definitions={definitions} onDone={finishCustomizing} />
      ) : (
        <>
          <div className="flex min-h-0 flex-1">
            <MediaColumn photos={photos} step={step} />
            <section className="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto px-6 py-5" aria-label={`${STEP_LABEL[step]} fields`}>
              {!form && <SkeletonFields n={5} />}
              {group && group.fields.length === 0 && (
                <p className="text-sm text-muted-foreground">No fields in this step. Use Customize fields to add some.</p>
              )}
              {group?.fields.map(field => (
                <MetafieldInput
                  key={field.definition.key}
                  field={field}
                  value={state.valueOf(field)}
                  edited={state.isEdited(field.definition.key)}
                  onChange={value => state.edit(field.definition.key, value)}
                  onBlur={() => void blurField(field.definition.key)}
                  conditionPhotos={conditionPhotos}
                  onLooksRight={f => act.run(() => setAttributeValue(item.id, f.definition.key, rawValue(f)), 'Could not accept')}
                  onClearAi={f => act.run(() => clearAttribute(item.id, f.definition.key), 'Could not clear')}
                  onSeeResearch={key => setResearch({ open: true, focus: key })}
                />
              ))}
            </section>
            {research.open && (
              <ResearchPanel
                fields={aiFields}
                focusKey={research.focus}
                onFocus={focus => setResearch(current => ({ ...current, focus }))}
                onAcceptAll={() => act.run(() => acceptAllSuggestions(item.id), 'Could not accept')}
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
