import { useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';
import { ActionButton } from '@/components/ui/action-button';
import { SkeletonRows } from '@/components/pending/SkeletonRows';
import { FormDialog, type FormPosition } from '@/features/form/FormDialog';
import { getForm, readShopValues, type Form, type FormRead } from '@/services/attributes';
import { getSessionView, type SessionView } from '@/services/photos';
import { PublishPopover } from './PublishPopover';

const filledText = (form: Form) => `${form.groups.reduce((n, group) => n + group.filled, 0)}/${form.groups.reduce((n, group) => n + group.total, 0)} filled`;

export function BatchQueue({ ids, onClose }: { ids: number[]; onClose: () => void }) {
  const [index, setIndex] = useState(0);
  const [confirmed, setConfirmed] = useState<number[]>([]);
  const [views, setViews] = useState<Map<number, SessionView>>(new Map());
  const [reviewOpen, setReviewOpen] = useState(false);
  const [counts, setCounts] = useState<Map<number, string>>(new Map());
  // P67 (changelog 46 (3)): the linked products of the whole batch are read once, together; each form waits for its own.
  const [reads, setReads] = useState<Map<number, Promise<FormRead | undefined>>>(new Map());
  // P67 (changelog 46 (6)): the step and scroll of the form the photographer was on; the next form opens there.
  const position = useRef<FormPosition | null>(null);
  useEffect(() => {
    let live = true;
    Promise.all(ids.map(async id => {
      const view = await getSessionView(id);
      const form = await getForm(view.item!.id);
      return { view, form };
    })).then(found => {
      if (!live) return;
      setViews(new Map(found.map(({ view }) => [view.session.id, view])));
      setCounts(new Map(found.map(({ view, form }) => [view.session.id, filledText(form)])));
      const linked = found.map(({ view }) => view.item!).filter(item => item.origin === 'linked');
      if (linked.length === 0) return;
      const batch = readShopValues(linked.map(item => item.id));
      setReads(new Map(linked.map(item => [item.id, batch.then(results => results.find(result => result.item_id === item.id))])));
      void batch.then(() => Promise.all(found.map(async ({ view }) => [view.session.id, filledText(await getForm(view.item!.id))] as const)))
        .then(updated => live && setCounts(new Map(updated)))
        .catch(() => undefined);
    }).catch(error => toast.error(`Could not open queue: ${(error as Error).message}`));
    return () => { live = false; };
  }, [ids]);
  const view = views.get(ids[index]);
  const item = view?.item;
  const linked = item?.origin === 'linked';
  // A linked item's form waits for the batch read; it never starts a read of its own while the batch's is coming.
  const remoteRead = item && linked ? reads.get(item.id) : undefined;
  return <>
    {(!item || (linked && !remoteRead)) && <div className="p-6"><SkeletonRows n={4} /></div>}
    {item && view && (!linked || remoteRead) && <FormDialog key={ids[index]} open onOpenChange={open => !open && onClose()} item={item} photos={view.photos} onItemChanged={() => void getSessionView(ids[index]).then(updated => setViews(old => new Map(old).set(ids[index], updated)))}
      remoteRead={remoteRead}
      position={position.current}
      onPosition={next => { position.current = next; }}
      queueStrip={<nav aria-label="Publish queue" className="flex gap-2 border-b px-5 py-2">{ids.map((id, position) => <button key={id} type="button" onClick={() => setIndex(position)} aria-current={position === index}>{confirmed.includes(id) ? '✓ ' : ''}{views.get(id)?.item?.title ?? `Session ${id}`} · {counts.get(id) ?? '…'}</button>)}</nav>}
      queueConfirm={() => {
        setConfirmed(old => old.includes(ids[index]) ? old : [...old, ids[index]]);
        void getForm(item.id).then(form => setCounts(old => new Map(old).set(ids[index], filledText(form))));
        setIndex(old => Math.min(old + 1, ids.length - 1));
      }}
      publishSlot={<>
        <ActionButton disabled={confirmed.length === 0} action={async () => { setReviewOpen(true); }}>Publish {confirmed.length} confirmed</ActionButton>
        {/* Inside the form dialog's tree, so the review is the top modal layer (not inert behind the form). */}
        {reviewOpen && <PublishPopover modal sessionIds={confirmed} onClose={() => setReviewOpen(false)} onConfirmed={onClose} />}
      </>}
    />}
  </>;
}
