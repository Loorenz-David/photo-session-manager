/** The fields of a photo the display rule reads. */
export type Orderable = {
  id: number;
  classification: 'unreviewed' | 'main' | 'condition' | 'deleted';
  position: number | null;
  captured_at: number | null;
};

export type DisplayOrder<P extends Orderable> = { main: P[]; unreviewed: P[]; condition: P[] };

function byPosition(a: Orderable, b: Orderable): number {
  return (a.position ?? Number.MAX_SAFE_INTEGER) - (b.position ?? Number.MAX_SAFE_INTEGER) || a.id - b.id;
}

function byCapture(a: Orderable, b: Orderable): number {
  return (a.captured_at ?? Number.MAX_SAFE_INTEGER) - (b.captured_at ?? Number.MAX_SAFE_INTEGER) || a.id - b.id;
}

/**
 * MC-5 display rule (ratify R-I2): Main by position; unreviewed photos after the last Main photo, by `captured_at`
 * then id (MC-4 tie rule); Condition by position. Deleted photos are not shown.
 */
export function displayOrder<P extends Orderable>(photos: readonly P[]): DisplayOrder<P> {
  return {
    main: photos.filter(photo => photo.classification === 'main').sort(byPosition),
    unreviewed: photos.filter(photo => photo.classification === 'unreviewed').sort(byCapture),
    condition: photos.filter(photo => photo.classification === 'condition').sort(byPosition),
  };
}

/** The whole display sequence: Main, then unreviewed, then Condition (stage navigation, folded strip). */
export function displaySequence<P extends Orderable>(photos: readonly P[]): P[] {
  const order = displayOrder(photos);
  return [...order.main, ...order.unreviewed, ...order.condition];
}
