import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';
import { clearAttribute, getForm, setAttributeValue, type Form, type FormField } from '@/services/attributes';
import { kindOf } from './kinds';

/** Quiet time after the last keystroke before a field is written (delegated choice; ch08 "values save as typed"). */
export const SAVE_DEBOUNCE_MS = 300;

export type SaveState = 'idle' | 'saving' | 'saved' | 'error';

/** No value: nothing typed, an empty list, or null. Emptying a field clears it instead of storing "". */
export function isEmptyValue(value: unknown): boolean {
  return value === null || value === undefined || (typeof value === 'string' && value.trim() === '') || (Array.isArray(value) && value.length === 0);
}

/** The stored value of a field in its core form, or undefined when there is none or it is invalid (MC-9). */
export function storedValue(field: FormField): unknown {
  const value = field.value;
  if (!value || !value.valid) return undefined;
  try {
    return JSON.parse(value.value);
  } catch {
    return undefined;
  }
}

/**
 * The form of one item: `form_get`, local edits shown at once, each field written 300 ms after its last edit
 * (`attribute_values_set`), and the whole form re-fetched when a field loses focus (counts, provenance).
 */
export function useFormState(itemId: number) {
  const [form, setForm] = useState<Form | null>(null);
  const [drafts, setDrafts] = useState<Record<string, unknown>>({});
  const [saveState, setSaveState] = useState<SaveState>('idle');
  const timers = useRef(new Map<string, number>());
  const pending = useRef(new Map<string, unknown>());
  const inFlight = useRef(new Set<Promise<unknown>>());
  const alive = useRef(true);

  const reload = useCallback(async () => {
    try {
      const next = await getForm(itemId);
      if (!alive.current) return;
      setForm(next);
      // Drafts already written are now in the form; only edits still waiting keep their draft.
      setDrafts(current => Object.fromEntries(Object.entries(current).filter(([key]) => pending.current.has(key))));
    } catch (error) {
      toast.error(`Could not load the product details: ${(error as Error).message}`);
    }
  }, [itemId]);

  useEffect(() => {
    alive.current = true;
    void reload();
    return () => {
      alive.current = false;
    };
  }, [reload]);

  const write = useCallback(
    (key: string) => {
      const handle = timers.current.get(key);
      if (handle !== undefined) window.clearTimeout(handle);
      timers.current.delete(key);
      if (!pending.current.has(key)) return Promise.resolve();
      const value = pending.current.get(key);
      pending.current.delete(key);
      setSaveState('saving');
      const call = (isEmptyValue(value) ? clearAttribute(itemId, key) : setAttributeValue(itemId, key, value))
        .then(() => alive.current && setSaveState('saved'))
        .catch(error => {
          if (alive.current) setSaveState('error');
          toast.error(`Could not save: ${(error as Error).message}`);
        });
      inFlight.current.add(call);
      void call.finally(() => inFlight.current.delete(call));
      return call;
    },
    [itemId],
  );

  /** A human edit: shown at once, written after the debounce. */
  const edit = useCallback(
    (key: string, value: unknown) => {
      setDrafts(current => ({ ...current, [key]: value }));
      pending.current.set(key, value);
      const handle = timers.current.get(key);
      if (handle !== undefined) window.clearTimeout(handle);
      timers.current.set(
        key,
        window.setTimeout(() => void write(key), SAVE_DEBOUNCE_MS),
      );
    },
    [write],
  );

  /** Writes every waiting edit now and waits for the writes in flight. */
  const flush = useCallback(async () => {
    await Promise.all([...pending.current.keys()].map(key => write(key)));
    await Promise.all([...inFlight.current]);
  }, [write]);

  /** A field lost focus: write it now, then re-fetch the form. */
  const blur = useCallback(async () => {
    await flush();
    await reload();
  }, [flush, reload]);

  // Closing the form never loses an edit: whatever is still waiting is written at once.
  useEffect(() => {
    const waiting = pending.current;
    return () => {
      for (const key of [...waiting.keys()]) void write(key);
    };
  }, [write]);

  const fields = useMemo(() => form?.groups.flatMap(group => group.fields) ?? [], [form]);

  /** What the field shows: the local edit, else the stored value. */
  const valueOf = useCallback(
    (field: FormField) => {
      if (field.definition.key in drafts) return drafts[field.definition.key];
      if (field.value && !field.value.valid && ['text', 'textarea', 'url', 'int', 'decimal'].includes(kindOf(field.definition))) {
        try {
          const parsed: unknown = JSON.parse(field.value.value);
          return typeof parsed === 'string' ? parsed : field.value.value;
        } catch {
          return field.value.value;
        }
      }
      return storedValue(field);
    },
    [drafts],
  );
  const isEdited = useCallback((key: string) => key in drafts, [drafts]);

  return { form, fields, saveState, edit, flush, blur, reload, valueOf, isEdited };
}

export type FormState = ReturnType<typeof useFormState>;
