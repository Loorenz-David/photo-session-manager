import { useEffect, useState } from 'react';
import { DropdownMenu } from 'radix-ui';
import { CONNECT_PRICING, connectionLine } from '@/features/conversations/connectionLine';
import { conversationsSettingsGet } from '@/services/conversations';
import type { LabelsAccess, SessionLabel } from '@/services/labels';
import { DENIED_HINT, GENERATE_LABEL, PUBLISH_FIRST } from './labelWords';

/**
 * L1 (09C §6 "⋯ menu entry", projection r0 D5, D6): "Generate label" in a session's ⋯ menu, shown in L1 only where it is
 * off and says why. First match: no answer yet → nothing; no shop product → off, "Publish first…" (in every access
 * state); connection off → off with 09B's words for it; key denied → off with the denied words; access unknown, or it
 * would be on → nothing (Generate arrives in L3). Rendered inside the open menu, so the connection is read when it opens.
 */
export function LabelMenuItem({ label, access }: { label?: SessionLabel | null; access?: LabelsAccess | null }) {
  const hint = useHint(label, access);
  if (!hint) return null;
  return (
    <DropdownMenu.Item className="photo-menu-item label-menu-item" disabled>
      <span className="label-menu-text">
        <span>{GENERATE_LABEL}</span>
        <small className="label-menu-hint">{hint}</small>
      </span>
    </DropdownMenu.Item>
  );
}

function useHint(label?: SessionLabel | null, access?: LabelsAccess | null): string | null {
  const readsConnection = !!label && label.product_id !== null && access?.state === 'off' && access.reason !== 'no_key';
  const [offWords, setOffWords] = useState<string | null>(null);
  useEffect(() => {
    if (!readsConnection) return;
    let alive = true;
    conversationsSettingsGet()
      .then(settings => alive && setOffWords(connectionLine(settings.connection)), () => alive && setOffWords(null))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [readsConnection]);
  if (!label || !access) return null;
  if (label.product_id === null) return PUBLISH_FIRST;
  if (access.state === 'off') return access.reason === 'no_key' ? CONNECT_PRICING : (offWords ?? CONNECT_PRICING);
  if (access.state === 'denied') return DENIED_HINT;
  return null;
}
