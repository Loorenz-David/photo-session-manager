import { useSyncExternalStore } from 'react';
import type { CompositionValues, Frame } from '@/services/compositions';
import { refit } from './refit';

let copied: CompositionValues | null = null;
const listeners = new Set<() => void>();
export function copyComposition(values: CompositionValues): void {
  copied = structuredClone(values);
  listeners.forEach(listener => listener());
}
export function clipboardComposition(): CompositionValues | null { return copied; }
export function useClipboardComposition(): CompositionValues | null {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => { listeners.delete(listener); }; }, clipboardComposition);
}
export function refitClipboard(values: CompositionValues, frame: Frame): CompositionValues {
  return refit(values, { kind: 'reframe' }, frame);
}
