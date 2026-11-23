import { ChevronLeft, ChevronRight, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { previewUrl } from '@/services/sessions';
import type { CheckGroup, CheckItem, LabelCheck } from '@/services/labels';
import {
  CANCEL,
  CANT_GENERATE,
  CLOSE,
  GENERATE_PDF,
  NO_TEMPLATE,
  NOT_FOUND_IN_PRICING,
  REPRINT_PDF,
  TRY_AGAIN,
  checkSentences,
  formatPrice,
  outcomeWords,
  sizeText,
} from './labelWords';
import { canRetryGenerate, type LabelFailure, type PreviewState } from './useLabelDialog';

/** A group's heading: `<template name> · <product types> · <size>` (types left out when none); `No template`. */
export function groupHeading(group: CheckGroup): string {
  const template = group.template;
  if (!template) return NO_TEMPLATE;
  const parts = [template.name];
  if (group.product_types.length > 0) parts.push(group.product_types.join(', '));
  parts.push(sizeText(template.width_mm, template.height_mm));
  return parts.join(' · ');
}

function CheckLine({ item }: { item: CheckItem }) {
  const failing = item.outcome === 'fail';
  return (
    <li className="label-line" data-testid={`label-line-${item.product_id}`} data-outcome={item.outcome}>
      <div className="label-line-main">
        <span className="label-line-sku">{item.sku ?? item.product_id}</span>
        <span className="label-line-title">{item.title ?? ''}</span>
        <span className="label-line-price">{formatPrice(item.price_minor)}</span>
        <span className={failing ? 'label-line-outcome label-line-fail' : 'label-line-outcome'}>{failing ? CANT_GENERATE : outcomeWords(item.outcome)}</span>
      </div>
      {failing && item.issues.map(issue => <p key={issue} className="label-line-issue">{issue}</p>)}
      {item.notes.map(note => <p key={note} className="label-line-note">{note}</p>)}
    </li>
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
 * L3 (09C §6.1, §6.2): the check: one section per template group with a line per product, the products Pricing did not
 * find, the footer and notes in Pricing's wording, the preview of the product shown, and Generate PDF (Reprint PDF for
 * one printed product) with Cancel. While generating both are disabled; after a failed Generate its message is shown with
 * Try again (when it may help) and Cancel, or only Close.
 */
export function LabelCheckView({
  check,
  single,
  generating,
  failure,
  items,
  shown,
  preview,
  onStep,
  onGenerate,
  onLeave,
}: {
  check: LabelCheck;
  /** The dialog was opened with one product id (projection r0 D11). */
  single: boolean;
  generating: boolean;
  failure: LabelFailure | null;
  items: CheckItem[];
  shown: number;
  preview: PreviewState | null;
  onStep: (by: number) => void;
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
      <div className="label-check-list">
        {check.groups.map(group => (
          <section key={group.template?.id ?? '-'} className="label-group" aria-label={groupHeading(group)}>
            <h3 className="label-group-heading">{groupHeading(group)}</h3>
            <ul className="label-lines">
              {group.items.map(item => <CheckLine key={item.product_id} item={item} />)}
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
        <div className="label-footer">
          {sentences.map(sentence => <p key={sentence}>{sentence}</p>)}
        </div>
      </div>
      <PreviewPane
        item={shownItem}
        template={check.groups.find(group => shownItem !== null && group.items.includes(shownItem))?.template ?? null}
        preview={preview}
        index={shown}
        count={items.length}
        onStep={onStep}
      />
      {failure && (
        <div className="labels-error label-dialog-error" role="alert">
          <span>{failure.message}</span>
        </div>
      )}
      <div className="label-dialog-buttons">
        {generating && (
          <span className="label-busy" role="status" aria-busy="true">
            <Loader2 className="animate-spin" aria-hidden /> Generating…
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
  );
}
