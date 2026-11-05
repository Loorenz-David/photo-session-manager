import { useCallback, useLayoutEffect, useRef, useState, useSyncExternalStore, type ReactNode } from 'react';
import { toast } from 'sonner';
import { displaySequence } from '@/features/workspace/order';
import type { TileHandlers } from '@/features/workspace/PhotoGrid';
import type { PhotoMenuAction } from '@/features/workspace/PhotoMenu';
import { Stage } from '@/features/workspace/Stage';
import type { ViewMode } from '@/features/workspace/useWorkspaceState';
import { canCompose, resetComposition, type Composition } from '@/services/compositions';
import type { Photo } from '@/services/photos';
import { BrowserOverlay } from './BrowserOverlay';
import { ComposeStage } from './ComposeStage';
import { CompositionSection } from './CompositionSection';
import type { Box } from './geometry';
import type { ComposeEvent } from './machine';
import { ToolsPanel } from './ToolsPanel';
import { useComposeGesture } from './useComposeGesture';
import { useComposeKeys } from './useComposeKeys';
import { useCompositionSaves } from './useCompositionSaves';
import { useToolsPanelState } from './useToolsPanelState';
import './compose.css';

/** `localStorage` key of the first-time line. */
export const COMPOSE_HINT_KEY = 'compose.hintShown';

type ComposeModeProps = {
  photos: readonly Photo[];
  /** The photo on the stage; `canCompose(active)` holds while this is mounted. */
  active: Photo;
  mode: ViewMode;
  onStep: (delta: number) => void;
  onMenu: (action: PhotoMenuAction, photo: Photo) => void;
  onItemCard: () => void;
  tiles: Omit<TileHandlers, 'onOpen'>;
  onExit: () => void;
  onSwitch: (photoId: number) => void;
  patchPhoto: (photoId: number, patch: Partial<Photo>) => void;
  /** The stage column's other occupants (item card, the Details and Sessions buttons). */
  children?: ReactNode;
};

/** True exactly once per Mac: the line is shown and the key set (guarded: `localStorage` may be unavailable). */
function firstOpen(): boolean {
  try {
    if (window.localStorage.getItem(COMPOSE_HINT_KEY) === '1') return false;
    window.localStorage.setItem(COMPOSE_HINT_KEY, '1');
    return true;
  } catch {
    return false;
  }
}

const sameBox = (a: Box | null, b: Box) => a !== null && a.left === b.left && a.top === b.top && a.width === b.width && a.height === b.height;

/**
 * Compose mode (10D §What the photographer sees, plans P31 and P32): the Tools panel on the left, the stage as the
 * editor, the right browser behind the edge reveal; every gesture, control and key runs through the machine and the
 * changes save when a gesture ends.
 */
export function ComposeMode({ photos, active, mode, onStep, onMenu, onItemCard, tiles, onExit, onSwitch, patchPhoto, children }: ComposeModeProps) {
  const stageRef = useRef<HTMLDivElement>(null);
  const imageRef = useRef<HTMLDivElement>(null);
  const [stageBox, setStageBox] = useState<Box | null>(null);
  const [hint] = useState(firstOpen);
  const photosRef = useRef(photos);
  photosRef.current = photos;
  const activeRef = useRef(active);
  activeRef.current = active;
  const dispatchRef = useRef<(event: ComposeEvent) => boolean>(() => false);
  const tools = useToolsPanelState();

  useLayoutEffect(() => {
    const element = stageRef.current;
    if (!element) return;
    const measure = () => {
      const rect = element.getBoundingClientRect();
      const box = { left: rect.left, top: rect.top, width: rect.width, height: rect.height };
      setStageBox(current => (sameBox(current, box) ? current : box));
    };
    measure();
    const observer = typeof ResizeObserver === 'function' ? new ResizeObserver(measure) : null;
    observer?.observe(element);
    return () => observer?.disconnect();
  }, []);

  const save = useCompositionSaves({
    onStored: useCallback((photoId: number, composition: Composition) => patchPhoto(photoId, { composition }), [patchPhoto]),
    onSettled: useCallback((photoId: number, composition: Composition) => {
      dispatchRef.current({ type: 'Saved', photoId, composition });
    }, []),
    storedOf: useCallback((photoId: number) => photosRef.current.find(photo => photo.id === photoId)?.composition ?? null, []),
  });

  const controller = useComposeGesture({ stageRef, imageRef, photo: active, stageBox, onSave: save, onExit, onSwitch });
  dispatchRef.current = controller.dispatch;
  const state = useSyncExternalStore(controller.subscribe, controller.getSnapshot);

  const pick = useCallback(
    (photoId: number) => {
      const photo = photosRef.current.find(candidate => candidate.id === photoId);
      if (!photo) return;
      if (canCompose(photo)) controller.dispatch({ type: 'SwitchPhoto', photoId, composition: photo.composition });
      else {
        controller.dispatch({ type: 'KeyReturn' });
        onSwitch(photoId);
      }
    },
    [controller, onSwitch],
  );

  /** `[` / `]`: the neighbour in the display sequence; no wrap at the ends (plan Notes). */
  const stepPhoto = useCallback(
    (delta: -1 | 1) => {
      const sequence = displaySequence(photosRef.current);
      const index = sequence.findIndex(photo => photo.id === activeRef.current.id);
      const target = sequence[index + delta];
      if (target) pick(target.id);
    },
    [pick],
  );

  /** Reset (⌘⌥R): the backend's default composition becomes the stored row and the view (machine rule 16). */
  const reset = useCallback(() => {
    const photoId = activeRef.current.id;
    resetComposition(photoId).then(
      composition => {
        patchPhoto(photoId, { composition });
        dispatchRef.current({ type: 'Saved', photoId, composition });
      },
      error => toast.error(`Not reset: ${(error as Error).message}`),
    );
  }, [patchPhoto]);

  const done = useCallback(() => controller.dispatch({ type: 'KeyReturn' }), [controller]);
  useComposeKeys({ controller, onToggleTools: tools.toggleHidden, onReset: reset, onStepPhoto: stepPhoto });

  return (
    <>
      <ToolsPanel
        hidden={tools.hidden}
        isOpen={tools.isOpen}
        onToggle={tools.toggleSection}
        sections={[
          {
            id: 'composition',
            title: 'Composition',
            content: state ? <CompositionSection photo={active} state={state} controller={controller} onReset={reset} onDone={done} /> : null,
          },
        ]}
      />
      <div className="stage-column">
        <Stage
          photos={photos}
          active={active}
          mode={mode}
          onToggleMode={() => undefined}
          onStep={onStep}
          onAction={() => undefined}
          onMenu={onMenu}
          onItemCard={onItemCard}
          compose={{
            onDone: done,
            children: <ComposeStage photo={active} stageBox={stageBox} state={state} stageRef={stageRef} imageRef={imageRef} hint={hint} />,
          }}
        />
        <BrowserOverlay photos={photos} isIdle={controller.isIdle} onPick={pick} {...tiles} />
        {children}
      </div>
    </>
  );
}
