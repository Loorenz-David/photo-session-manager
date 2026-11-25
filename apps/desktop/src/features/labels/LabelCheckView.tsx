import { ChevronLeft, ChevronRight, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { previewUrl } from '@/services/sessions';
import type { CheckGroup, CheckItem, LabelCheck } from '@/services/labels';
import { LabelCard } from './LabelCard';
import {
  CANCEL,
  CLOSE,
  GENERATE_PDF,
  NOT_FOUND_IN_PRICING,
  REPRINT_PDF,
  TRY_AGAIN,
  checkSentences,
  groupLines,
} from './labelWords';
import { canRetryGenerate, type LabelFailure, type PreviewState } from './useLabelDialog';

/** 09C §14.1: a group's header, two lines (`groupLines`), each on one line with its full text as the tooltip. */
function GroupHeader({ group }: { group: CheckGroup }) {
  const { name, detail } = groupLines(group);
  return (
    <header className="label-group-header">
      <h3 className="label-group-heading" title={name}>{name}</h3>
      {detail && <p className="label-group-detail" title={detail}>{detail}</p>}
    </header>
  );
}

/** The preview pane: a placeholder of the label's proportions while loading, the picture on white, or the message. */
function PreviewPane({ item, template, preview, index, count, onStep }: {
  item: CheckItem | null;
  /** The template of the product shown (its group's). */
  template: CheckGroup['template'];
  preview: PreviewState | null;
  index: number;
  count: number;
  onStep: (by: number) => void;
}) {
  const ratio = template?.width_mm && template.height_mm ? `${template.width_mm} / ${template.height_mm}` : '1 / 1';
  return (
    <div className="label-preview-pane">
      {preview?.kind === 'ready' ? (
        <div className="label-preview">
          <img alt="Label preview" src={previewUrl(preview.path)} />
        </div>
      ) : preview?.kind === 'failed' ? (
        <p className="label-preview-message" role="status">{preview.message}</p>
      ) : (
        <div className="label-preview label-preview-placeholder" data-testid="label-preview-placeholder" data-ratio={ratio} style={{ aspectRatio: ratio }} aria-busy="true" />
      )}
      {count > 1 && (
        <div className="label-preview-steps">
          <Button variant="ghost" size="icon-sm" aria-label="‹" disabled={index === 0} onClick={() => onStep(-1)}>
            <ChevronLeft />
          </Button>
          <span className="label-preview-name">{item ? item.sku ?? item.product_id : ''}</span>
          <Button variant="ghost" size="icon-sm" aria-label="›" disabled={index >= count - 1} onClick={() => onStep(1)}>
            <ChevronRight />
          </Button>
        </div>
      )}
    </div>
  );
}

/**
 * L3 (09C §6.1, §6.2), L4 (09C §14.1): the check. A scrolling list (one section per template group with a two-line
 * header and a card per product, then the products Pricing did not find) with the preview of the product shown beside
 * it, and a footer outside the list, always in view: the sentences in Pricing's wording, a failed Generate's message,
 * and Generate PDF (Reprint PDF for one printed product) with Cancel. While generating both are disabled; after a failed
 * Generate its message is shown with Try again (when it may help) and Cancel, or only Close. Clicking a card shows it.
 */
export function LabelCheckView({
  check,
  single,
  generating,
  busyWords,
  failure,
  items,
  shown,
  preview,
  onStep,
  onShow,
  onGenerate,
  onLeave,
}: {
  check: LabelCheck;
  /** The dialog was opened with one product id (projection r0 D11). */
  single: boolean;
  generating: boolean;
  /** The busy line while generating (`Generating…` when none). */
  busyWords?: string;
  failure: LabelFailure | null;
  items: CheckItem[];
  shown: number;
  preview: PreviewState | null;
  onStep: (by: number) => void;
  /** Shows a product's preview (a click on its card). */
  onShow: (productId: string) => void;
  onGenerate: () => void;
  onLeave: () => void;
}) {
  const singleItem = single ? items[0] ?? null : null;
  const shownItem = items[shown] ?? null;
  const sentences = checkSentences(check.summary, singleItem, Date.now());
  const reprint = singleItem?.outcome === 'reprint';
  const nothingReady = check.summary.ready === 0;
  return (
    <div className="label-check">
      <div className="label-check-scroll" data-testid="label-check-scroll">
        {check.groups.map(group => (
          <section key={group.template?.id ?? '-'} className="label-group" aria-label={groupLines(group).name}>
            <GroupHeader group={group} />
            <ul className="label-cards">
              {group.items.map(item => (
                <LabelCard
                  key={item.product_id}
                  item={item}
                  selected={shownItem?.product_id === item.product_id}
                  onSelect={() => onShow(item.product_id)}
                />
              ))}
            </ul>
          </section>
        ))}
        {check.unresolved.length > 0 && (
          <ul className="label-unresolved" aria-label="Not found in Pricing">
            {check.unresolved.map((u, i) => (
              <li key={`${u.product_id ?? u.sku ?? ''}-${i}`}>{`${u.sku ?? u.product_id ?? ''} · ${NOT_FOUND_IN_PRICING}`}</li>
            ))}
          </ul>
        )}
      </div>
      <PreviewPane
        item={shownItem}
        template={check.groups.find(group => shownItem !== null && group.items.includes(shownItem))?.template ?? null}
        preview={preview}
        index={shown}
        count={items.length}
        onStep={onStep}
      />
      <div className="label-check-footer" data-testid="label-check-footer">
        <div className="label-footer">
          {sentences.map(sentence => <p key={sentence}>{sentence}</p>)}
        </div>
        {failure && (
          <div className="labels-error label-dialog-error" role="alert">
            <span>{failure.message}</span>
          </div>
        )}
        <div className="label-dialog-buttons">
          {generating && (
            <span className="label-busy" role="status" aria-busy="true">
              <Loader2 className="animate-spin" aria-hidden /> {busyWords ?? 'Generating…'}
            </span>
          )}
          {failure && !canRetryGenerate(failure) ? (
            <Button variant="outline" onClick={onLeave}>{CLOSE}</Button>
          ) : (
            <>
              <Button variant="outline" disabled={generating} onClick={onLeave}>{CANCEL}</Button>
              <Button disabled={generating || nothingReady} onClick={onGenerate}>
                {failure ? TRY_AGAIN : reprint ? REPRINT_PDF : GENERATE_PDF}
              </Button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
