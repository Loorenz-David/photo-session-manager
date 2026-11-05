import { useCallback, useEffect, useState } from 'react';
import { ArrowDown, ArrowLeft, ArrowRight, ArrowUp, FlipHorizontal2, Move, RectangleVertical, RotateCcw, RotateCw } from 'lucide-react';
import { DropdownMenu, Popover } from 'radix-ui';
import { Hint } from '@/components/ui/tooltip';
import type { Photo } from '@/services/photos';
import type { ComposeState } from './machine';
import type { SetValuesCause } from './machine';
import { OutputPreview } from './OutputPreview';
import { refit, type RefitChange } from './refit';
import { SplitPreviews } from './SplitPreviews';
import type { ComposeController } from './useComposeGesture';
import { StraightenControl } from './StraightenControl';
import { OVERLAY_ORDER, type OverlayKind } from './useComposePrefs';

/** The Shape menu's entries (10D: Original, 1:1, 4:5, 3:2, 2:3, 16:9, Free; labels are the contract). */
export const SHAPE_OPTIONS: readonly { value: string; label: string }[] = [
  { value: 'original', label: 'Original' },
  { value: '1:1', label: 'Square (1:1)' },
  { value: '4:5', label: '4:5' },
  { value: '3:2', label: '3:2' },
  { value: '2:3', label: '2:3' },
  { value: '16:9', label: '16:9' },
  { value: 'free', label: 'Free' },
];

type CompositionSectionProps = {
  photo: Photo;
  /** The machine's published state (what the controls show). */
  state: ComposeState;
  controller: ComposeController;
  onReset: () => void;
  onDone: () => void;
  overlay: OverlayKind;
  onOverlay: (value: OverlayKind) => void;
  straightenGrid: boolean;
  onStraightenGrid: (value: boolean) => void;
  onStraightening: (value: boolean) => void;
};

/**
 * The Composition section of the Tools panel (10D bullet 3; plan P32 task 4, P33 task 2): Normal | Split, the Shape
 * menu, Turn, rotate, flip, straighten, the output preview(s) and Centre / Reset / Done. Every control runs through the
 * machine as a `SetValues` of `refit(values, change)`; the straighten slider is live on `input` and saves on `change`;
 * while split the frame is 2:1 and the preview shows the two squares, Left and Right.
 */
export function CompositionSection({ photo, state, controller, onReset, onDone, overlay, onOverlay, straightenGrid, onStraightenGrid, onStraightening }: CompositionSectionProps) {
  const { values } = state;
  const [nudgeOpen, setNudgeOpen] = useState(false);
  useEffect(() => {
    if (!nudgeOpen) return;
    const dismiss = (event: PointerEvent) => {
      const target = event.target as Element;
      if (!target.closest('#compose-nudge-pad, [aria-label="Nudge"][data-state]')) setNudgeOpen(false);
    };
    document.addEventListener('pointerdown', dismiss, true);
    return () => document.removeEventListener('pointerdown', dismiss, true);
  }, [nudgeOpen]);

  const set = useCallback(
    (change: RefitChange, cause: SetValuesCause = 'set') => {
      const current = controller.getSnapshot();
      if (!current) return;
      controller.dispatch({ type: 'SetValues', values: refit(current.values, change, current.frame), cause });
    },
    [controller],
  );

  const setKind = (to: 'normal' | 'split') => {
    if (values.kind === to) return;
    set({ kind: 'kind', to }, 'kind');
  };

  const straighten = values.straighten ?? 0;
  return (
    <div className="composition-section">
      <div className="compose-kind" role="group" aria-label="Kind">
        <Hint text={{ name: 'Normal', key: 'S', description: 'One frame, one file' }}><button type="button" className="compose-segment" aria-pressed={values.kind === 'normal'} onClick={() => setKind('normal')}>
          Normal
        </button></Hint>
        <Hint text={{ name: 'Split', key: 'S', description: 'One 2:1 frame cut into two square files, Left and Right' }}><button
          type="button"
          className="compose-segment"
          aria-pressed={values.kind === 'split'}
          onClick={() => setKind('split')}
        >
          Split
        </button></Hint>
      </div>

      <div className="compose-field">
        <label htmlFor="compose-shape">Shape</label>
        <Hint text={{ name: 'Shape', key: '1–6, 0', description: 'Original, Square, 4:5, 3:2, 2:3, 16:9 or Free' }}><select id="compose-shape" value={values.shape} onChange={event => { set({ kind: 'shape', shape: event.target.value }); event.target.blur(); }}>
          {SHAPE_OPTIONS.map(option => <option key={option.value} value={option.value}>{option.label}</option>)}
        </select></Hint>
        <Hint text={{ name: 'Turn frame', key: 'X', description: 'Portrait or landscape, same centre and area' }}><button type="button" className="compose-tool" aria-label="Turn frame" onClick={() => set({ kind: 'orientation', orientation: values.orientation === 'turned' ? 'named' : 'turned' })}><RectangleVertical aria-hidden /></button></Hint>
      </div>

      <label className="compose-field" htmlFor="compose-overlay"><span>Overlay</span>
        <Hint text={{ name: 'Overlay', key: 'O', description: 'Guides inside the frame; O cycles them' }}><select id="compose-overlay" value={overlay} onChange={event => { onOverlay(event.target.value as OverlayKind); event.target.blur(); }}>
          {OVERLAY_ORDER.map(value => <option key={value} value={value}>{value === 'golden' ? 'Golden ratio' : value[0].toUpperCase() + value.slice(1)}</option>)}
        </select></Hint>
      </label>

      <StraightenControl value={straighten} onSet={(degrees, live) => set({ kind: 'straighten', degrees }, live ? 'live' : 'set')} onStraightening={onStraightening} grid={straightenGrid} onGrid={onStraightenGrid}>
        <DropdownMenu.Root>
          <Hint text={{ name: 'Rotate and flip', description: 'Quarter turns and a mirror for the photo' }}><DropdownMenu.Trigger asChild><button type="button" className="compose-tool" aria-label="Rotate and flip"><RotateCw aria-hidden /></button></DropdownMenu.Trigger></Hint>
          <DropdownMenu.Portal><DropdownMenu.Content className="compose-menu" sideOffset={4}>
            <DropdownMenu.Item className="compose-menu-item" onSelect={() => set({ kind: 'turn', direction: 'left' })}><RotateCcw aria-hidden />Rotate 90° left</DropdownMenu.Item>
            <DropdownMenu.Item className="compose-menu-item" onSelect={() => set({ kind: 'turn', direction: 'right' })}><RotateCw aria-hidden />Rotate 90° right</DropdownMenu.Item>
            <DropdownMenu.Item className="compose-menu-item" onSelect={() => set({ kind: 'flip' })}><FlipHorizontal2 aria-hidden />Flip horizontal</DropdownMenu.Item>
          </DropdownMenu.Content></DropdownMenu.Portal>
        </DropdownMenu.Root>
        <Popover.Root modal={false} open={nudgeOpen} onOpenChange={setNudgeOpen}>
          <Hint text={{ name: 'Nudge', key: '← ↑ → ↓', description: 'Moves the frame one screen pixel, ⇧ ×10' }}><Popover.Trigger asChild><button type="button" className="compose-tool" aria-label="Nudge"><Move aria-hidden /></button></Popover.Trigger></Hint>
          <Popover.Portal><Popover.Content id="compose-nudge-pad" role="group" aria-label="Nudge" className="compose-nudge-pad compose-dropdown" onOpenAutoFocus={event => event.preventDefault()} sideOffset={4}>
            {([['up', 0, -1, '↑', ArrowUp], ['left', -1, 0, '←', ArrowLeft], ['down', 0, 1, '↓', ArrowDown], ['right', 1, 0, '→', ArrowRight]] as const).map(([direction, x, y, key, Icon]) =>
              <Hint key={direction} text={{ name: `Nudge ${direction}`, key, description: `One screen pixel ${direction}, ⇧ ×10` }}><button type="button" className="compose-tool" aria-label={`Nudge ${direction}`} onClick={event => controller.dispatch({ type: 'Nudge', dx: -x * (event.shiftKey ? 10 : 1), dy: -y * (event.shiftKey ? 10 : 1) })}><Icon aria-hidden /></button></Hint>)}
          </Popover.Content></Popover.Portal>
        </Popover.Root>
      </StraightenControl>

      {photo.composition.frame && (values.kind === 'split' ? <SplitPreviews photo={photo} values={values} frame={state.frame} /> : <OutputPreview photo={photo} values={values} frame={state.frame} />)}

      <div className="compose-actions">
        <Hint text={{ name: 'Centre', key: '.', description: 'Centres the frame on the photo' }}><button type="button" className="compose-tool" onClick={() => set({ kind: 'centre' })}>
          Centre
        </button></Hint>
        <Hint text={{ name: 'Reset', key: '⌘⌥R', description: 'Back to the default framing' }}><button type="button" className="compose-tool" onClick={onReset}>
          Reset
        </button></Hint>
        <Hint text={{ name: 'Done', key: 'Return', description: 'Keeps the changes and leaves the editor' }}><button type="button" className="compose-done tools-done" onClick={onDone}>
          Done
        </button></Hint>
      </div>
    </div>
  );
}
