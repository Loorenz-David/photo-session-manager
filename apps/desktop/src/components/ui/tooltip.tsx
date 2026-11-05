import type { ReactElement } from 'react';
import { Tooltip } from 'radix-ui';

export const HINT_DELAY_MS = 400;
export type HintCard = { name: string; key?: string; description: string };

export function Hint({ text, children }: { text: string | HintCard; children: ReactElement }) {
  return <Tooltip.Provider delayDuration={HINT_DELAY_MS}>
    <Tooltip.Root>
      <Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
      <Tooltip.Portal><Tooltip.Content className="hint-tooltip" sideOffset={5}>{typeof text === 'string' ? text : <span className="hint-card"><strong className="hint-name">{text.name}</strong>{text.key && <> {'·'} <kbd className="hint-key">{text.key}</kbd></>} {'—'} <span className="hint-description">{text.description}</span></span>}<Tooltip.Arrow /></Tooltip.Content></Tooltip.Portal>
    </Tooltip.Root>
  </Tooltip.Provider>;
}
