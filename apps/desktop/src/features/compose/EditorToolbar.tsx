import { Hint } from '@/components/ui/tooltip';

type Props = {
  onRevert: () => void;
  loupe: boolean; onLoupe: () => void;
  snap: boolean; onSnap: () => void;
  onCopy: () => void; onPaste: () => void; canPaste: boolean;
  tools: boolean; onTools: () => void;
  meter: boolean; onMeter: () => void;
};

export function EditorToolbar(props: Props) {
  const button = (name: string, hint: string, action: () => void, pressed?: boolean, unavailable?: boolean) =>
    <Hint key={name} text={hint}><button type="button" className="compose-tool" aria-label={name} aria-pressed={pressed} aria-disabled={unavailable || undefined} onClick={() => { if (!unavailable) action(); }}>{name}</button></Hint>;
  return <div className="editor-toolbar">
    {button('Revert', 'Revert (Esc)', props.onRevert)}
    {button('Loupe', 'Loupe (hold Z)', props.onLoupe, props.loupe)}
    {button('Snapping', 'Snapping (hold ⌥ to suspend)', props.onSnap, props.snap)}
    {button('Copy framing', 'Copy framing (⌥⌘C)', props.onCopy)}
    {button('Paste framing', 'Paste framing (⌥⌘V)', props.onPaste, undefined, !props.canPaste)}
    {button('Tools panel', 'Tools panel (⌘T)', props.onTools, props.tools)}
    {button('Frame meter', 'Frame meter (⌥⌘P)', props.onMeter, props.meter)}
  </div>;
}
