import { useCallback, useEffect, useState } from 'react';

/** `localStorage` keys of the Tools panel (master plan §6.10). */
export const TOOLS_HIDDEN_KEY = 'tools.hidden';
export const TOOLS_SECTIONS_KEY = 'tools.sections';

export type ToolsSections = Record<string, boolean>;

function readHidden(): boolean {
  try {
    return window.localStorage.getItem(TOOLS_HIDDEN_KEY) === '1';
  } catch {
    return false;
  }
}

function readSections(): ToolsSections {
  try {
    const raw = window.localStorage.getItem(TOOLS_SECTIONS_KEY);
    const parsed: unknown = raw ? JSON.parse(raw) : null;
    if (!parsed || typeof parsed !== 'object') return {};
    return Object.fromEntries(Object.entries(parsed as Record<string, unknown>).filter(([, value]) => typeof value === 'boolean')) as ToolsSections;
  } catch {
    return {};
  }
}

function write(key: string, value: string | null) {
  try {
    if (value === null) window.localStorage.removeItem(key);
    else window.localStorage.setItem(key, value);
  } catch {
    // Without storage the panel simply forgets between launches.
  }
}

export type ToolsPanelState = {
  hidden: boolean;
  toggleHidden: () => void;
  /** A section is open unless it was folded (new sections open). */
  isOpen: (id: string) => boolean;
  toggleSection: (id: string) => void;
};

/**
 * The Tools panel's fold state (10D: "sections that fold open and closed … remembered across launches and hidden with
 * ⌘T"): `tools.hidden` (`'1'`) and `tools.sections` (JSON `{ composition: boolean }`) in `localStorage`.
 */
export function useToolsPanelState(): ToolsPanelState {
  const [hidden, setHidden] = useState(readHidden);
  const [sections, setSections] = useState(readSections);
  useEffect(() => write(TOOLS_HIDDEN_KEY, hidden ? '1' : null), [hidden]);
  useEffect(() => write(TOOLS_SECTIONS_KEY, JSON.stringify(sections)), [sections]);
  const toggleHidden = useCallback(() => setHidden(current => !current), []);
  const isOpen = useCallback((id: string) => sections[id] ?? true, [sections]);
  const toggleSection = useCallback((id: string) => setSections(current => ({ ...current, [id]: !(current[id] ?? true) })), []);
  return { hidden, toggleHidden, isOpen, toggleSection };
}
