import { useAction } from '@/lib/pending';

type Props = { count: number; onBack: () => Promise<unknown>; canShiftBack?: boolean; onShiftBack?: () => Promise<unknown> };

const none = async () => undefined;

/**
 * 10G §6.2: one quiet line while photos differ from the template, with Back to template (one undoable step).
 * 10G §6.4: while a denied photo leaves a gap before later template-placed photos, the same line offers Shift the rest
 * back (shown also when nothing differs; gone when there is nothing to shift).
 */
export function TemplateWarning({ count, onBack, canShiftBack = false, onShiftBack = none }: Props) {
  const back = useAction(onBack);
  const shift = useAction(onShiftBack);
  if (count <= 0 && !canShiftBack) return null;
  return (
    <div className="template-warning">
      {count > 0 && (
        <span role="status" className="template-warning-text">
          <span>{count} {count === 1 ? 'photo differs' : 'photos differ'} from the template</span>
          <button type="button" className="template-back-button" onClick={() => void back.run()} aria-disabled={back.busy || undefined} aria-busy={back.busy ? 'true' : undefined}>
            Back to template
          </button>
        </span>
      )}
      {canShiftBack && (
        <button type="button" className="template-back-button template-shift-button" onClick={() => void shift.run()} aria-disabled={shift.busy || undefined} aria-busy={shift.busy ? 'true' : undefined}>
          Shift the rest back
        </button>
      )}
    </div>
  );
}
