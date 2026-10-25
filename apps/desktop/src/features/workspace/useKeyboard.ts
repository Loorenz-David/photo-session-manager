import { useEffect, useRef } from 'react';
import { isTyping } from '@/lib/keyboard';

export type WorkspaceKeys = {
  approve: () => void;
  condition: () => void;
  remove: () => void;
  undo: () => void;
  previous: () => void;
  next: () => void;
  toggleMode: () => void;
  fold: () => void;
  itemCard: () => void;
  escape: () => void;
};

/**
 * Workspace keyboard map (design handoff §Keyboard): A, C, ⌫/Delete, ⌘Z, ←/→ (↑/↓), Space, F, I, Esc.
 * Presses inside text fields are left to the field; letters with ⌘/Ctrl/Alt are not shortcuts (except ⌘Z).
 */
export function useWorkspaceKeyboard(keys: WorkspaceKeys, enabled: boolean): void {
  const current = useRef(keys);
  current.current = keys;
  useEffect(() => {
    if (!enabled) return;
    function onKey(event: KeyboardEvent) {
      if (isTyping(event.target)) return;
      const map = current.current;
      const key = event.key.toLowerCase();
      if (key === 'z' && (event.metaKey || event.ctrlKey) && !event.shiftKey) {
        event.preventDefault();
        map.undo();
        return;
      }
      if (event.metaKey || event.ctrlKey || event.altKey) return;
      const action: Record<string, (() => void) | undefined> = {
        a: map.approve,
        c: map.condition,
        backspace: map.remove,
        delete: map.remove,
        arrowleft: map.previous,
        arrowup: map.previous,
        arrowright: map.next,
        arrowdown: map.next,
        ' ': map.toggleMode,
        f: map.fold,
        i: map.itemCard,
        escape: map.escape,
      };
      const handler = action[key];
      if (!handler) return;
      // A focused button would also "click" on Space; the shortcut owns the key in the workspace.
      event.preventDefault();
      handler();
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [enabled]);
}
