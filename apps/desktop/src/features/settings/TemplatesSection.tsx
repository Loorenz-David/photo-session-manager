import { useEffect, useMemo, useRef, useState } from 'react';
import { GripVertical, Info, Plus, X } from 'lucide-react';
import { DndContext, KeyboardSensor, PointerSensor, closestCenter, useSensor, useSensors, type DragEndEvent } from '@dnd-kit/core';
import { SortableContext, rectSortingStrategy, sortableKeyboardCoordinates, useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { ChoiceSelect } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { ItemTypePicker, useItemTypes } from '@/features/form/ItemTypePicker';
import {
  addContainer, createTemplate, deleteTemplate, getTemplate, listTemplates, pickExampleFiles,
  removeContainer, reorderContainers, setExample, setLabel, setPresets, setShootingOrder,
  type Presets, type Template, type TemplateSummary,
} from '@/services/templates';
import { PresetPreview } from './PresetPreview';

type Container = Template['containers'][number];
type Order = 'final' | 'shooting';
const SHAPES = [
  { value: '', label: 'As the session' }, { value: 'original', label: 'Original' },
  { value: '1:1', label: '1:1' }, { value: '4:5', label: '4:5' },
  { value: '3:2', label: '3:2' }, { value: '2:3', label: '2:3' }, { value: '16:9', label: '16:9' },
];
const ZOOM_SAVE_MS = 300;

/** The ids of `ids` with `active` moved to where `over` is (dnd-kit's sortable move). */
export function moved(ids: number[], active: number, over: number): number[] | null {
  const from = ids.indexOf(active);
  const to = ids.indexOf(over);
  if (from < 0 || to < 0 || from === to) return null;
  const next = ids.slice();
  next.splice(to, 0, next.splice(from, 1)[0]);
  return next;
}

/** The container's zoom slider: the preview follows the slider at once, the value is saved once it rests. */
function useZoomDraft(container: Container, onPresets: (presets: Presets) => void) {
  const [draft, setDraft] = useState(container.presets.zoom);
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => { setDraft(container.presets.zoom); }, [container.presets.zoom]);
  useEffect(() => () => window.clearTimeout(timer.current), []);
  const change = (zoom: number) => {
    setDraft(zoom);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => { if (zoom !== container.presets.zoom) onPresets({ ...container.presets, zoom }); }, ZOOM_SAVE_MS);
  };
  return [draft, change] as const;
}

function DescriptionButton({ container, title, onSave }: { container: Container; title: string; onSave: (description: string) => void }) {
  const [open, setOpen] = useState(false);
  const [text, setText] = useState(container.description ?? '');
  useEffect(() => { if (open) setText(container.description ?? ''); }, [open, container.description]);
  return <Popover open={open} onOpenChange={setOpen}>
    <PopoverTrigger asChild>
      <button type="button" className="template-icon-button" aria-label={`Description, ${title}`} data-filled={container.description ? true : undefined}><Info aria-hidden /></button>
    </PopoverTrigger>
    <PopoverContent align="end" className="w-80">
      <label className="flex flex-col gap-1 text-sm font-medium">How to take this shot
        <Textarea aria-label={`Description of ${title}`} value={text} placeholder="Where you stand, the height, what must be in the frame…"
          onChange={event => setText(event.target.value)} />
      </label>
      <div className="flex justify-end gap-2">
        <Button type="button" variant="outline" size="sm" onClick={() => setOpen(false)}>Cancel</Button>
        <Button type="button" size="sm" onClick={() => { onSave(text); setOpen(false); }}>Save description</Button>
      </div>
    </PopoverContent>
  </Popover>;
}

function RemoveButton({ title, onRemove }: { title: string; onRemove: () => void }) {
  const [open, setOpen] = useState(false);
  return <Popover open={open} onOpenChange={setOpen}>
    <PopoverTrigger asChild>
      <button type="button" className="template-icon-button" aria-label={`Remove ${title}`}><X aria-hidden /></button>
    </PopoverTrigger>
    <PopoverContent align="end" className="w-64">
      <p>Remove {title} and its example photo?</p>
      <div className="flex justify-end gap-2">
        <Button type="button" variant="outline" size="sm" onClick={() => setOpen(false)}>Keep</Button>
        <Button type="button" size="sm" onClick={() => { setOpen(false); onRemove(); }}>Remove</Button>
      </div>
    </PopoverContent>
  </Popover>;
}

function TemplateCard({ container, order, onPresets, onLabel, onReplace, onRemove }:
  { container: Container; order: Order; onPresets: (presets: Presets) => void;
    onLabel: (name: string, description: string) => void; onReplace: () => void; onRemove: () => void }) {
  const sortable = useSortable({ id: container.id });
  const [zoom, setZoom] = useZoomDraft(container, onPresets);
  const number = order === 'shooting' ? container.shoot_no : container.final_no;
  const title = container.name ?? `Container ${container.final_no}`;
  const saveName = (value: string) => { if (value.trim() !== (container.name ?? '')) onLabel(value, container.description ?? ''); };
  return <article ref={sortable.setNodeRef} className="template-card" data-testid={`template-card-${container.id}`} data-dragging={sortable.isDragging || undefined}
    style={{ transform: CSS.Translate.toString(sortable.transform), transition: sortable.transition }}>
    <div className="template-card-heading">
      <button type="button" className="template-drag-handle" aria-label={`Drag ${title}`} {...sortable.attributes} {...sortable.listeners}><GripVertical aria-hidden /></button>
      <span className="template-card-number" aria-label={order === 'shooting' ? `Shot ${number}` : `Image ${number}`}>{number}</span>
      <Input key={`${container.id}-${container.name ?? ''}`} className="template-name" aria-label={`Name of container ${container.final_no}`}
        placeholder={`Container ${container.final_no}`} defaultValue={container.name ?? ''} maxLength={60}
        onBlur={event => saveName(event.target.value)}
        onKeyDown={event => { if (event.key === 'Enter') event.currentTarget.blur(); if (event.key === 'Escape') { event.currentTarget.value = container.name ?? ''; event.currentTarget.blur(); } }} />
      <DescriptionButton container={container} title={title} onSave={description => onLabel(container.name ?? '', description)} />
      <RemoveButton title={title} onRemove={onRemove} />
    </div>
    <button type="button" className="template-replace" onClick={onReplace} aria-label={container.example_path ? `Replace photo, ${title}` : `Add photo, ${title}`}>
      <PresetPreview path={container.example_path} presets={{ ...container.presets, zoom }} />
      <span className="template-replace-hint">{container.example_path ? 'Replace photo' : 'Add photo'}</span>
    </button>
    <span className="template-shoot-badge" aria-label={order === 'shooting' ? `Final image ${container.final_no}` : `Shooting number ${container.shoot_no}`}>
      {order === 'shooting' ? `Image ${container.final_no}` : `Shot ${container.shoot_no}`}
    </span>
    <div className="template-card-controls">
      <label className="template-split"><Checkbox aria-label={`Split container ${container.final_no}`} checked={container.presets.split}
        onCheckedChange={checked => onPresets({ ...container.presets, split: checked === true })} /> Split into Left and Right</label>
      <span className="template-control-label" aria-hidden>Shape</span>
      <ChoiceSelect className="template-shape-select" aria-label={`Shape container ${container.final_no}`} value={container.presets.shape ?? ''} choices={SHAPES}
        onValueChange={shape => onPresets({ ...container.presets, shape: shape || null })} />
      <span className="template-control-label" aria-hidden>Zoom</span>
      <div className="template-zoom">
        <input type="range" min={0} max={80} step={5} aria-label={`Zoom container ${container.final_no}`} value={zoom}
          onChange={event => setZoom(Number(event.target.value))} />
        <output aria-label={`Zoom of container ${container.final_no}`}>{zoom === 0 ? 'None' : `${zoom} % · ${(100 / (100 - zoom)).toFixed(2).replace(/\.?0+$/, '')}×`}</output>
      </div>
    </div>
  </article>;
}

/** Templates are local database settings; each editor action commits its own change. */
export function TemplatesSection({ active = true }: { active?: boolean }) {
  const [itemType, setItemType] = useState('');
  const [summaries, setSummaries] = useState<TemplateSummary[]>([]);
  const [template, setTemplate] = useState<Template | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [order, setOrder] = useState<Order>('final');
  const [deleteOpen, setDeleteOpen] = useState(false);
  const shopTypes = useItemTypes(active);
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  useEffect(() => { if (active) void listTemplates().then(setSummaries).catch(e => setError(String(e))); }, [active]);
  useEffect(() => {
    let live = true;
    if (!active || itemType.trim() === '') { setTemplate(null); return; }
    void getTemplate(itemType).then(found => { if (live) setTemplate(found); }).catch(e => { if (live) setError(String(e)); });
    return () => { live = false; };
  }, [active, itemType]);
  // Types with a template come first, then the shop's other types.
  const choices = useMemo(() => Array.from(new Set([...summaries.map(s => s.item_type), ...(shopTypes.types ?? [])])), [shopTypes.types, summaries]);
  const hints = useMemo(() => Object.fromEntries(summaries.map(s => [s.item_type, s.container_count === 1 ? '1 container' : `${s.container_count} containers`])), [summaries]);
  const ordered = template?.containers.slice().sort((a, b) => order === 'shooting' ? a.shoot_no - b.shoot_no : a.final_no - b.final_no) ?? [];
  const refreshList = () => { void listTemplates().then(setSummaries).catch(e => setError(String(e))); };
  const failure = async (cause: unknown) => {
    setError(cause instanceof Error ? cause.message : String(cause));
    if (itemType.trim()) {
      try { setTemplate(await getTemplate(itemType)); } catch { /* keep the visible failure */ }
    }
  };
  const change = async (op: () => Promise<Template>) => {
    try { setTemplate(await op()); setError(null); } catch (cause) { await failure(cause); }
  };
  const create = async () => {
    try { setTemplate(await createTemplate(itemType)); setError(null); refreshList(); } catch (cause) { await failure(cause); }
  };
  const addPhotos = async () => {
    if (!template) return;
    try {
      const paths = await pickExampleFiles();
      let latest: Template | null = null;
      for (const path of paths) latest = await addContainer(template.id, path);
      if (latest) { setTemplate(latest); refreshList(); setError(null); }
    } catch (cause) { await failure(cause); }
  };
  const dragEnd = (event: DragEndEvent) => {
    if (!template || !event.over) return;
    const ids = moved(ordered.map(c => c.id), Number(event.active.id), Number(event.over.id));
    if (!ids) return;
    // Show the new order at once; the stored template replaces it when the write returns.
    const position = new Map(ids.map((id, index) => [id, index + 1]));
    const key = order === 'shooting' ? 'shoot_no' : 'final_no';
    setTemplate({ ...template, containers: template.containers.map(c => ({ ...c, [key]: position.get(c.id) ?? c[key] })) });
    void change(() => order === 'shooting' ? setShootingOrder(template.id, ids) : reorderContainers(template.id, ids));
  };
  const replace = async (container: Container) => {
    try {
      const paths = await pickExampleFiles();
      if (paths[0]) await change(() => setExample(container.id, paths[0]));
    } catch (cause) { await failure(cause); }
  };
  const remove = (container: Container) => { void change(() => removeContainer(container.id).then(t => { refreshList(); return t; })); };
  const deleteCurrent = async () => {
    if (!template) return;
    try { await deleteTemplate(template.id); setTemplate(null); setError(null); refreshList(); } catch (cause) { await failure(cause); }
  };

  return <div className="templates-editor">
    <h2>Shooting templates</h2>
    <p className="health-detail">Choose an item type to see or make its template. Types that already have one show how many containers it has.</p>
    <ItemTypePicker id="template-item-type" value={itemType} onChange={setItemType} types={choices} failed={shopTypes.failed} hints={hints} />
    {error && <p role="alert">{error}</p>}
    {itemType.trim() && !template && <Button type="button" onClick={() => void create()}>Create template</Button>}
    {template && <>
      <div className="template-toolbar">
        <div className="template-order-switch" role="group" aria-label="Order">
          <button type="button" aria-pressed={order === 'final'} onClick={() => setOrder('final')}>Final order</button>
          <button type="button" aria-pressed={order === 'shooting'} onClick={() => setOrder('shooting')}>Shooting order</button>
        </div>
        <Button type="button" variant="outline" className="template-delete" onClick={() => setDeleteOpen(true)}>Delete template</Button>
      </div>
      <p className="health-detail">{order === 'shooting'
        ? 'Drag the containers into the order you shoot them. The final image order stays as it is.'
        : 'Drag the containers into the order the images are published in.'}</p>
      <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={dragEnd}>
        <SortableContext items={ordered.map(c => c.id)} strategy={rectSortingStrategy}>
          <div className="template-cards">
            {ordered.map(container => <TemplateCard key={container.id} container={container} order={order}
              onPresets={presets => void change(() => setPresets(container.id, presets))}
              onLabel={(name, description) => void change(() => setLabel(container.id, name, description))}
              onReplace={() => void replace(container)} onRemove={() => remove(container)} />)}
            <div className="template-add-tile">
              <button type="button" className="template-add-main" onClick={() => void addPhotos()}>
                <Plus aria-hidden /><span>Add container</span><small>Choose one or more example photos</small>
              </button>
              <button type="button" className="template-add-empty"
                onClick={() => void change(() => addContainer(template.id, null).then(t => { refreshList(); return t; }))}>Add without a photo</button>
            </div>
          </div>
        </SortableContext>
      </DndContext>
    </>}
    <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
      <DialogContent>
        <DialogTitle>Delete shooting template?</DialogTitle>
        <DialogDescription>Delete the template for {template?.item_type}? Its example photos will also be removed.</DialogDescription>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => setDeleteOpen(false)}>Cancel</Button>
          <Button type="button" onClick={() => { setDeleteOpen(false); void deleteCurrent(); }}>Delete template</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>;
}
