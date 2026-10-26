import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { FormDialog } from '@/features/form/FormDialog';
import { getForm } from '@/services/attributes';
import { getSessionView, type SessionView } from '@/services/photos';
import { PublishPopover } from './PublishPopover';

export function BatchQueue({ ids, onClose }: { ids: number[]; onClose: () => void }) {
  const [index, setIndex] = useState(0);
  const [confirmed, setConfirmed] = useState<number[]>([]);
  const [views, setViews] = useState<Map<number, SessionView>>(new Map());
  const [reviewOpen, setReviewOpen] = useState(false);
  const [counts, setCounts] = useState<Map<number, string>>(new Map());
  useEffect(() => {
    let live = true;
    Promise.all(ids.map(async id => {
      const view = await getSessionView(id);
      const form = await getForm(view.item!.id);
      return { view, form };
    })).then(found => {
      if (live) {
        setViews(new Map(found.map(({ view }) => [view.session.id, view])));
        setCounts(new Map(found.map(({ view, form }) => [view.session.id, `${form.groups.reduce((n, group) => n + group.filled, 0)}/${form.groups.reduce((n, group) => n + group.total, 0)} filled`])));
      }
    }).catch(error => toast.error(`Could not open queue: ${(error as Error).message}`));
    return () => { live = false; };
  }, [ids]);
  const view = views.get(ids[index]);
  const item = view?.item;
  return <>
    {item && view && <FormDialog key={ids[index]} open onOpenChange={open => !open && onClose()} item={item} photos={view.photos} onItemChanged={() => void getSessionView(ids[index]).then(updated => setViews(old => new Map(old).set(ids[index], updated)))}
      queueStrip={<nav aria-label="Publish queue" className="flex gap-2 border-b px-5 py-2">{ids.map((id, position) => <button key={id} type="button" onClick={() => setIndex(position)} aria-current={position === index}>{confirmed.includes(id) ? '✓ ' : ''}{views.get(id)?.item?.title ?? `Session ${id}`} · {counts.get(id) ?? 'Loading'}</button>)}</nav>}
      queueConfirm={() => {
        setConfirmed(old => old.includes(ids[index]) ? old : [...old, ids[index]]);
        void getForm(item.id).then(form => setCounts(old => new Map(old).set(ids[index], `${form.groups.reduce((n, group) => n + group.filled, 0)}/${form.groups.reduce((n, group) => n + group.total, 0)} filled`)));
        setIndex(old => Math.min(old + 1, ids.length - 1));
      }}
      publishSlot={<Button disabled={confirmed.length === 0} onClick={() => setReviewOpen(true)}>Publish {confirmed.length} confirmed</Button>}
    />}
    {reviewOpen && <PublishPopover sessionIds={confirmed} onClose={() => setReviewOpen(false)} onConfirmed={onClose} />}
  </>;
}
