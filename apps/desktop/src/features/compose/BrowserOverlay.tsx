import { useEffect, useRef, useState } from 'react';
import { pickAndImport } from '@/features/workspace/ImportDropZone';
import type { TileHandlers } from '@/features/workspace/PhotoGrid';
import { PhotoPanel } from '@/features/workspace/PhotoPanel';
import type { Photo } from '@/services/photos';

/** The pointer this close to the window's right edge reveals the browser. */
export const EDGE_REVEAL_PX = 12;

type BrowserOverlayProps = Omit<TileHandlers, 'onOpen'> & {
  photos: readonly Photo[];
  /** The machine's phase: the browser never opens during a gesture. */
  isIdle: () => boolean;
  /** A tile click moves the editor to that photo, keeping the changes. */
  onPick: (photoId: number) => void;
  sessionsOpen: boolean;
  onToggleSessions: () => void;
};

/**
 * The photo browser while composing (10D): hidden, it slides in over the image when the pointer reaches the
 * window's right edge and hides again when the pointer leaves it — never during a drag.
 */
export function BrowserOverlay({ photos, isIdle, onPick, sessionsOpen, onToggleSessions, ...tiles }: BrowserOverlayProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    const workspace: EventTarget = element.closest('main.workspace') ?? window;
    const onMove = (event: Event) => {
      if ((event as PointerEvent).clientX >= window.innerWidth - EDGE_REVEAL_PX && isIdle()) setOpen(true);
    };
    const onLeave = () => setOpen(false);
    workspace.addEventListener('pointermove', onMove);
    element.addEventListener('pointerleave', onLeave);
    return () => {
      workspace.removeEventListener('pointermove', onMove);
      element.removeEventListener('pointerleave', onLeave);
    };
  }, [isIdle]);

  return (
    <div ref={ref} className="compose-browser" data-open={open ? 'true' : 'false'}>
      <PhotoPanel photos={photos} onFold={() => undefined} onImport={() => pickAndImport()} onOpen={onPick} sessionsOpen={sessionsOpen} onToggleSessions={onToggleSessions} {...tiles} />
    </div>
  );
}
