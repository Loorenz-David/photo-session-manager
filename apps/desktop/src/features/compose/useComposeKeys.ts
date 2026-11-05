import { useEffect, useRef } from 'react';
import { isTyping } from '@/lib/keyboard';
import { refit, type RefitChange } from './refit';
import { LAYER_OPEN, type ComposeController } from './useComposeGesture';

type KeysOptions = {
  controller: ComposeController;
  /** ⌘T. */
  onToggleTools: () => void;
  /** ⌘⌥R. */
  onReset: () => void;
  /** `[` / `]`: the previous / next photo of the display sequence, keeping the changes. */
  onStepPhoto: (delta: -1 | 1) => void;
  /** S: Normal ⇄ Split (P33). */
  onToggleKind: () => void;
};

/** 1–6 and 0 pick Original, 1:1, 4:5, 3:2, 2:3, 16:9, Free (10D). */
const SHAPE_KEYS: Record<string, string> = { '1': 'original', '2': '1:1', '3': '4:5', '4': '3:2', '5': '2:3', '6': '16:9', '0': 'free' };
const ARROWS: Record<string, [number, number]> = { arrowleft: [-1, 0], arrowright: [1, 0], arrowup: [0, -1], arrowdown: [0, 1] };

/**
 * The editor's keys (10D §What the photographer sees, bullet 5; plan P32 task 6, P33 task 2): arrows nudge one screen
 * pixel (Shift ×10), `.` centres, 1–6 / 0 pick the shape, X turns the frame, S toggles Split, `[` `]` step photos, ⌘⌥R
 * resets, ⌘T toggles the panel;
 * Esc and Return stay in P31's binding. Every other key while composing is swallowed (presses inside a field belong to
 * the field; an open menu or dialog owns its own keys). The listener runs in the capture phase so the workspace's and
 * the app's own key maps — ⌘[ Back among them — never see a swallowed key.
 */
export function useComposeKeys(options: KeysOptions): void {
  const latest = useRef(options);
  latest.current = options;
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.defaultPrevented || document.querySelector(LAYER_OPEN) || isTyping(event.target)) return;
      const { controller, onToggleTools, onReset, onStepPhoto, onToggleKind } = latest.current;
      const key = event.key.toLowerCase();
      const swallow = () => {
        event.preventDefault();
        event.stopPropagation();
      };
      if (event.metaKey || event.ctrlKey) {
        if (event.metaKey && !event.altKey && key === 't') {
          swallow();
          onToggleTools();
        } else if (event.metaKey && event.altKey && (key === 'r' || event.code === 'KeyR')) {
          swallow();
          onReset();
        } else if (key === '[') {
          swallow(); // ⌘[ Back is a page key; the editor owns the keyboard while composing.
        }
        return;
      }
      // Esc and Return are P31's; Tab keeps moving the focus between the panel's controls.
      if (event.altKey || key === 'escape' || key === 'enter' || key === 'tab') return;
      const state = controller.getSnapshot();
      const set = (change: RefitChange) => {
        if (state) controller.dispatch({ type: 'SetValues', values: refit(state.values, change, state.frame) });
      };
      const arrow = ARROWS[key];
      if (arrow) {
        // The frame moves over the photo in the arrow's direction (plan Notes), which is the photo moving the other way.
        const step = event.shiftKey ? 10 : 1;
        controller.dispatch({ type: 'Nudge', dx: -arrow[0] * step, dy: -arrow[1] * step });
      } else if (key === '.') set({ kind: 'centre' });
      else if (key in SHAPE_KEYS) set({ kind: 'shape', shape: SHAPE_KEYS[key] });
      else if (key === 'x' && state) set({ kind: 'orientation', orientation: state.values.orientation === 'turned' ? 'named' : 'turned' });
      else if (key === 's') onToggleKind();
      else if (key === '[') onStepPhoto(-1);
      else if (key === ']') onStepPhoto(1);
      swallow();
    };
    window.addEventListener('keydown', onKey, true);
    return () => window.removeEventListener('keydown', onKey, true);
  }, []);
}
