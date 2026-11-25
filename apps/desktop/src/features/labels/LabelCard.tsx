import { useState, type KeyboardEvent } from 'react';
import { ExternalLink } from '@/components/ExternalLink';
import { Thumb } from '@/features/sessions/HoverPreview';
import type { CheckItem } from '@/services/labels';
import { CANT_GENERATE, NOT_PUBLISHED_YET, NO_PRICE_PILL, QR_LINK, cantGenerate, formatPrice, outcomeWords } from './labelWords';

type Tone = 'amber' | 'red' | 'grey';

/**
 * L4 (09C §14.1): one product of the check as a card. Left: its picture in a 1:1 square (the one already on this Mac,
 * else the neutral placeholder) with the SKU (else the product id) under it. Right: the title on one line (the full
 * title as its tooltip), then the price and the outcome in words. Under them, only when there is something to say, the
 * warnings row: `No price` (amber) · one red `Can't generate: <issue>` per issue · `Not published yet` (amber) per
 * not-published note, with the `QR link` pill button when the note names an address · each other note as a grey pill.
 * `QR link` shows or hides the address under the pills (it opens nothing itself; the address opens in the browser when
 * clicked). Clicking the card selects it (`aria-current="true"` when selected).
 */
export function LabelCard({ item, selected = false, onSelect }: { item: CheckItem; selected?: boolean; onSelect?: () => void }) {
  const [shownLink, setShownLink] = useState<string | null>(null);
  const failing = item.outcome === 'fail';
  const title = item.title ?? '';
  const pills: { key: string; text: string; tone: Tone; link?: string }[] = [];
  if (item.outcome === 'no_price' || item.price_minor == null) pills.push({ key: 'no-price', text: NO_PRICE_PILL, tone: 'amber' });
  item.issues.forEach((issue, i) => pills.push({ key: `issue-${i}`, text: cantGenerate(issue), tone: 'red' }));
  item.notes.forEach((note, i) => {
    if (note.kind === 'not_published') pills.push({ key: `note-${i}`, text: NOT_PUBLISHED_YET, tone: 'amber', link: note.link ?? undefined });
    else pills.push({ key: `note-${i}`, text: note.message, tone: 'grey' });
  });
  const onKeyDown = (event: KeyboardEvent<HTMLLIElement>) => {
    if (event.target !== event.currentTarget || !onSelect) return;
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onSelect();
    }
  };
  return (
    <li
      className="label-card"
      data-testid={`label-card-${item.product_id}`}
      data-outcome={item.outcome}
      aria-current={selected ? 'true' : undefined}
      tabIndex={onSelect ? 0 : undefined}
      onClick={onSelect}
      onKeyDown={onKeyDown}
    >
      <div className="label-card-side">
        <Thumb path={item.picture_path} className="label-card-thumb" />
        <span className="label-card-sku">{item.sku ?? item.product_id}</span>
      </div>
      <div className="label-card-body">
        <span className="label-card-title" title={title}>{title}</span>
        <span className="label-card-line">
          <span className="label-card-price">{formatPrice(item.price_minor)}</span>
          <span className={failing ? 'label-card-outcome label-card-fail' : 'label-card-outcome'}>{failing ? CANT_GENERATE : outcomeWords(item.outcome)}</span>
        </span>
        {pills.length > 0 && (
          <div className="label-card-warnings">
            {pills.map(pill => (
              <span key={pill.key} className="label-card-pill-group">
                <span className="pill label-card-pill" data-tone={pill.tone}>{pill.text}</span>
                {pill.link && (
                  <button
                    type="button"
                    className="pill label-card-pill label-card-qr"
                    data-tone="link"
                    aria-expanded={shownLink === pill.link}
                    onClick={event => {
                      event.stopPropagation();
                      const link = pill.link ?? null;
                      setShownLink(current => (current === link ? null : link));
                    }}
                  >
                    {QR_LINK}
                  </button>
                )}
              </span>
            ))}
          </div>
        )}
        {shownLink && (
          <p className="label-card-link" onClick={event => event.stopPropagation()}>
            <ExternalLink href={shownLink} />
          </p>
        )}
      </div>
    </li>
  );
}
