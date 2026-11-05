import { useEffect, useRef, useState } from 'react';
import { Hint } from '@/components/ui/tooltip';
import { STRAIGHTEN_STEP, STRAIGHTEN_STEP_SHIFT } from './useComposePrefs';

type Props = {
  value: number;
  onSet: (degrees: number, live?: boolean) => void;
  onStraightening: (value: boolean) => void;
  grid: boolean;
  onGrid: (value: boolean) => void;
};

export function StraightenControl({ value, onSet, onStraightening, grid, onGrid }: Props) {
  const rangeRef = useRef<HTMLInputElement>(null);
  const fieldRef = useRef<HTMLInputElement>(null);
  const skipBlur = useRef(false);
  const valueRef = useRef(value);
  valueRef.current = value;
  const [text, setText] = useState(value.toFixed(1));
  const [focused, setFocused] = useState(false);
  const [held, setHeld] = useState(false);
  useEffect(() => { if (!focused) setText(value.toFixed(1)); }, [value, focused]);
  useEffect(() => { onStraightening(focused || held); }, [focused, held, onStraightening]);
  useEffect(() => {
    if (!held) return;
    const release = () => { setHeld(false); rangeRef.current?.blur(); };
    window.addEventListener('pointerup', release);
    window.addEventListener('pointercancel', release);
    return () => { window.removeEventListener('pointerup', release); window.removeEventListener('pointercancel', release); };
  }, [held]);
  useEffect(() => {
    const element = rangeRef.current;
    if (!element) return;
    const change = () => onSet(Number(element.value));
    element.addEventListener('change', change);
    return () => element.removeEventListener('change', change);
  }, [onSet]);
  const commit = () => {
    const parsed = Number(text.trim().replace('−', '-').replace(',', '.'));
    if (text.trim() !== '' && Number.isFinite(parsed)) { onSet(parsed); setText(Math.max(-45, Math.min(45, Math.round(parsed * 10) / 10)).toFixed(1)); }
    else setText(valueRef.current.toFixed(1));
  };
  return <>
    <div className="compose-field compose-straighten">
      <label htmlFor="compose-straighten">Straighten</label>
      <Hint text="Straighten (double-click to level)"><input ref={rangeRef} id="compose-straighten" aria-label="Straighten" type="range" min={-45} max={45} step={0.1} value={value} onChange={event => onSet(Number(event.target.value), true)} onDoubleClick={() => onSet(0)} onPointerDown={() => setHeld(true)} onPointerUp={event => { setHeld(false); event.currentTarget.blur(); }} onPointerCancel={() => setHeld(false)} /></Hint>
    </div>
    <div className="compose-field compose-degrees">
      <label htmlFor="compose-straighten-degrees">Degrees</label>
      <Hint text="Straighten in degrees (↑ ↓ 0.1°, ⇧ 1°)"><input ref={fieldRef} id="compose-straighten-degrees" aria-label="Straighten degrees" inputMode="decimal" value={text} onChange={event => setText(event.target.value)} onFocus={() => setFocused(true)} onBlur={() => { if (!skipBlur.current) commit(); skipBlur.current = false; setFocused(false); }} onKeyDown={event => {
        event.stopPropagation();
        if (event.key === 'Enter') { event.preventDefault(); commit(); skipBlur.current = true; fieldRef.current?.blur(); }
        else if (event.key === 'Escape') { event.preventDefault(); setText(valueRef.current.toFixed(1)); skipBlur.current = true; fieldRef.current?.blur(); }
        else if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
          event.preventDefault();
          const next = Math.round((valueRef.current + (event.key === 'ArrowUp' ? 1 : -1) * (event.shiftKey ? STRAIGHTEN_STEP_SHIFT : STRAIGHTEN_STEP)) * 10) / 10;
          onSet(next);
          setText(Math.max(-45, Math.min(45, next)).toFixed(1));
        }
      }} /></Hint><span aria-hidden>°</span>
    </div>
    <Hint text="Straightening grid"><button type="button" role="switch" className="compose-tool" aria-label="Straightening grid" aria-checked={grid} onClick={() => onGrid(!grid)}>Straightening grid</button></Hint>
  </>;
}
