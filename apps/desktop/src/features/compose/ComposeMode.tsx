import { useCallback, useLayoutEffect, useRef, useState, useSyncExternalStore, type ReactNode } from 'react';
import { toast } from 'sonner';
import { displaySequence } from '@/features/workspace/order';
import type { TileHandlers } from '@/features/workspace/PhotoGrid';
import type { PhotoMenuAction } from '@/features/workspace/PhotoMenu';
import { Stage } from '@/features/workspace/Stage';
import { FoldedStrip } from '@/features/workspace/FoldedStrip';
import { PhotoPanel } from '@/features/workspace/PhotoPanel';
import { pickAndImport } from '@/features/workspace/ImportDropZone';
import type { ViewMode } from '@/features/workspace/useWorkspaceState';
import { canCompose, resetComposition, type Composition } from '@/services/compositions';
import { previewUrl } from '@/services/sessions';
import type { Photo } from '@/services/photos';
import { EditorToolbar } from './EditorToolbar';
import { ComposeStage } from './ComposeStage';
import { CompositionSection } from './CompositionSection';
import type { Box } from './geometry';
import type { ComposeEvent } from './machine';
import { refit } from './refit';
import { ToolsPanel } from './ToolsPanel';
import { useComposeGesture } from './useComposeGesture';
import { useComposeKeys } from './useComposeKeys';
import { useCompositionSaves } from './useCompositionSaves';
import { useToolsPanelState } from './useToolsPanelState';
import { clipboardComposition, copyComposition, refitClipboard, useClipboardComposition } from './useClipboardComposition';
import { useComposePrefs } from './useComposePrefs';
import { FrameMeter } from './FrameMeter';
import { Loupe } from './Loupe';
import './compose.css';

/** `localStorage` key of the first-time line. */
export const COMPOSE_HINT_KEY = 'compose.hintShown';

type ComposeModeProps = {
  photos: readonly Photo[];
  /** The photo on the stage; `canCompose(active)` holds while this is mounted. */
  active: Photo;
  mode: ViewMode;
  onMenu: (action: PhotoMenuAction, photo: Photo) => void;
  onItemCard: () => void;
  tiles: Omit<TileHandlers, 'onOpen'>;
  onExit: () => void;
  onSwitch: (photoId: number) => void;
  patchPhoto: (photoId: number, patch: Partial<Photo>) => void;
  sessionsOpen?: boolean;
  onToggleSessions?: () => void;
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
export function ComposeMode({ photos, active, mode, onMenu, onItemCard, tiles, onExit, onSwitch, patchPhoto, sessionsOpen = false, onToggleSessions = () => undefined, children }: ComposeModeProps) {
  const stageRef = useRef<HTMLDivElement>(null);
  const imageRef = useRef<HTMLDivElement>(null);
  const [stageBox, setStageBox] = useState<Box | null>(null);
  const [hint] = useState(firstOpen);
  const [loupeHeld, setLoupeHeld] = useState(false);
  const [loupePinned, setLoupePinned] = useState(false);
  const [pointer, setPointer] = useState({ x: 0, y: 0 });
  const [meter, setMeter] = useState(false);
  const [browserFolded, setBrowserFolded] = useState(true);
  const [straightening, setStraightening] = useState(false);
  const prefs = useComposePrefs();
  const clipboard = useClipboardComposition();
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

  const controller = useComposeGesture({ stageRef, imageRef, photo: active, stageBox, onSave: save, onExit, onSwitch, snapEnabled: prefs.snap });
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

  /** S / Normal | Split / the menu's Split–Normal on the stage photo: the kind change runs through the machine like any control (P33). */
  const setKind = useCallback(
    (to: 'normal' | 'split') => {
      const state = controller.getSnapshot();
      if (!state || state.values.kind === to) return;
      controller.dispatch({ type: 'SetValues', values: refit(state.values, { kind: 'kind', to }, state.frame), cause: 'kind' });
    },
    [controller],
  );
  const toggleKind = useCallback(() => {
    const state = controller.getSnapshot();
    if (state) setKind(state.values.kind === 'split' ? 'normal' : 'split');
  }, [controller, setKind]);

  /**
   * The ⋯ and right-click menus while composing: Split / Normal on the stage photo (alone or as part of the selection)
   * goes through the machine so the editor follows; the workspace handles every other photo (it skips the stage photo).
   */
  const menu = useCallback(
    (action: PhotoMenuAction, photo: Photo) => {
      if (action === 'split' || action === 'normal') {
        const activeId = activeRef.current.id;
        const { selection } = tiles;
        if (photo.id === activeId || (selection.has(photo.id) && selection.has(activeId))) setKind(action);
      }
      onMenu(action, photo);
    },
    [onMenu, setKind, tiles],
  );

  const copy = () => { const current = controller.getSnapshot(); if (current) copyComposition(current.values); };
  const paste = () => { const current = controller.getSnapshot(); const clip = clipboardComposition(); if (current && clip) controller.dispatch({ type: 'SetValues', values: refitClipboard(clip, current.frame), cause: 'paste' }); };
  useComposeKeys({ controller, onToggleTools: tools.toggleHidden, onReset: reset, onStepPhoto: stepPhoto, onToggleKind: toggleKind,
    onCopy: copy, onPaste: paste, onToggleMeter: () => setMeter(value => !value), onLoupe: setLoupeHeld,
    onCycleOverlay: prefs.cycleOverlay, onToggleBrowser: () => setBrowserFolded(value => !value) });

  return (
    <>
      <ToolsPanel
        hidden={tools.hidden}
        isOpen={tools.isOpen}
        onToggle={tools.toggleSection}
        onToggleHidden={tools.toggleHidden}
        sections={[
          {
            id: 'composition',
            title: 'Composition',
            content: state ? <CompositionSection photo={active} state={state} controller={controller} onReset={reset} onDone={done} overlay={prefs.overlay} onOverlay={prefs.setOverlay} straightenGrid={prefs.straightenGrid} onStraightenGrid={prefs.setStraightenGrid} onStraightening={setStraightening} /> : null,
          },
        ]}
      />
      <div className="stage-column">
        <Stage
          photos={photos}
          active={active}
          mode={mode}
          onToggleMode={() => undefined}
          onStep={delta => stepPhoto(delta < 0 ? -1 : 1)}
          onAction={() => undefined}
          onMenu={menu}
          onItemCard={onItemCard}
          compose={{
            onDone: done,
            toolbar: <EditorToolbar onRevert={() => controller.dispatch({ type: 'KeyEsc' })} loupe={loupePinned} onLoupe={() => setLoupePinned(value => !value)} snap={prefs.snap} onSnap={() => prefs.setSnap(!prefs.snap)} onCopy={copy} onPaste={paste} canPaste={clipboard !== null} />,
            children: <>
              <ComposeStage photo={active} stageBox={stageBox} state={state} stageRef={stageRef} imageRef={imageRef} hint={hint} onPointer={setPointer} overlay={prefs.overlay} straightenGrid={prefs.straightenGrid && straightening} />
              {(loupeHeld || loupePinned) && state && active.adjusted_preview_path && <Loupe src={previewUrl(active.adjusted_preview_path)} pointer={pointer} state={state} />}
              {meter && <FrameMeter controller={controller} />}
            </>,
          }}
        />
        {children}
      </div>
      {browserFolded ? <FoldedStrip photos={photos} onUnfold={() => setBrowserFolded(false)} onOpen={pick} sessionsOpen={sessionsOpen} onToggleSessions={onToggleSessions} {...tiles} onMenu={menu} /> : <PhotoPanel photos={photos} onFold={() => setBrowserFolded(true)} onImport={() => pickAndImport()} onOpen={pick} sessionsOpen={sessionsOpen} onToggleSessions={onToggleSessions} {...tiles} onMenu={menu} />}
    </>
  );
}
