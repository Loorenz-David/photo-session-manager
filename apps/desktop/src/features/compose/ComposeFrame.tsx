import { useId, type ReactNode } from 'react';
import type { Box } from './geometry';
import type { SnapGuides } from './snapping';
import { overlayLines } from './overlays';
import { STRAIGHTEN_GRID_PX, type OverlayKind } from './useComposePrefs';

type ComposeFrameProps = { width: number; height: number; frameBox: Box; kind?: string; guides?: SnapGuides; overlay?: OverlayKind; straightenGrid?: boolean; children?: ReactNode };

/**
 * The frame over the stage: everything outside it dimmed through an SVG mask, a hairline border, for a Split (P33) a
 * 1 px hairline at the frame's exact middle (10D: "whose exact middle makes two square files"), and the handles (P32) as
 * children.
 */
export function ComposeFrame({ width, height, frameBox, kind = 'normal', guides, overlay = 'thirds', straightenGrid = false, children }: ComposeFrameProps) {
  const maskId = `compose-mask-${useId().replace(/[^a-zA-Z0-9]/g, '')}`;
  const gridId = `compose-grid-${useId().replace(/[^a-zA-Z0-9]/g, '')}`;
  const middle = frameBox.left + frameBox.width / 2;
  const guideX = guides?.x === 'centre' ? middle : guides?.x === 'start' ? frameBox.left : frameBox.left + frameBox.width;
  const guideY = guides?.y === 'centre' ? frameBox.top + frameBox.height/2 : guides?.y === 'start' ? frameBox.top : frameBox.top + frameBox.height;
  return (
    <svg className="compose-overlay" viewBox={`0 0 ${width} ${height}`} width={width} height={height} aria-hidden="true">
      <defs>
        <mask id={maskId}>
          <rect width={width} height={height} fill="white" />
          <rect x={frameBox.left} y={frameBox.top} width={frameBox.width} height={frameBox.height} fill="black" />
        </mask>
        {straightenGrid && <pattern id={gridId} patternUnits="userSpaceOnUse" width={STRAIGHTEN_GRID_PX} height={STRAIGHTEN_GRID_PX}><path d={`M ${STRAIGHTEN_GRID_PX} 0 L 0 0 0 ${STRAIGHTEN_GRID_PX}`} /></pattern>}
      </defs>
      <rect className="compose-dim" width={width} height={height} mask={`url(#${maskId})`} />
      <rect className="compose-frame" data-kind={kind} x={frameBox.left} y={frameBox.top} width={frameBox.width} height={frameBox.height} />
      <g className="compose-overlay-guides" data-overlay={overlay}>{overlayLines(overlay, frameBox, kind === 'split').map((line, index) => <line key={index} {...line} />)}</g>
      {straightenGrid && <rect className="compose-straighten-grid" width={width} height={height} fill={`url(#${gridId})`} />}
      {kind === 'split' && <line className="compose-middle" x1={middle} y1={frameBox.top} x2={middle} y2={frameBox.top + frameBox.height} />}
      {guides?.x && <line className="compose-guide" data-axis="x" x1={guideX} x2={guideX} y1={0} y2={height} />}
      {guides?.y && <line className="compose-guide" data-axis="y" x1={0} x2={width} y1={guideY} y2={guideY} />}
      {children}
    </svg>
  );
}
