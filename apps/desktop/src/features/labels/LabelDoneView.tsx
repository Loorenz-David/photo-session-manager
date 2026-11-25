import { Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import type { CheckItem, LabelCheck, LabelFile, LabelRunResult } from '@/services/labels';
import { LabelCard } from './LabelCard';
import {
  CLOSE,
  FAILED_STAY_SELECTED,
  NOT_FOUND_IN_PRICING,
  OPENING_IN_PREVIEW,
  PRINT_AT_100,
  SAVED_AND_OPENED,
  SAVED_NOT_OPENED,
  TRY_AGAIN,
  fullyGenerated,
  resultLines,
} from './labelWords';
import { canRetryDownload, displayItems, type DownloadState } from './useLabelDialog';

/** What happened to one file: being saved, saved and opened, saved but not opened (where it is), or the failure. */
function FileLine({ state, onRetry }: { state: DownloadState | undefined; onRetry: () => void }) {
  if (state?.kind === 'failed') {
    return (
      <div className="labels-error label-dialog-error" role="alert">
        <span>{state.failure.message}</span>
        {canRetryDownload(state.failure) && (
          <button type="button" className="batch-link" onClick={onRetry}>{TRY_AGAIN}</button>
        )}
      </div>
    );
  }
  if (state?.kind === 'saved') {
    return (
      <p className="label-saved">
        {state.saved.opened ? SAVED_AND_OPENED : SAVED_NOT_OPENED}
        <span className="label-saved-path">{state.saved.path}</span>
      </p>
    );
  }
  return (
    <p className="label-saved" role="status" aria-busy="true">
      <Loader2 className="animate-spin" aria-hidden /> {OPENING_IN_PREVIEW}
    </p>
  );
}

/** The card of a failed product: the check's item when there is one, else what the failure names. */
function failureItem(check: LabelCheck, failure: LabelRunResult['failures'][number]): CheckItem {
  const found = displayItems(check).find(item => item.product_id === failure.product_id);
  if (found) return found;
  return {
    product_id: failure.product_id ?? '', sku: failure.sku, title: failure.title, price_minor: null, status: 'not_printed',
    printed_at: null, outcome: 'fail', issues: [], notes: [], picture_path: null,
  };
}

/**
 * L4 (09C §14.2, MC-L2.17): the result when anything went wrong: the result lines (none whose number is zero), a card
 * per failed product with its message, the products Pricing did not find, that failed products stay selected (a batch
 * not fully generated), a line per file (saved and opened, saved but not opened with where it is, or the failure with
 * Try again when it may help), "Print at 100 % (not Fit to page)." when a file was saved, and Close. The list scrolls
 * inside the dialog; Close stays in view. A saved file's path is plain text (it is outside the data folder).
 */
export function LabelDoneView({
  check,
  result,
  single,
  downloads,
  onRetry,
  onClose,
}: {
  check: LabelCheck;
  result: LabelRunResult;
  single: boolean;
  downloads: Map<string, DownloadState>;
  onRetry: (file: LabelFile) => void;
  onClose: () => void;
}) {
  const files = result.artifacts;
  const anySaved = files.some(file => downloads.get(file.id)?.kind === 'saved');
  return (
    <div className="label-done">
      <div className="label-done-scroll" data-testid="label-done-scroll">
        <ul className="label-done-lines">
          {resultLines(result).map(line => (
            <li key={line.text}>{`${line.mark} ${line.text}`}</li>
          ))}
        </ul>
        {result.failures.length > 0 && (
          <ul className="label-cards" aria-label="Failed">
            {result.failures.map((failure, i) => (
              <LabelCard key={`${failure.product_id ?? ''}-${i}`} item={failureItem(check, failure)} failure={failure.message} />
            ))}
          </ul>
        )}
        {check.unresolved.length > 0 && (
          <ul className="label-unresolved" aria-label="Not found in Pricing">
            {check.unresolved.map((u, i) => (
              <li key={`${u.product_id ?? u.sku ?? ''}-${i}`}>{`${u.sku ?? u.product_id ?? ''} · ${NOT_FOUND_IN_PRICING}`}</li>
            ))}
          </ul>
        )}
        {!single && !fullyGenerated(result) && <p className="label-done-note">{FAILED_STAY_SELECTED}</p>}
        {files.length > 0 && (
          <ul className="label-files" aria-label="Label files">
            {files.map(file => (
              <li key={file.id} className="label-file" data-testid={`label-file-${file.id}`}>
                <FileLine state={downloads.get(file.id)} onRetry={() => onRetry(file)} />
              </li>
            ))}
          </ul>
        )}
        {anySaved && <p className="label-print-hint">{PRINT_AT_100}</p>}
      </div>
      <div className="label-dialog-buttons">
        <Button onClick={onClose}>{CLOSE}</Button>
      </div>
    </div>
  );
}
