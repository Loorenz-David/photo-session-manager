import { useCallback } from 'react';
import { FlipHorizontal2, RotateCcw, RotateCw } from 'lucide-react';
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
        <Hint text="Normal (S)"><button type="button" className="compose-segment" aria-pressed={values.kind === 'normal'} onClick={() => setKind('normal')}>
          Normal
        </button></Hint>
        <Hint text="Split (S)"><button
          type="button"
          className="compose-segment"
          aria-pressed={values.kind === 'split'}
          onClick={() => setKind('split')}
        >
          Split
        </button></Hint>
      </div>

      <label className="compose-field" htmlFor="compose-shape">
        <span>Shape</span>
        <Hint text="Shape (1–6, 0)"><select id="compose-shape" value={values.shape} onChange={event => { set({ kind: 'shape', shape: event.target.value }); event.target.blur(); }}>
          {SHAPE_OPTIONS.map(option => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select></Hint>
      </label>

      <label className="compose-field" htmlFor="compose-overlay"><span>Overlay</span>
        <Hint text="Overlay (O)"><select id="compose-overlay" value={overlay} onChange={event => { onOverlay(event.target.value as OverlayKind); event.target.blur(); }}>
          {OVERLAY_ORDER.map(value => <option key={value} value={value}>{value === 'golden' ? 'Golden ratio' : value[0].toUpperCase() + value.slice(1)}</option>)}
        </select></Hint>
      </label>

      <div className="compose-buttons">
        <Hint text="Turn frame (X)"><button type="button" className="compose-tool" onClick={() => set({ kind: 'orientation', orientation: values.orientation === 'turned' ? 'named' : 'turned' })}>
          Turn frame
        </button></Hint>
        <Hint text="Rotate 90° left"><button type="button" className="compose-tool" onClick={() => set({ kind: 'turn', direction: 'left' })}>
          <RotateCcw aria-hidden /> Rotate left
        </button></Hint>
        <Hint text="Rotate 90° right"><button type="button" className="compose-tool" onClick={() => set({ kind: 'turn', direction: 'right' })}>
          <RotateCw aria-hidden /> Rotate right
        </button></Hint>
        <Hint text="Flip horizontally"><button type="button" className="compose-tool" onClick={() => set({ kind: 'flip' })}>
          <FlipHorizontal2 aria-hidden /> Flip
        </button></Hint>
      </div>

      <StraightenControl value={straighten} onSet={(degrees, live) => set({ kind: 'straighten', degrees }, live ? 'live' : 'set')} onStraightening={onStraightening} grid={straightenGrid} onGrid={onStraightenGrid} />
      <div className="compose-nudge" role="group" aria-label="Nudge">
        {([['left', -1, 0, '←'], ['right', 1, 0, '→'], ['up', 0, -1, '↑'], ['down', 0, 1, '↓']] as const).map(([direction, x, y, key]) =>
          <Hint key={direction} text={`Nudge ${direction} (${key})`}><button type="button" className="compose-tool" onClick={event => controller.dispatch({ type: 'Nudge', dx: -x * (event.shiftKey ? 10 : 1), dy: -y * (event.shiftKey ? 10 : 1) })}>Nudge {direction}</button></Hint>)}
      </div>

      {photo.composition.frame && (values.kind === 'split' ? <SplitPreviews photo={photo} values={values} frame={state.frame} /> : <OutputPreview photo={photo} values={values} frame={state.frame} />)}

      <div className="compose-actions">
        <Hint text="Centre (.)"><button type="button" className="compose-tool" onClick={() => set({ kind: 'centre' })}>
          Centre
        </button></Hint>
        <Hint text="Reset (⌘⌥R)"><button type="button" className="compose-tool" onClick={onReset}>
          Reset
        </button></Hint>
        <Hint text="Done (Return)"><button type="button" className="compose-done tools-done" onClick={onDone}>
          Done
        </button></Hint>
      </div>
    </div>
  );
}
