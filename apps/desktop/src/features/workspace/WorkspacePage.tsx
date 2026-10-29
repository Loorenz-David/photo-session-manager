import { useCallback, useEffect, useRef, useState } from 'react';
import {
  DndContext,
  DragOverlay,
  defaultDropAnimationSideEffects,
  PointerSensor,
  pointerWithin,
  useSensor,
  useSensors,
  type CollisionDetection,
  type DragEndEvent,
  type DragOverEvent,
  type DragStartEvent,
  type DropAnimation,
} from '@dnd-kit/core';
import { ClipboardList, PanelsRightBottom } from 'lucide-react';
import { toast } from 'sonner';
import { Skeleton } from '@/components/ui/skeleton';
import { SkeletonTiles } from '@/components/pending/SkeletonTiles';
import { assignUnassigned, type Photo } from '@/services/photos';
import { createSession } from '@/services/sessions';
import { adoptItem, type Item } from '@/services/items';
import { useKeyboard } from '@/lib/keyboard';
import { FormDialog } from '@/features/form/FormDialog';
import { FailurePopover } from '@/features/publish/FailurePopover';
import { publishRetry } from '@/services/publish';
import { FoldedStrip } from './FoldedStrip';
import { ImportDropZone, pickAndImport } from './ImportDropZone';
import { ItemDetailsCard } from './ItemDetailsCard';
import { displayOrder, displaySequence } from './order';
import { ItemPickerDialog, SessionPickerDialog } from './PickerDialogs';
import { PhotoPanel } from './PhotoPanel';
import type { PhotoMenuAction } from './PhotoMenu';
import { PhotoTileFace, PublishFailedContext } from './PhotoTile';
import { SessionsPanel } from './SessionsPanel';
import { Stage } from './Stage';
import type { SwipeAction } from './SwipeLayer';
import { useWorkspaceKeyboard } from './useKeyboard';
import { useWorkspaceState, type WorkspaceState } from './useWorkspaceState';
import './workspace.css';

export const UNASSIGNED_SESSION_ID = 1;

type WorkspacePageProps = {
  sessionId: number;
  /** "Assign product" in the header of the Unassigned workspace opens the item search. */
  assignOpen: boolean;
  onAssignOpenChange: (open: boolean) => void;
  navigate: (sessionId?: number) => void;
};

type Picker = { kind: 'move' | 'copy' | 'assign'; photoIds: number[] } | null;

const idNumber = (id: string | number, prefix: string): number | null =>
  String(id).startsWith(prefix) ? Number(String(id).slice(prefix.length)) : null;

/** Drops land on what is under the pointer; a tile wins over the group around it. */
const collisions: CollisionDetection = args => {
  const within = pointerWithin(args);
  const tile = within.find(hit => String(hit.id).startsWith('photo:'));
  return tile ? [tile] : within;
};

const dropAnimation: DropAnimation = {
  duration: 200,
  easing: 'cubic-bezier(.22, 1, .36, 1)',
  keyframes: ({ transform: { initial, final } }) => [
    { transform: `translate3d(${initial.x}px, ${initial.y}px, 0)` },
    { transform: `translate3d(${final.x}px, ${final.y}px, 0)` },
  ],
  sideEffects: defaultDropAnimationSideEffects({ styles: { active: { opacity: '0' } } }),
};

/**
 * What a drag of photo `photoId` onto `overId` does (ch06 §Order, MC-5): within a group it reorders to the slot
 * under the pointer; into the other group it classifies (end of that group, whatever the drop index); onto a
 * session row it moves the dragged set there.
 */
function useDrop(state: WorkspaceState) {
  return useCallback(
    (photoId: number, overId: string | number) => {
      const photos = state.photos;
      const photo = photos.find(candidate => candidate.id === photoId);
      if (!photo) return;
      const targetSession = idNumber(overId, 'session:');
      if (targetSession !== null) {
        const dragged = state.selection.has(photoId) ? displaySequence(photos).filter(p => state.selection.has(p.id)).map(p => p.id) : [photoId];
        void state.move(dragged, targetSession);
        return;
      }
      const order = displayOrder(photos);
      const overPhotoId = idNumber(overId, 'photo:');
      const overPhoto = overPhotoId === null ? undefined : photos.find(candidate => candidate.id === overPhotoId);
      const overGroup = String(overId).startsWith('group:') ? String(overId).slice('group:'.length) : null;
      const target = overPhoto ? (overPhoto.classification === 'condition' ? 'condition' : 'main') : overGroup;
      if (target !== 'main' && target !== 'condition') return;
      if (photo.classification !== target) {
        void state.classify(photoId, target, false);
        return;
      }
      if (!overPhoto || overPhoto.id === photoId) return;
      const group = target === 'main' ? order.main : order.condition;
      const index = group.findIndex(candidate => candidate.id === overPhoto.id);
      const position = index === -1 ? group.length : index + 1;
      if (position !== photo.position) void state.reorder(photoId, position);
    },
    [state],
  );
}

/** The session workspace (NOTES.md §Screens 4–5): stage, photo panel or strip, and in Unassigned the sessions panel. */
export function WorkspacePage({ sessionId, assignOpen, onAssignOpenChange, navigate }: WorkspacePageProps) {
  const state = useWorkspaceState(sessionId);
  const unassigned = sessionId === UNASSIGNED_SESSION_ID;
  const [folded, setFolded] = useState(false);
  const [sessionsOpen, setSessionsOpen] = useState(unassigned);
  const [itemCardOpen, setItemCardOpen] = useState(false);
  const [formOpen, setFormOpen] = useState(false);
  const [picker, setPicker] = useState<Picker>(null);
  const [draggingId, setDraggingId] = useState<number | null>(null);
  const [overId, setOverId] = useState<string | number | null>(null);
  const [failureOpen, setFailureOpen] = useState(false);
  const shownFailure = useRef<number | null>(null);
  const shownFailureSession = useRef(sessionId);
  const drop = useDrop(state);
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));
  const { active } = state;
  const dragging = state.photos.find(photo => photo.id === draggingId) ?? null;
  const ordered = displayOrder(state.photos);
  const draggedGroup = dragging?.classification === 'condition' ? ordered.condition : [...ordered.main, ...ordered.unreviewed];
  const draggedNumber = dragging ? draggedGroup.findIndex(photo => photo.id === dragging.id) + 1 : 0;
  let projectedNumber = draggedNumber;
  let projectedGroup = dragging?.classification;
  if (dragging && overId !== null && idNumber(overId, 'session:') === null && String(overId) !== `photo:${dragging.id}`) {
    const overPhotoId = idNumber(overId, 'photo:');
    const overPhoto = overPhotoId === null ? undefined : state.photos.find(photo => photo.id === overPhotoId);
    const target = overPhoto?.classification === 'condition' ? 'condition'
      : overPhoto ? 'main'
      : String(overId).startsWith('group:') ? String(overId).slice('group:'.length) : null;
    if (target === 'main' || target === 'condition') {
      if (dragging.classification !== target) {
        projectedGroup = target;
        projectedNumber = ordered[target].length + 1;
      } else if (overPhoto) {
        projectedNumber = ordered[target].findIndex(photo => photo.id === overPhoto.id) + 1;
      }
    }
  }
  const reducedMotion = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;
  const failure = state.view?.session.publish?.step === 'failed' ? state.view.session.publish : null;
  const retry = useCallback(() => {
    if (!failure) return;
    return publishRetry(failure.item_id).then(() => { setFailureOpen(false); toast.dismiss(`publish-failed-${failure.item_id}`); void state.reload(); });
  }, [failure, state]);
  useEffect(() => {
    if (shownFailureSession.current !== sessionId) {
      shownFailureSession.current = sessionId;
      shownFailure.current = null;
    }
    if (!failure) {
      shownFailure.current = null;
      return;
    }
    if (shownFailure.current === failure.item_id) return;
    shownFailure.current = failure.item_id;
    setFailureOpen(true);
    toast.error('Publish failed', { id: `publish-failed-${failure.item_id}`, duration: Infinity, action: { label: 'Retry', onClick: () => void retry()?.catch(error => toast.error(`Retry failed: ${(error as Error).message}`)) } });
  }, [failure, retry, sessionId]);

  const review = useCallback(
    (action: SwipeAction) => {
      if (!active) return;
      if (action === 'delete') void state.remove(active.id);
      else if (action === 'approve') {
        if (active.classification === 'main') state.advanceFrom(active.id);
        else void state.classify(active.id, 'main');
      } else if (active.classification === 'condition') state.advanceFrom(active.id);
      else void state.classify(active.id, 'condition');
    },
    [active, state],
  );

  /** The ⋯ / right-click menu acts on the selection when the photo is part of it, else on the photo alone. */
  const onMenu = useCallback(
    (action: PhotoMenuAction, photo: Photo) => {
      const photoIds = state.selection.has(photo.id)
        ? displaySequence(state.photos).filter(p => state.selection.has(p.id)).map(p => p.id)
        : [photo.id];
      if (action === 'toggle') void state.classify(photo.id, photo.classification === 'condition' ? 'main' : 'condition', false);
      else if (action === 'delete') void state.remove(photo.id);
      else setPicker({ kind: action, photoIds });
    },
    [state],
  );

  const assignWhole = useCallback(
    async (item: Item) => {
      try {
        const local = item.id === 0 && item.external_id ? await adoptItem(item.external_id) : item;
        const result = await assignUnassigned(local.id);
        onAssignOpenChange(false);
        const target = result.sessions.find(session => session.id !== UNASSIGNED_SESSION_ID);
        if (target) navigate(target.id);
        else await state.reload();
      } catch (error) {
        toast.error(`Could not assign the photos: ${(error as Error).message}`);
      }
    },
    [navigate, onAssignOpenChange, state],
  );

  const assignSome = useCallback(
    async (item: Item, photoIds: number[]) => {
      try {
        const local = item.id === 0 && item.external_id ? await adoptItem(item.external_id) : item;
        const session = await createSession(local.id);
        await state.move(photoIds, session.id);
        setPicker(null);
      } catch (error) {
        toast.error(`Could not assign the photos: ${(error as Error).message}`);
      }
    },
    [state],
  );

  const item = state.view?.item ?? null;
  const dialogOpen = picker !== null || assignOpen || formOpen;
  // D opens the product details form (design handoff §Keyboard); Esc closes it (the dialog's own Esc).
  useKeyboard({ d: () => setFormOpen(true) }, !dialogOpen && item !== null);
  useWorkspaceKeyboard(
    {
      approve: () => review('approve'),
      condition: () => review('condition'),
      remove: () => review('delete'),
      undo: () => void state.undo(),
      previous: () => state.step(-1),
      next: () => state.step(1),
      toggleMode: state.toggleMode,
      fold: () => setFolded(current => !current),
      itemCard: () => setItemCardOpen(current => !current),
      escape: () => {
        if (itemCardOpen) setItemCardOpen(false);
        else state.clearSelection();
      },
    },
    !dialogOpen,
  );

  function onDragStart(event: DragStartEvent) {
    const id = idNumber(event.active.id, 'photo:');
    setDraggingId(state.photos.some(photo => photo.id === id) ? id : null);
    setOverId(null);
  }

  function onDragOver(event: DragOverEvent) {
    setOverId(event.over?.id ?? null);
  }

  function onDragEnd(event: DragEndEvent) {
    const id = idNumber(event.active.id, 'photo:');
    const overId = event.over?.id;
    if (id !== null && overId !== undefined) drop(id, overId);
    setDraggingId(null);
    setOverId(null);
  }

  const tileHandlers = {
    activeId: state.activeId,
    selection: state.selection,
    onOpen: state.setActiveId,
    onToggleSelected: state.toggleSelected,
    onMenu,
  };

  if (state.loadError && !state.view) {
    return (
      <main className="workspace workspace-error">
        <p role="alert">{state.loadError}</p>
      </main>
    );
  }

  if (!state.view) return <main className="workspace" aria-busy="true">
    <div className="stage-column"><Skeleton className="h-full min-h-64 w-full" />{itemCardOpen && <ItemDetailsCard view={null} onClose={() => setItemCardOpen(false)} />}</div>
    <aside className="photo-panel" aria-label="Photos"><div className="panel-scroll"><SkeletonTiles n={8} /></div></aside>
  </main>;

  return (
    <PublishFailedContext.Provider value={failure !== null}>
    <DndContext sensors={sensors} collisionDetection={collisions} onDragStart={onDragStart} onDragOver={onDragOver} onDragEnd={onDragEnd} onDragCancel={() => { setDraggingId(null); setOverId(null); }}>
      <main className="workspace" data-folded={folded || undefined}>
        <div className="stage-column">
          <Stage
            photos={state.photos}
            active={active}
            mode={state.mode}
            onToggleMode={state.toggleMode}
            onStep={state.step}
            onAction={review}
            onMenu={onMenu}
            onItemCard={() => setItemCardOpen(current => !current)}
          />
          {itemCardOpen && state.view && <ItemDetailsCard view={state.view} onClose={() => setItemCardOpen(false)} />}
          {item && (
            <button type="button" className="sessions-toggle" style={{ top: sessionsOpen ? 42 : 74 }} onClick={() => setFormOpen(true)} aria-label="Product details" title="Product details (D)">
              <ClipboardList /> Details
            </button>
          )}
          {!sessionsOpen && (
            <button type="button" className="sessions-toggle" onClick={() => setSessionsOpen(true)} aria-label="Show sessions panel">
              <PanelsRightBottom /> Sessions
            </button>
          )}
        </div>
        {folded ? (
          <FoldedStrip photos={state.photos} onUnfold={() => setFolded(false)} {...tileHandlers} />
        ) : (
          <PhotoPanel photos={state.photos} onFold={() => setFolded(true)} onImport={() => pickAndImport()} {...tileHandlers} />
        )}
        {sessionsOpen && <SessionsPanel currentSessionId={sessionId} onClose={() => setSessionsOpen(false)} />}
        <ImportDropZone />
      </main>
      <DragOverlay dropAnimation={reducedMotion ? null : dropAnimation}>
        {dragging && <div className={`photo-tile tile-${folded ? 'strip' : 'grid'} drag-overlay`} style={{ width: '100%', height: '100%' }} data-active={state.activeId === dragging.id || undefined} data-selected={state.selection.has(dragging.id) || undefined} aria-hidden="true">
          <PhotoTileFace photo={{ ...dragging, classification: projectedGroup ?? dragging.classification }} number={projectedNumber} variant={folded ? 'strip' : 'grid'} />
        </div>}
      </DragOverlay>
      {failureOpen && failure && <FailurePopover reason={failure.error ?? 'Publish failed'} onRetry={retry} onClose={() => setFailureOpen(false)} />}

      <SessionPickerDialog
        open={picker?.kind === 'move' || picker?.kind === 'copy'}
        title={picker?.kind === 'copy' ? 'Copy to session' : 'Move to session'}
        excludeSessionId={sessionId}
        onOpenChange={open => !open && setPicker(null)}
        onPick={async row => {
          if (!picker) return;
          const { kind, photoIds } = picker;
          await (kind === 'copy' ? state.copy(photoIds, row.id) : state.move(photoIds, row.id));
          setPicker(null);
        }}
      />
      <ItemPickerDialog
        open={picker?.kind === 'assign'}
        title="Assign to product"
        onOpenChange={open => !open && setPicker(null)}
        onPick={item => { if (picker) return assignSome(item, picker.photoIds); }}
      />
      {item && <FormDialog open={formOpen} onOpenChange={setFormOpen} item={item} photos={state.photos} onItemChanged={() => void state.reload()} />}
      {unassigned && <ItemPickerDialog open={assignOpen} title="Assign product" onOpenChange={onAssignOpenChange} onPick={assignWhole} />}
    </DndContext>
    </PublishFailedContext.Provider>
  );
}
