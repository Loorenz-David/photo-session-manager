import { useCallback, useEffect, useRef } from 'react';
import { FlipHorizontal2, RotateCcw, RotateCw } from 'lucide-react';
import type { Photo } from '@/services/photos';
import type { ComposeState } from './machine';
import type { SetValuesCause } from './machine';
import { OutputPreview } from './OutputPreview';
import { refit, type RefitChange } from './refit';
import { SplitPreviews } from './SplitPreviews';
import type { ComposeController } from './useComposeGesture';

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
};

/**
 * The Composition section of the Tools panel (10D bullet 3; plan P32 task 4, P33 task 2): Normal | Split, the Shape
 * menu, Turn, rotate, flip, straighten, the output preview(s) and Centre / Reset / Done. Every control runs through the
 * machine as a `SetValues` of `refit(values, change)`; the straighten slider is live on `input` and saves on `change`;
 * while split the frame is 2:1 and the preview shows the two squares, Left and Right.
 */
export function CompositionSection({ photo, state, controller, onReset, onDone }: CompositionSectionProps) {
  const rangeRef = useRef<HTMLInputElement>(null);
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

  // A slider drag is one gesture (plan Notes): `input` moves the view, the native `change` on release saves once.
  useEffect(() => {
    const element = rangeRef.current;
    if (!element) return;
    const onChange = () => set({ kind: 'straighten', degrees: Number(element.value) }, 'set');
    element.addEventListener('change', onChange);
    return () => element.removeEventListener('change', onChange);
  }, [set]);

  const straighten = values.straighten ?? 0;
  return (
    <div className="composition-section">
      <div className="compose-kind" role="group" aria-label="Kind">
        <button type="button" className="compose-segment" aria-pressed={values.kind === 'normal'} onClick={() => setKind('normal')} title="Normal (S)">
          Normal
        </button>
        <button
          type="button"
          className="compose-segment"
          aria-pressed={values.kind === 'split'}
          onClick={() => setKind('split')}
          title="Split (S)"
        >
          Split
        </button>
      </div>

      <label className="compose-field" htmlFor="compose-shape">
        <span>Shape</span>
        <select id="compose-shape" value={values.shape} onChange={event => set({ kind: 'shape', shape: event.target.value })}>
          {SHAPE_OPTIONS.map(option => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>

      <div className="compose-buttons">
        <button type="button" className="compose-tool" onClick={() => set({ kind: 'orientation', orientation: values.orientation === 'turned' ? 'named' : 'turned' })} title="Turn frame (X)">
          Turn frame
        </button>
        <button type="button" className="compose-tool" onClick={() => set({ kind: 'turn', direction: 'left' })} title="Rotate 90° left">
          <RotateCcw aria-hidden /> Rotate left
        </button>
        <button type="button" className="compose-tool" onClick={() => set({ kind: 'turn', direction: 'right' })} title="Rotate 90° right">
          <RotateCw aria-hidden /> Rotate right
        </button>
        <button type="button" className="compose-tool" onClick={() => set({ kind: 'flip' })} title="Flip horizontally">
          <FlipHorizontal2 aria-hidden /> Flip
        </button>
      </div>

      <div className="compose-field compose-straighten">
        <label htmlFor="compose-straighten">Straighten</label>
        <input
          ref={rangeRef}
          id="compose-straighten"
          type="range"
          min={-45}
          max={45}
          step={0.1}
          value={straighten}
          onChange={event => set({ kind: 'straighten', degrees: Number(event.target.value) }, 'live')}
          onDoubleClick={() => set({ kind: 'straighten', degrees: 0 })}
          title="Double-click to level"
        />
        <output htmlFor="compose-straighten" className="compose-readout">
          {straighten.toFixed(1)}°
        </output>
      </div>

      {photo.composition.frame && (values.kind === 'split' ? <SplitPreviews photo={photo} values={values} frame={state.frame} /> : <OutputPreview photo={photo} values={values} frame={state.frame} />)}

      <div className="compose-actions">
        <button type="button" className="compose-tool" onClick={() => set({ kind: 'centre' })} title="Centre (.)">
          Centre
        </button>
        <button type="button" className="compose-tool" onClick={onReset} title="Reset to the default (⌘⌥R)">
          Reset
        </button>
        <button type="button" className="compose-done tools-done" onClick={onDone} title="Done (Return)">
          Done
        </button>
      </div>
    </div>
  );
}
