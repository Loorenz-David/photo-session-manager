import { cn } from '@/lib/utils';
import type { Group } from '@/services/attributes';
import type { Photo } from '@/services/photos';
import { Thumb } from '@/features/workspace/PhotoTile';
import { displayOrder } from '@/features/workspace/order';

function Grid({ title, photos, large, coverLabel }: { title: string; photos: Photo[]; large: boolean; coverLabel: string }) {
  return (
    <section className="flex flex-col gap-2">
      <h4 className="flex justify-between text-[11px] font-extrabold tracking-widest uppercase">
        {title} <span className="font-normal tracking-normal text-muted-foreground normal-case">{photos.length} {photos.length === 1 ? 'photo' : 'photos'}</span>
      </h4>
      {photos.length === 0 ? (
        <p className="text-xs text-muted-foreground">None yet.</p>
      ) : (
        <div className={cn('grid gap-1.5', large ? 'grid-cols-3' : 'grid-cols-5')}>
          {photos.map((photo, index) => (
            <div
              key={photo.id}
              className={cn('relative aspect-square overflow-hidden rounded-md bg-muted [&_img]:size-full [&_img]:object-cover', large && index === 0 && 'col-span-2 row-span-2')}
            >
              <Thumb photo={photo} />
              {large && index === 0 && <span className="absolute top-1.5 left-1.5 rounded bg-background px-1.5 py-0.5 text-[10px] font-semibold">{coverLabel}</span>}
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

/**
 * The photos that will be uploaded (NOTES §Screens 6): on Details and Product the main images with the cover 2×2
 * (unreviewed photos follow Main, as they publish in Main), condition photos smaller below; on Condition the
 * other way round. Thumbnails: adjusted preview, else the camera original.
 */
export function MediaColumn({ photos, step }: { photos: Photo[]; step: Group }) {
  const order = displayOrder(photos);
  const main = [...order.main, ...order.unreviewed];
  const condition = order.condition;
  return (
    <div className="flex w-[300px] shrink-0 flex-col gap-5 overflow-y-auto border-r p-4">
      {step === 'condition' ? (
        <>
          <Grid title="Condition photos" photos={condition} large coverLabel="01" />
          <Grid title="Main images" photos={main} large={false} coverLabel="Cover" />
        </>
      ) : (
        <>
          <Grid title="Media" photos={main} large coverLabel="Cover" />
          <Grid title="Condition photos" photos={condition} large={false} coverLabel="01" />
        </>
      )}
    </div>
  );
}
