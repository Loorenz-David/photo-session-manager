import { useAction } from '@/lib/pending';

type Props = { count: number; onBack: () => Promise<unknown> };

/** 10G §6.2: one quiet line while photos differ from the template, with Back to template (one undoable step). */
export function TemplateWarning({ count, onBack }: Props) {
  const back = useAction(onBack);
  if (count <= 0) return null;
  return (
    <div role="status" className="template-warning">
      <span>{count} {count === 1 ? 'photo differs' : 'photos differ'} from the template</span>
      <button type="button" className="template-back-button" onClick={() => void back.run()} aria-disabled={back.busy || undefined} aria-busy={back.busy ? 'true' : undefined}>
        Back to template
      </button>
    </div>
  );
}
