import { useCallback, useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';

const message = (error: unknown) => error instanceof Error ? error.message : String(error);

/** Keeps the lock in a ref so two clicks in one render cannot both start an action. */
export function useAction<T, A extends unknown[]>(fn: (...args: A) => Promise<T>, onError: (error: unknown) => void = error => toast.error(message(error))) {
  const locked = useRef(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const run = useCallback(async (...args: A): Promise<T | undefined> => {
    if (locked.current) return undefined;
    locked.current = true;
    setBusy(true);
    setError(null);
    try {
      return await fn(...args);
    } catch (failure) {
      setError(failure);
      onError(failure);
      return undefined;
    } finally {
      locked.current = false;
      setBusy(false);
    }
  }, [fn, onError]);
  return { run, busy, error };
}

/** First load owns the placeholder; later loads retain the last visible result. */
export function usePageData<T>(loader: () => Promise<T>, deps: readonly unknown[] = []) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<unknown>(null);
  const ticket = useRef(0);
  const loaderRef = useRef(loader);
  loaderRef.current = loader;
  const reload = useCallback(async () => {
    const current = ++ticket.current;
    try {
      const next = await loaderRef.current();
      if (current === ticket.current) {
        setData(next);
        setLoading(false);
        setError(null);
      }
    } catch (failure) {
      if (current === ticket.current) {
        setError(failure);
        setLoading(false);
      }
    }
  }, []);
  useEffect(() => {
    void reload();
    return () => { ticket.current++; };
    // The caller names the dependencies that require a fresh load.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [reload, ...deps]);
  return { data, loading, error, reload };
}
