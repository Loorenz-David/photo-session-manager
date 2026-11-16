import { useEffect, useMemo, useState } from 'react';
import { DndContext, PointerSensor, useSensor, useSensors, type DragEndEvent } from '@dnd-kit/core';
import { SortableContext, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogTitle } from '@/components/ui/dialog';
import { ItemTypePicker, useItemTypes } from '@/features/form/ItemTypePicker';
import {
  addContainer, createTemplate, deleteTemplate, getTemplate, listTemplates, pickExampleFiles,
  removeContainer, reorderContainers, setExample, setPresets, setShootingOrder,
  type Presets, type Template, type TemplateSummary,
} from '@/services/templates';
import { TemplateExample } from './TemplateExample';

type Container = Template['containers'][number];
const SHAPES = [
  { value: '', label: 'None' }, { value: 'original', label: 'Original' },
  { value: '1:1', label: '1:1' }, { value: '4:5', label: '4:5' },
  { value: '3:2', label: '3:2' }, { value: '2:3', label: '2:3' }, { value: '16:9', label: '16:9' },
];

function TemplateCard({ container, onMove, onTap, onPresets, onReplace, onRemove, shooting }:
  { container: Container; onMove: (direction: -1 | 1) => void; onTap: () => void;
    onPresets: (presets: Presets) => void; onReplace: () => void; onRemove: () => void; shooting: boolean }) {
  const sortable = useSortable({ id: container.id, disabled: shooting });
  return <article ref={sortable.setNodeRef} className="template-card" data-testid={`template-card-${container.id}`}
    style={{ transform: CSS.Transform.toString(sortable.transform), transition: sortable.transition }}>
    <div className="template-card-heading">
      <button type="button" className="template-drag-handle" aria-label={`Drag container ${container.final_no}`}
        {...sortable.attributes} {...sortable.listeners}>⋮⋮</button>
      <strong>Container {container.final_no}</strong>
      <span className="template-shoot-badge" aria-label={`Shooting number ${container.shoot_no}`}>Shoot {container.shoot_no}</span>
    </div>
    <TemplateExample path={container.example_path} />
    {shooting ? <Button type="button" onClick={onTap}>Tap container {container.final_no}</Button> : <>
      <div className="template-card-controls">
        <label><input type="checkbox" aria-label={`Split container ${container.final_no}`} checked={container.presets.split}
          onChange={e => onPresets({ ...container.presets, split: e.target.checked })} /> Split</label>
        <label>Shape
          <select aria-label={`Shape container ${container.final_no}`} value={container.presets.shape ?? ''}
            onChange={e => onPresets({ ...container.presets, shape: e.target.value || null })}>
            {SHAPES.map(shape => <option key={shape.value} value={shape.value}>{shape.label}</option>)}
          </select>
        </label>
        <label>Zoom %
          <input type="number" min={0} max={80} aria-label={`Zoom container ${container.final_no}`} value={container.presets.zoom}
            onChange={e => onPresets({ ...container.presets, zoom: Math.min(80, Math.max(0, Number(e.target.value) || 0)) })} />
        </label>
      </div>
      <div className="template-card-actions">
        <Button type="button" variant="outline" onClick={() => onMove(-1)} aria-label={`Move earlier container ${container.final_no}`}>Move earlier</Button>
        <Button type="button" variant="outline" onClick={() => onMove(1)} aria-label={`Move later container ${container.final_no}`}>Move later</Button>
        <Button type="button" variant="outline" onClick={onReplace}>Replace photo</Button>
        <Button type="button" variant="outline" onClick={onRemove}>Remove</Button>
      </div>
    </>}
  </article>;
}

/** Templates are local database settings; each editor action commits its own change. */
export function TemplatesSection({ active = true }: { active?: boolean }) {
  const [itemType, setItemType] = useState('');
  const [summaries, setSummaries] = useState<TemplateSummary[]>([]);
  const [template, setTemplate] = useState<Template | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [shooting, setShooting] = useState(false);
  const [tapped, setTapped] = useState<number[]>([]);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const shopTypes = useItemTypes(active);
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));

  useEffect(() => { if (active) void listTemplates().then(setSummaries).catch(e => setError(String(e))); }, [active]);
  useEffect(() => {
    let live = true;
    if (!active || itemType.trim() === '') { setTemplate(null); return; }
    void getTemplate(itemType).then(found => { if (live) setTemplate(found); }).catch(e => { if (live) setError(String(e)); });
    return () => { live = false; };
  }, [active, itemType]);
  const choices = useMemo(() => Array.from(new Set([...(shopTypes.types ?? []), ...summaries.map(s => s.item_type)])), [shopTypes.types, summaries]);
  const ordered = template?.containers.slice().sort((a,b) => a.final_no - b.final_no) ?? [];
  const refreshList = () => { void listTemplates().then(setSummaries).catch(e => setError(String(e))); };
  const failure = async (cause: unknown) => {
    setError(cause instanceof Error ? cause.message : String(cause));
    setTapped([]);
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
  const move = (container: Container, direction: -1 | 1) => {
    if (!template) return;
    const ids = ordered.map(c => c.id);
    const index = ids.indexOf(container.id);
    const other = index + direction;
    if (other < 0 || other >= ids.length) return;
    [ids[index], ids[other]] = [ids[other], ids[index]];
    void change(() => reorderContainers(template.id, ids));
  };
  const dragEnd = (event: DragEndEvent) => {
    if (!template || !event.over || event.active.id === event.over.id) return;
    const ids = ordered.map(c => c.id);
    const from = ids.indexOf(Number(event.active.id));
    const to = ids.indexOf(Number(event.over.id));
    if (from < 0 || to < 0) return;
    ids.splice(to, 0, ids.splice(from, 1)[0]);
    void change(() => reorderContainers(template.id, ids));
  };
  const tap = (container: Container) => {
    if (!template || tapped.includes(container.id)) return;
    const next = [...tapped, container.id];
    setTapped(next);
    const rest = template.containers.slice().sort((a,b) => a.shoot_no - b.shoot_no).map(c => c.id).filter(id => !next.includes(id));
    void change(() => setShootingOrder(template.id, [...next, ...rest]));
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
    <ItemTypePicker id="template-item-type" value={itemType} onChange={setItemType} types={choices} failed={shopTypes.failed} />
    {summaries.length > 0 && <div className="template-list" aria-label="Existing templates">
      {summaries.map(summary => <Button key={summary.id} type="button" variant="outline" onClick={() => setItemType(summary.item_type)}>
        {summary.item_type} · {summary.container_count} containers
      </Button>)}
    </div>}
    {error && <p role="alert">{error}</p>}
    {itemType.trim() && !template && <Button type="button" onClick={() => void create()}>Create template</Button>}
    {template && <>
      <div className="template-toolbar">
        <Button type="button" onClick={() => void addPhotos()}>Add photos</Button>
        <Button type="button" variant="outline" onClick={() => void change(() => addContainer(template.id, null).then(t => { refreshList(); return t; }))}>Add empty container</Button>
        {shooting ? <Button type="button" variant="outline" onClick={() => { setShooting(false); setTapped([]); }}>Done</Button>
          : <Button type="button" variant="outline" onClick={() => { setTapped([]); setShooting(true); }}>Set shooting order</Button>}
        <Button type="button" variant="outline" onClick={() => setDeleteOpen(true)}>Delete template</Button>
      </div>
      {shooting && <p>Tap the containers in the order you shoot them.</p>}
      <DndContext sensors={sensors} onDragEnd={dragEnd}>
        <SortableContext items={ordered.map(c => c.id)} strategy={verticalListSortingStrategy}>
          <div className="template-cards">
            {ordered.map(container => <TemplateCard key={container.id} container={container} shooting={shooting}
              onTap={() => tap(container)} onMove={direction => move(container, direction)}
              onPresets={presets => void change(() => setPresets(container.id, presets))}
              onReplace={() => void replace(container)} onRemove={() => remove(container)} />)}
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
