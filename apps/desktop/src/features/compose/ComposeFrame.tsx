import { useId, type ReactNode } from 'react';
import type { Box } from './geometry';
import type { SnapGuides } from './snapping';

type ComposeFrameProps = { width: number; height: number; frameBox: Box; kind?: string; guides?: SnapGuides; children?: ReactNode };

/**
 * The frame over the stage: everything outside it dimmed through an SVG mask, a hairline border, for a Split (P33) a
 * 1 px hairline at the frame's exact middle (10D: "whose exact middle makes two square files"), and the handles (P32) as
 * children.
 */
export function ComposeFrame({ width, height, frameBox, kind = 'normal', guides, children }: ComposeFrameProps) {
  const maskId = `compose-mask-${useId().replace(/[^a-zA-Z0-9]/g, '')}`;
  const middle = frameBox.left + frameBox.width / 2;
  return (
    <svg className="compose-overlay" viewBox={`0 0 ${width} ${height}`} width={width} height={height} aria-hidden="true">
      <defs>
        <mask id={maskId}>
          <rect width={width} height={height} fill="white" />
          <rect x={frameBox.left} y={frameBox.top} width={frameBox.width} height={frameBox.height} fill="black" />
        </mask>
      </defs>
      <rect className="compose-dim" width={width} height={height} mask={`url(#${maskId})`} />
      <rect className="compose-frame" data-kind={kind} x={frameBox.left} y={frameBox.top} width={frameBox.width} height={frameBox.height} />
      {kind === 'split' && <line className="compose-middle" x1={middle} y1={frameBox.top} x2={middle} y2={frameBox.top + frameBox.height} />}
      {guides?.x && <line className="compose-guide" data-axis="x" x1={width / 2} x2={width / 2} y1={0} y2={height} />}
      {guides?.y && <line className="compose-guide" data-axis="y" x1={0} x2={width} y1={height / 2} y2={height / 2} />}
      {children}
    </svg>
  );
}
