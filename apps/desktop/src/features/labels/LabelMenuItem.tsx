import { useEffect, useState } from 'react';
import { DropdownMenu } from 'radix-ui';
import { CONNECT_PRICING, connectionLine } from '@/features/conversations/connectionLine';
import { conversationsSettingsGet } from '@/services/conversations';
import type { LabelsAccess, SessionLabel } from '@/services/labels';
import { DENIED_HINT, GENERATE_LABEL, PUBLISH_FIRST, REPRINT_LABEL } from './labelWords';

/**
 * L1 (09C §6 "⋯ menu entry", projection r0 D5, D6): "Generate label" in a session's ⋯ menu. First match: no answer yet →
 * nothing; no shop product → off, "Publish first…" (in every access state); connection off → off with 09B's words for
 * it; key denied → off with the denied words. Rendered inside the open menu, so the connection is read when it opens.
 * L3 (09C §6; ACC.1, projection r0 D6): with a product and the access `allowed` or `unknown` the item is on — "Reprint
 * label" when the marker is printed, else "Generate label" — and a press asks `onGenerate` for that one product.
 */
export function LabelMenuItem({ label, access, onGenerate }: {
  label?: SessionLabel | null;
  access?: LabelsAccess | null;
  /** L3: opens the label dialog for the product. */
  onGenerate?: (productId: string) => void;
}) {
  const hint = useHint(label, access);
  const productId = label?.product_id ?? null;
  const on = !!label && productId !== null && (access?.state === 'allowed' || access?.state === 'unknown');
  if (on && onGenerate) {
    return (
      <DropdownMenu.Item className="photo-menu-item label-menu-item" onSelect={() => onGenerate(productId)}>
        <span>{label.marker?.status === 'printed' ? REPRINT_LABEL : GENERATE_LABEL}</span>
      </DropdownMenu.Item>
    );
  }
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
