import { Download } from 'lucide-react';
import { Button } from '@/components/ui/button';
import type { LabelFile, LabelRunResult } from '@/services/labels';
import {
  CLOSE,
  DONE,
  DOWNLOAD_PDF,
  FAILED_STAY_SELECTED,
  PRINT_AT_100,
  TRY_AGAIN,
  fullyGenerated,
  labelCount,
  resultLines,
  sizeText,
} from './labelWords';
import { canRetryDownload, type DownloadState } from './useLabelDialog';

/** What a download row says after its press: the failure (with Try again when it may help) or where it was saved. */
function DownloadNote({ state, onRetry }: { state: DownloadState | undefined; onRetry: () => void }) {
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
      <p className="label-saved" role="status">
        {state.saved.opened ? 'Saved and opened in Preview: ' : "Saved, but Preview couldn't open it: "}
        <span className="label-saved-path">{state.saved.path}</span>
      </p>
    );
  }
  return null;
}

/**
 * L3 (09C §6.3–§6.5): the done view: the title, the result lines (none whose number is zero), each failure with its SKU ·
 * name and message, the files (one: a single Download PDF; several: a row per file with its own, plus Done; none: Close),
 * "Print at 100 % (not Fit to page).", and for a batch with failures that they stay selected. The saved file's path is
 * shown as text (it is outside the data folder).
 */
export function LabelDoneView({
  result,
  single,
  downloads,
  onDownload,
  onClose,
}: {
  result: LabelRunResult;
  single: boolean;
  downloads: Map<string, DownloadState>;
  onDownload: (file: LabelFile, single: boolean) => void;
  onClose: () => void;
}) {
  const files = result.artifacts;
  const one = files.length === 1;
  return (
    <div className="label-done">
      <ul className="label-done-lines">
        {resultLines(result).map(line => (
          <li key={line.text}>{`${line.mark} ${line.text}`}</li>
        ))}
      </ul>
      {result.failures.length > 0 && (
        <ul className="label-failures" aria-label="Failed">
          {result.failures.map((failure, i) => (
            <li key={`${failure.product_id ?? ''}-${i}`}>
              <strong>{`${failure.sku ?? failure.product_id ?? ''} · ${failure.title ?? ''}`}</strong>
              <p className="label-line-issue">{failure.message}</p>
            </li>
          ))}
        </ul>
      )}
      {!single && !fullyGenerated(result) && <p className="label-done-note">{FAILED_STAY_SELECTED}</p>}
      {files.length > 1 && (
        <ul className="label-files" aria-label="Label files">
          {files.map(file => (
            <li key={file.id} className="label-file" data-testid={`label-file-${file.id}`}>
              <span className="label-file-name">{file.filename}</span>
              <span>{labelCount(file.page_count)}</span>
              <span>{sizeText(file.width_mm, file.height_mm)}</span>
              <Button variant="outline" size="sm" disabled={downloads.get(file.id)?.kind === 'busy'} onClick={() => onDownload(file, false)}>
                <Download /> {DOWNLOAD_PDF}
              </Button>
              <DownloadNote state={downloads.get(file.id)} onRetry={() => onDownload(file, false)} />
            </li>
          ))}
        </ul>
      )}
      {one && <DownloadNote state={downloads.get(files[0].id)} onRetry={() => onDownload(files[0], true)} />}
      {files.length > 0 && <p className="label-print-hint">{PRINT_AT_100}</p>}
      <div className="label-dialog-buttons">
        {one ? (
          <>
            {downloads.get(files[0].id)?.kind === 'saved' && <Button variant="outline" onClick={onClose}>{CLOSE}</Button>}
            <Button disabled={downloads.get(files[0].id)?.kind === 'busy'} onClick={() => onDownload(files[0], true)}>
              <Download /> {DOWNLOAD_PDF}
            </Button>
          </>
        ) : files.length > 1 ? (
          <Button onClick={onClose}>{DONE}</Button>
        ) : (
          <Button onClick={onClose}>{CLOSE}</Button>
        )}
      </div>
    </div>
  );
}
