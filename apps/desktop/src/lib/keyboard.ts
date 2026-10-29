import { useEffect, useRef, useState } from 'react';

/** True when the key press belongs to a text field, so single-letter shortcuts must not fire. */
export function isTyping(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName) || target.isContentEditable;
}

type KeyMap = Partial<Record<string, (event: KeyboardEvent) => void>>;

/**
 * App keyboard map (design handoff §Keyboard). Keys are `event.key` lower-cased ("n", "escape").
 * Plain letters ignore presses with ⌘/Ctrl/Alt and presses inside text fields.
 */
export function useKeyboard(map: KeyMap, enabled = true): void {
  const current = useRef(map);
  current.current = map;
  useEffect(() => {
    if (!enabled) return;
    function onKey(event: KeyboardEvent) {
      const key = event.key.toLowerCase();
      const handler = current.current[key];
      if (!handler) return;
      if (key.length === 1 && (event.metaKey || event.ctrlKey || event.altKey || isTyping(event.target))) return;
      handler(event);
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [enabled]);
}

/** Back keys belong to pages; an open layer owns its own keyboard dismissal. */
export function useBackKeys(parent: string | null, escape: boolean): void {
  useEffect(() => {
    if (parent === null) return;
    function onKey(event: KeyboardEvent) {
      if (document.querySelector('[role="dialog"], [role="alertdialog"], [role="menu"]')) return;
      const commandBack = event.key === '[' && (event.metaKey || event.ctrlKey);
      const escapeBack = escape && event.key === 'Escape' && !isTyping(event.target);
      if (!commandBack && !escapeBack) return;
      event.preventDefault();
      window.location.hash = parent!;
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [parent, escape]);
}

/** Value that follows `value` after `ms` of quiet. */
export function useDebounced<T>(value: T, ms: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => setDebounced(value), ms);
    return () => window.clearTimeout(timer);
  }, [value, ms]);
  return debounced;
}
