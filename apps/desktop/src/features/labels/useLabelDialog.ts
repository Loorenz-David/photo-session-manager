import { useCallback, useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';
import { CONNECT_PRICING, connectionLine } from '@/features/conversations/connectionLine';
import { conversationsSettingsGet } from '@/services/conversations';
import {
  labelsCheck,
  labelsDownload,
  labelsGenerate,
  labelsPartial,
  labelsPreview,
  type CheckItem,
  type LabelCheck,
  type LabelFile,
  type LabelRunResult,
  type SavedLabel,
} from '@/services/labels';
import { fullyGenerated, generatedToast } from './labelWords';

/** A command failure as the wrappers reject it. */
export type LabelFailure = { code: string; message: string; data?: Record<string, string> };

/**
 * L3 (09C §6, MC-L2), L4 (09C §14.2, MC-L2.17): the dialog's states. `checking` → `check` → `generating` → `saving`
 * (every file of the result saved and opened, not closable) → closed with a toast, or `done` (the result view);
 * `error` when the check failed; a failed Generate goes back to `check` with its failure (Try again / Cancel).
 */
export type DialogState =
  | { phase: 'checking' }
  | { phase: 'error'; failure: LabelFailure; words: string }
  | { phase: 'check'; check: LabelCheck; failure: LabelFailure | null }
  | { phase: 'generating'; check: LabelCheck }
  | { phase: 'saving'; check: LabelCheck; result: LabelRunResult }
  | { phase: 'done'; check: LabelCheck; result: LabelRunResult };

/** The preview picture of one product: loading, its file, or the message shown in its place. */
export type PreviewState = { kind: 'loading' } | { kind: 'ready'; path: string } | { kind: 'failed'; message: string };

/** One file's save: in flight, failed (with Try again when it may help), saved (opened in Preview or not). */
export type DownloadState = { kind: 'busy' } | { kind: 'failed'; failure: LabelFailure } | { kind: 'saved'; saved: SavedLabel };

/** Generate failures that a press of Try again may cure (09C §6.2, §10). */
const RETRYABLE = new Set(['labels_unreachable', 'labels_renderer_unavailable']);
/** Download failures that a press of Try again may cure (09C §10: saving fails "with Try again"). */
const DOWNLOAD_RETRYABLE = new Set(['labels_save_failed', 'labels_unreachable']);

export function canRetryGenerate(failure: LabelFailure): boolean {
  return RETRYABLE.has(failure.code);
}

export function canRetryDownload(failure: LabelFailure): boolean {
  return DOWNLOAD_RETRYABLE.has(failure.code);
}

/** The check's items in group display order (‹ › step through them, projection r0 D15). */
export function displayItems(check: LabelCheck): CheckItem[] {
  return check.groups.flatMap(group => group.items);
}

/**
 * The dialog's logic: `labelsCheck` when opened, `labelsPreview` for the product shown, `labelsGenerate` on the button
 * (a second press while one is in flight does nothing), `labelsPartial` when the photographer leaves after a failed
 * Generate. Whenever a result arrives, `labelsDownload` saves and opens each of its files, in order, each at most once
 * automatically (MC-L2.17); everything generated → one toast and the dialog closes, otherwise the result view (Try
 * again per file whose save may succeed again). `onClose` gets the run's result when there is one.
 */
export function useLabelDialog(productIds: string[], onClose: (result: LabelRunResult | null) => void) {
  const [state, setState] = useState<DialogState>({ phase: 'checking' });
  const [shown, setShown] = useState(0);
  const [previews, setPreviews] = useState<Map<string, PreviewState>>(() => new Map());
  const [downloads, setDownloads] = useState<Map<string, DownloadState>>(() => new Map());
  /** The files' states as last set (read when the automatic saves have settled). */
  const files = useRef(new Map<string, DownloadState>());
  /** The files already asked automatically (by artifact id; never asked twice, MC-L2.17). */
  const savedOnce = useRef(new Set<string>());
  const inFlight = useRef(false);
  /** The products whose preview was asked (once each). */
  const asked = useRef(new Set<string>());
  const closed = useRef(false);
  const closeRef = useRef(onClose);
  closeRef.current = onClose;
  const idsKey = productIds.join('\u0000');

  const close = useCallback((result: LabelRunResult | null) => {
    if (closed.current) return;
    closed.current = true;
    closeRef.current(result);
  }, []);

  // Opening: one check of the products.
  useEffect(() => {
    let alive = true;
    labelsCheck({ product_ids: idsKey.split('\u0000') }).then(
      check => alive && setState({ phase: 'check', check, failure: null }),
      (failure: LabelFailure) => {
        if (!alive) return;
        if (failure.code !== 'labels_off') {
          setState({ phase: 'error', failure, words: failure.message });
          return;
        }
        // Projection r0 D17: 09B's connection words, as the ⋯ hint shows them.
        setState({ phase: 'error', failure, words: CONNECT_PRICING });
        if (failure.data?.reason === 'no_key') return;
        try {
          conversationsSettingsGet().then(
            settings => alive && setState({ phase: 'error', failure, words: connectionLine(settings.connection) ?? CONNECT_PRICING }),
            () => undefined,
          );
        } catch {
          // The words stay CONNECT_PRICING.
        }
      },
    );
    return () => {
      alive = false;
    };
  }, [idsKey]);

  const check = state.phase === 'check' || state.phase === 'generating' || state.phase === 'saving' || state.phase === 'done' ? state.check : null;
  const items = check ? displayItems(check) : [];
  const shownItem: CheckItem | null = items[Math.min(shown, Math.max(items.length - 1, 0))] ?? null;
  const checkId = check?.id ?? null;
  const shownId = shownItem?.product_id ?? null;

  // The preview of the product shown (asked once per product).
  useEffect(() => {
    if (checkId === null || shownId === null || asked.current.has(shownId)) return;
    asked.current.add(shownId);
    setPreviews(previous => new Map(previous).set(shownId, { kind: 'loading' }));
    labelsPreview({ check_id: checkId, product_id: shownId }).then(
      answer => setPreviews(previous => new Map(previous).set(shownId, { kind: 'ready', path: answer.path })),
      (failure: LabelFailure) => setPreviews(previous => new Map(previous).set(shownId, { kind: 'failed', message: failure.message })),
    );
  }, [checkId, shownId]);

  function generate() {
    if (inFlight.current || (state.phase !== 'check')) return;
    const current = state.check;
    inFlight.current = true;
    setState({ phase: 'generating', check: current });
    labelsGenerate({ check_id: current.id })
      .then(
        result => void deliver(current, result),
        (failure: LabelFailure) => setState({ phase: 'check', check: current, failure }),
      )
      .finally(() => {
        inFlight.current = false;
      });
  }

  function setFile(id: string, file: DownloadState) {
    files.current.set(id, file);
    setDownloads(new Map(files.current));
  }

  /** Saves and opens one file of the kept result. */
  async function save(current: LabelCheck, file: LabelFile) {
    setFile(file.id, { kind: 'busy' });
    try {
      const saved = await labelsDownload({ check_id: current.id, artifact_id: file.id });
      setFile(file.id, { kind: 'saved', saved });
    } catch (failure) {
      setFile(file.id, { kind: 'failed', failure: failure as LabelFailure });
    }
  }

  /**
   * MC-L2.17: a result arrived. Each of its files not asked yet is saved and opened, one after the other, while the
   * dialog cannot be closed; then everything generated (nothing failed or left, none not found, every file opened) →
   * one toast and the dialog closes with the result; otherwise the result view.
   */
  async function deliver(current: LabelCheck, result: LabelRunResult) {
    setState({ phase: 'saving', check: current, result });
    for (const file of result.artifacts) {
      if (savedOnce.current.has(file.id)) continue;
      savedOnce.current.add(file.id);
      await save(current, file);
    }
    const opened = result.artifacts.every(file => {
      const saved = files.current.get(file.id);
      return saved?.kind === 'saved' && saved.saved.opened;
    });
    if (fullyGenerated(result) && current.unresolved.length === 0 && opened) {
      toast.success(generatedToast(result));
      close(result);
      return;
    }
    setState({ phase: 'done', check: current, result });
  }

  /** Leaving: closes, except after a failed Generate, where the result so far is saved and shown (MC-L2.10, D16). */
  function leave() {
    if (state.phase === 'generating' || state.phase === 'saving') return;
    if (state.phase === 'done') {
      close(state.result);
      return;
    }
    if (state.phase === 'check' && state.failure) {
      const current = state.check;
      labelsPartial({ check_id: current.id }).then(
        result => void deliver(current, result),
        () => close(null),
      );
      return;
    }
    close(null);
  }

  /** Try again for a file whose save failed with a code that may succeed again; the dialog stays open. */
  function retry(file: LabelFile) {
    if (state.phase !== 'done') return;
    const current = files.current.get(file.id);
    if (current?.kind !== 'failed' || !canRetryDownload(current.failure)) return;
    void save(state.check, file);
  }

  return {
    state,
    items,
    shown: Math.min(shown, Math.max(items.length - 1, 0)),
    shownItem,
    preview: shownId === null ? null : previews.get(shownId) ?? { kind: 'loading' as const },
    step: (by: number) => setShown(index => Math.max(0, Math.min(items.length - 1, index + by))),
    /** L4 (09C §14.1): shows a product's preview (a click on its card). */
    show: (productId: string) => {
      const index = items.findIndex(item => item.product_id === productId);
      if (index >= 0) setShown(index);
    },
    generate,
    leave,
    close,
    retry,
    downloads,
  };
}
