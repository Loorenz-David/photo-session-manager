import type { CSSProperties, ReactNode } from 'react';
import type { CompositionValues, Frame } from '@/services/compositions';
import { composedLayout } from './composed';

type Props = { values: CompositionValues; frame: Frame; className: string; children: ReactNode };

export function ComposedPicture({ values, frame, className, children }: Props) {
  const layout = composedLayout(values, frame);
  const style = { aspectRatio: layout.ratio, '--composed-ratio': String(layout.ratioValue) } as CSSProperties;
  return <div className={className} style={style}>
    <div className="composed-frame" style={{ ...layout.frame, transform: layout.rotate }}>{children}</div>
    {values.kind === 'split' && <span className="composed-split" style={{ left: '50%' }} aria-hidden />}
  </div>;
}
