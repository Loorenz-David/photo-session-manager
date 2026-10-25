import { Check } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Thumb } from '@/features/workspace/PhotoTile';
import type { FieldInputProps } from '../MetafieldInput';

/**
 * `files` (the damage report, 07A MC-14): the session's condition photos in condition order, all selected by
 * default. The value stores exclusions, `{"excluded": [photo ids]}`, so a condition photo added later is selected.
 */
export function FilesPicker({ label, value, onChange, conditionPhotos = [], selected }: FieldInputProps) {
  const ids = conditionPhotos.map(photo => photo.id);
  const draft = value as { excluded?: number[] } | undefined;
  const chosen = new Set(draft?.excluded ? ids.filter(id => !draft.excluded!.includes(id)) : (selected ?? ids));

  function toggle(photoId: number) {
    const next = new Set(chosen);
    if (next.has(photoId)) next.delete(photoId);
    else next.add(photoId);
    onChange({ excluded: ids.filter(id => !next.has(id)) });
  }

  if (conditionPhotos.length === 0) {
    return <p className="text-xs text-muted-foreground">No condition photos yet. Photos marked Condition appear here, all selected.</p>;
  }
  return (
    <div role="group" aria-label={label} className="grid grid-cols-5 gap-2">
      {conditionPhotos.map((photo, index) => {
        const on = chosen.has(photo.id);
        return (
          <button
            key={photo.id}
            type="button"
            role="checkbox"
            aria-checked={on}
            aria-label={`Condition photo ${String(index + 1).padStart(2, '0')}`}
            data-photo-id={photo.id}
            className={cn('relative aspect-square overflow-hidden rounded-md border-2 [&_img]:size-full [&_img]:object-cover', on ? 'border-foreground' : 'border-transparent opacity-50')}
            onClick={() => toggle(photo.id)}
          >
            <Thumb photo={photo} />
            <span className={cn('absolute top-1 right-1 grid size-4 place-content-center rounded-sm border', on ? 'bg-foreground text-background' : 'bg-background')} aria-hidden>
              {on && <Check className="size-3" />}
            </span>
          </button>
        );
      })}
    </div>
  );
}
