import { type CSSProperties, type ReactNode, useEffect, useState } from 'react';
import { type PhoneImage, previewUrl } from '../api';

/** A preview that failed to load is asked for again this long after, this many times (plan LV6 point P3). */
const RETRY_MS = 2000;
const RETRIES = 3;

type Props = {
  token: string;
  image: PhoneImage;
  /** The picture's classes and inline style (the placeholder has its own look). */
  className: string;
  style?: CSSProperties;
  testId?: string;
  /** What the placeholder says, if anything (a dock or drawer tile shows its number tag beside it). */
  children?: ReactNode;
};

/**
 * A photo's preview, never a broken image (master_plan_live §5 PH6a, finding F7). The Mac is asked
 * only once it says the preview exists (`image.preview` is a string); until then, and whenever the
 * picture fails to load, a plain placeholder shows instead. A failed load is tried again three
 * times, two seconds apart; a new `preview` value or `revision` starts over.
 */
export function PhotoImage({ token, image, className, style, testId, children }: Props) {
  const key = `${image.photo_id}:${image.revision}:${image.preview}`;
  const [load, setLoad] = useState({ key, attempt: 0, failed: false });
  // What the Mac serves changed: whatever happened to the earlier picture no longer counts.
  const now = load.key === key ? load : { key, attempt: 0, failed: false };
  const waiting = now.failed && now.attempt < RETRIES;
  useEffect(() => {
    if (!waiting) return undefined;
    const timer = setTimeout(() => setLoad({ key, attempt: now.attempt + 1, failed: false }), RETRY_MS);
    return () => clearTimeout(timer);
  }, [waiting, key, now.attempt]);

  const src = previewUrl(token, image, now.attempt);
  if (src === null || now.failed) return <div className="ph-photo-wait" data-testid="photo-wait">{children}</div>;
  return <img className={className} style={style} data-testid={testId} alt="" draggable={false} src={src}
    onError={() => setLoad({ key, attempt: now.attempt, failed: true })} />;
}
