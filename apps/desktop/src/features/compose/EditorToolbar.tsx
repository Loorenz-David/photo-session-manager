import { ClipboardPaste, Copy, Magnet, Undo2, ZoomIn } from 'lucide-react';
import { Hint, type HintCard } from '@/components/ui/tooltip';

type Props = {
  onRevert: () => void;
  loupe: boolean; onLoupe: () => void;
  snap: boolean; onSnap: () => void;
  onCopy: () => void; onPaste: () => void; canPaste: boolean;
};

export function EditorToolbar(props: Props) {
  const button = (card: HintCard, icon: React.ReactNode, action: () => void, pressed?: boolean, unavailable?: boolean) =>
    <Hint key={card.name} text={card}><button type="button" className="compose-tool" aria-label={card.name} aria-pressed={pressed} data-on={pressed || undefined} aria-disabled={unavailable || undefined} onClick={() => { if (!unavailable) action(); }}>{icon}</button></Hint>;
  return <div className="editor-toolbar">
    {button({ name: 'Revert', key: 'Esc', description: 'Back to how this photo was when the editor opened' }, <Undo2 aria-hidden />, props.onRevert)}
    {button({ name: 'Loupe', key: 'Z', description: '100 % view under the pointer' }, <ZoomIn aria-hidden fill={props.loupe ? 'currentColor' : 'none'} />, props.onLoupe, props.loupe)}
    {button({ name: 'Snapping', key: '⌥', description: 'Snaps to the centre and the image edges; hold ⌥ to suspend' }, <Magnet aria-hidden fill={props.snap ? 'currentColor' : 'none'} />, props.onSnap, props.snap)}
    {button({ name: 'Copy framing', key: '⌥⌘C', description: "Takes this photo's composition" }, <Copy aria-hidden />, props.onCopy)}
    {button({ name: 'Paste framing', key: '⌥⌘V', description: 'Puts the copied composition on this photo, refitted to its frame' }, <ClipboardPaste aria-hidden />, props.onPaste, undefined, !props.canPaste)}
  </div>;
}
