import { useCallback, useEffect, useRef, useState } from 'react';
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

/** A command failure as the wrappers reject it. */
export type LabelFailure = { code: string; message: string; data?: Record<string, string> };

/**
 * L3 (09C §6, MC-L2): the dialog's states. `checking` → `check` → `generating` → `done`; `error` when the check
 * failed; a failed Generate goes back to `check` with its failure (Try again / Cancel).
 */
export type DialogState =
  | { phase: 'checking' }
  | { phase: 'error'; failure: LabelFailure; words: string }
  | { phase: 'check'; check: LabelCheck; failure: LabelFailure | null }
  | { phase: 'generating'; check: LabelCheck }
  | { phase: 'done'; check: LabelCheck; result: LabelRunResult };

/** The preview picture of one product: loading, its file, or the message shown in its place. */
export type PreviewState = { kind: 'loading' } | { kind: 'ready'; path: string } | { kind: 'failed'; message: string };

/** One file row's download: in flight, failed (with Try again when it may help), saved without Preview. */
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
 * Generate, `labelsDownload` per file. `onClose` gets the run's result when there is one.
 */
export function useLabelDialog(productIds: string[], onClose: (result: LabelRunResult | null) => void) {
  const [state, setState] = useState<DialogState>({ phase: 'checking' });
  const [shown, setShown] = useState(0);
  const [previews, setPreviews] = useState<Map<string, PreviewState>>(() => new Map());
  const [downloads, setDownloads] = useState<Map<string, DownloadState>>(() => new Map());
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

  const check = state.phase === 'check' || state.phase === 'generating' || state.phase === 'done' ? state.check : null;
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
        result => setState({ phase: 'done', check: current, result }),
        (failure: LabelFailure) => setState({ phase: 'check', check: current, failure }),
      )
      .finally(() => {
        inFlight.current = false;
      });
  }

  /** Leaving: closes, except after a failed Generate, where the result so far is shown (MC-L2.10, D16). */
  function leave() {
    if (state.phase === 'generating') return;
    if (state.phase === 'done') {
      close(state.result);
      return;
    }
    if (state.phase === 'check' && state.failure) {
      const current = state.check;
      labelsPartial({ check_id: current.id }).then(
        result => setState({ phase: 'done', check: current, result }),
        () => close(null),
      );
      return;
    }
    close(null);
  }

  function download(file: LabelFile, single: boolean) {
    if (state.phase !== 'done' || downloads.get(file.id)?.kind === 'busy') return;
    const { check: current, result } = state;
    setDownloads(previous => new Map(previous).set(file.id, { kind: 'busy' }));
    labelsDownload({ check_id: current.id, artifact_id: file.id }).then(
      saved => {
        if (single && saved.opened) {
          close(result);
          return;
        }
        setDownloads(previous => new Map(previous).set(file.id, { kind: 'saved', saved }));
      },
      (failure: LabelFailure) => setDownloads(previous => new Map(previous).set(file.id, { kind: 'failed', failure })),
    );
  }

  return {
    state,
    items,
    shown: Math.min(shown, Math.max(items.length - 1, 0)),
    shownItem,
    preview: shownId === null ? null : previews.get(shownId) ?? { kind: 'loading' as const },
    step: (by: number) => setShown(index => Math.max(0, Math.min(items.length - 1, index + by))),
    generate,
    leave,
    close,
    download,
    downloads,
  };
}
