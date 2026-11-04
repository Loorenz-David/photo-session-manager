import { useCallback, useLayoutEffect, useRef, useState } from 'react';
import type { TileHandlers } from '@/features/workspace/PhotoGrid';
import type { PhotoMenuAction } from '@/features/workspace/PhotoMenu';
import { Stage } from '@/features/workspace/Stage';
import type { ViewMode } from '@/features/workspace/useWorkspaceState';
import { canCompose, type Composition } from '@/services/compositions';
import type { Photo } from '@/services/photos';
import { BrowserOverlay } from './BrowserOverlay';
import { ComposeStage } from './ComposeStage';
import type { Box } from './geometry';
import type { ComposeEvent } from './machine';
import { useComposeGesture } from './useComposeGesture';
import { useCompositionSaves } from './useCompositionSaves';
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
 * Compose mode (10D §What the photographer sees, plan P31): the stage becomes the editor, the right browser hides
 * behind the edge reveal, every gesture runs through the machine, changes save when a gesture ends.
 */
export function ComposeMode({ photos, active, mode, onStep, onMenu, onItemCard, tiles, onExit, onSwitch, patchPhoto }: ComposeModeProps) {
  const stageRef = useRef<HTMLDivElement>(null);
  const imageRef = useRef<HTMLDivElement>(null);
  const [stageBox, setStageBox] = useState<Box | null>(null);
  const [hint] = useState(firstOpen);
  const photosRef = useRef(photos);
  photosRef.current = photos;
  const dispatchRef = useRef<(event: ComposeEvent) => boolean>(() => false);

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
    onSaved: useCallback(
      (photoId: number, composition: Composition) => {
        patchPhoto(photoId, { composition });
        dispatchRef.current({ type: 'Saved', photoId, composition });
      },
      [patchPhoto],
    ),
    storedOf: useCallback((photoId: number) => photosRef.current.find(photo => photo.id === photoId)?.composition ?? null, []),
  });

  const controller = useComposeGesture({ stageRef, imageRef, photo: active, stageBox, onSave: save, onExit, onSwitch });
  dispatchRef.current = controller.dispatch;

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

  return (
    <>
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
          onDone: () => controller.dispatch({ type: 'KeyReturn' }),
          children: <ComposeStage photo={active} stageBox={stageBox} stageRef={stageRef} imageRef={imageRef} hint={hint} />,
        }}
      />
      <BrowserOverlay photos={photos} isIdle={controller.isIdle} onPick={pick} {...tiles} />
    </>
  );
}
