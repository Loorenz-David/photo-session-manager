/**
 * Output numbering in the grid (10D MC-29 §Outputs: "a `normal` photo has one output; a `split` photo has two … Positions
 * stay per photo; outputs expand in photo order"; 10D: "In the grid a split photo shows … the two places it fills
 * (03–04)"). A display rule only: positions and drops stay per photo (`reorderPhoto(photoId, slot)` as before).
 */

/** The fields of a photo the numbering reads. */
export type Numberable = {
  id: number;
  classification: 'unreviewed' | 'main' | 'condition' | 'deleted';
  composition: { kind: string };
};

/** The display numbers a photo fills in its group: `first === last` for one output, `first + 1 === last` for a split. */
export type OutputNumbers = { first: number; last: number };

/**
 * The places a photo takes in its group: two for a split Main or Condition photo, one otherwise (an unreviewed photo
 * is not an output and counts one place, as before 10D).
 */
export function outputCount(photo: Numberable): number {
  return photo.classification !== 'unreviewed' && photo.composition.kind === 'split' ? 2 : 1;
}

/** Display numbers per photo of one group (Main with its unreviewed photos after it, or Condition), in the given order. */
export function outputNumbers<P extends Numberable>(group: readonly P[]): Map<number, OutputNumbers> {
  const numbers = new Map<number, OutputNumbers>();
  let next = 1;
  for (const photo of group) {
    const count = outputCount(photo);
    numbers.set(photo.id, { first: next, last: next + count - 1 });
    next += count;
  }
  return numbers;
}

/** The outputs in a group (the `n` of "Main 3–4 of n"). */
export function outputTotal(group: readonly Numberable[]): number {
  return group.reduce((sum, photo) => sum + outputCount(photo), 0);
}

const pad = (n: number) => String(n).padStart(2, '0');

/** The tile badge: `"03"` for one output, `"03–04"` (en dash) for the two places a split fills. */
export function outputLabel(first: number, last: number): string {
  return first === last ? pad(first) : `${pad(first)}–${pad(last)}`;
}
