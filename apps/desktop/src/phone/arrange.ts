/**
 * Sort and arrange on the phone (book 10J LV-D20; master_plan_live §5 PH9): the pure rules the
 * Preview's swipe and the strips' drag and drop stand on. The Mac decides every placement (PH4a);
 * these only say what to ask it and what to show meanwhile.
 */
import type { PhoneImage, PhoneSession } from './api';
import { DOCK } from './gestures';
import { photoIds, type PreviewSubject } from './state';

export type SwipeDirection = 'left' | 'right' | 'up';
/** What a counted swipe does: a request, the confirmation first (`main_confirm`), or nothing. */
export type SwipeAction = 'retake' | 'delete' | 'main' | 'main_confirm' | 'condition' | 'none';

/**
 * PH9.1: left denies (a template shot's photo by Retake, so its shot is empty and current), right
 * makes Main (a template shot's photo asks first; an extra already is), up sends to Condition (a
 * Condition photo already is).
 */
export function swipeAction(subject: PreviewSubject, direction: SwipeDirection): SwipeAction {
  if (direction === 'left') return subject.kind === 'shot' ? 'retake' : 'delete';
  if (direction === 'right') return subject.kind === 'shot' ? 'main_confirm' : subject.kind === 'extra' ? 'none' : 'main';
  return subject.kind === 'condition' ? 'none' : 'condition';
}

/**
 * Plan point P4: the photo after `photoId` in the strip's order (template shots that have a photo
 * in shooting order, the extras, the Condition images); `null` after the last one, or when the
 * photo is not in the session.
 */
export function nextPhoto(view: PhoneSession, photoId: number): number | null {
  const ids = photoIds(view);
  const at = ids.indexOf(photoId);
  return at < 0 ? null : ids[at + 1] ?? null;
}

/**
 * A tile's home (PH9.3): where it stands in the strip's content, measured once when a photo is
 * lifted and never during the drag (tiles sliding aside move on screen, not their homes).
 */
export type Home = {
  /** The dock tile's key (`p<photo>`, an empty shot's `s<container>`, `free`). */
  key: string;
  kind: 'shot' | 'extra' | 'free' | 'condition';
  /** The tile's centre: x in content coordinates (client left − strip left + `scrollLeft`). */
  centre: number;
  /** A template shot's container. */
  container_id: number | null;
};

/** Where a lifted photo would go if released now, and so what the Mac is asked (PH9.3). */
export type Drop =
  /** Onto a template shot: the two swap, an empty shot is filled (card LV8) → move `{container}`. */
  | { kind: 'swap'; container_id: number }
  /** Among its own group: put at place `index` (1-based), the others shift → move `{extra_insert | condition_insert}`. */
  | { kind: 'insert'; group: 'extra' | 'condition'; index: number }
  /** Into the other group, at its end (book 06) → classify. */
  | { kind: 'to_main' }
  | { kind: 'to_condition' }
  /** A template shot's photo into the extras: the confirmation first, then as `to_main`. */
  | { kind: 'main_confirm' };

/**
 * PH9.3: the target under the finger. `fingerX` is in content coordinates; `inBand` says the
 * finger is within the band above and below the strip. Over a template shot other than the
 * source → swap. In the span of the extras or of the Condition images (from half a pitch before
 * the first tile's centre to half a pitch after the last one's) → insert within the source's own
 * group (its place after the move = 1 + the others whose centre is left of the finger; its own
 * place → nothing), or to that group's end from outside it (a template shot's photo into the
 * extras asks first). Anywhere else (its own tile, the Free shot, a gap, out of the band) → nothing.
 */
export function dropTarget(homes: readonly Home[], source: string, fingerX: number, inBand: boolean): Drop | null {
  const from = homes.find(home => home.key === source);
  if (!inBand || !from || from.kind === 'free') return null;
  const half = DOCK.nearPx;
  const shot = homes.find(home => home.kind === 'shot' && home.key !== source && home.container_id !== null && Math.abs(fingerX - home.centre) < half);
  if (shot && shot.container_id !== null) return { kind: 'swap', container_id: shot.container_id };
  const group = (kind: 'extra' | 'condition') => homes.filter(home => home.kind === kind).sort((a, b) => a.centre - b.centre);
  const within = (tiles: Home[]) => tiles.length > 0 && fingerX >= tiles[0].centre - half && fingerX < tiles[tiles.length - 1].centre + half;
  const insert = (kind: 'extra' | 'condition', tiles: Home[]): Drop | null => {
    const index = 1 + tiles.filter(home => home.key !== source && home.centre < fingerX).length;
    return index === 1 + tiles.findIndex(home => home.key === source) ? null : { kind: 'insert', group: kind, index };
  };
  const extras = group('extra');
  if (within(extras)) return from.kind === 'shot' ? { kind: 'main_confirm' } : from.kind === 'condition' ? { kind: 'to_main' } : insert('extra', extras);
  const condition = group('condition');
  if (within(condition)) return from.kind === 'condition' ? insert('condition', condition) : { kind: 'to_condition' };
  return null;
}

/** PH9.5a: a denied photo (a swipe's Retake or delete) leaves its place; it is no drop target. */
export type Removal = { kind: 'remove' };

/**
 * PH9.5, PH9.5a: the arrangement the strips show while the Mac answers a drop or a swipe of
 * `photoId`. Swap: the two exchange places; an empty shot is filled and the source's place is left
 * empty (a shot) or closed up (an extra, a Condition image). Insert: the photo is put at its place,
 * the others keep their order. To Main / to Condition: out of its place, to that group's end.
 * Remove: out of its place (a template shot keeps its slot, empty; an extra or a Condition image
 * leaves its group). Nothing else changes (the current shot and the counts are the Mac's). A photo
 * that is not in the view changes nothing.
 */
export function applyDrop(view: PhoneSession, photoId: number, drop: Drop | Removal): PhoneSession {
  const shots = [...view.shots];
  const extras = [...view.extras];
  const condition = [...view.condition];
  const inShot = shots.findIndex(shot => shot.photo?.photo_id === photoId);
  const inExtras = extras.findIndex(image => image.photo_id === photoId);
  const inCondition = condition.findIndex(image => image.photo_id === photoId);
  const image: PhoneImage | null = shots[inShot]?.photo ?? extras[inExtras] ?? condition[inCondition] ?? null;
  if (!image) return view;
  /** The photo leaves its place; `other` (a swap's counterpart) takes it, if there is one. */
  const leave = (other: PhoneImage | null) => {
    if (inShot >= 0) shots[inShot] = { ...shots[inShot], photo: other };
    else if (inExtras >= 0) extras.splice(inExtras, 1, ...(other ? [other] : []));
    else condition.splice(inCondition, 1, ...(other ? [other] : []));
  };
  if (drop.kind === 'swap') {
    const target = shots.findIndex(shot => shot.container_id === drop.container_id);
    if (target < 0 || target === inShot) return view;
    const other = shots[target].photo;
    shots[target] = { ...shots[target], photo: image };
    leave(other);
  } else if (drop.kind === 'insert') {
    const list = drop.group === 'extra' ? extras : condition;
    const at = list.findIndex(candidate => candidate.photo_id === photoId);
    if (at < 0) return view;
    list.splice(at, 1);
    list.splice(drop.index - 1, 0, image);
  } else {
    leave(null);
    if (drop.kind === 'to_condition') condition.push(image);
    else if (drop.kind !== 'remove') extras.push(image);
  }
  return { ...view, shots, extras, condition };
}

/**
 * PH9.4: which photo is where. A drag is cancelled when this changes under it; a new preview
 * picture, a revision or the current shot is not part of it (a picture arriving never cancels).
 * The shots' `container:photo` (`-` when empty) in shooting order, then the extras' ids, then the
 * Condition ids.
 */
export function arrangementKey(view: PhoneSession): string {
  return [
    view.shots.map(shot => `${shot.container_id}:${shot.photo?.photo_id ?? '-'}`).join(','),
    view.extras.map(image => image.photo_id).join(','),
    view.condition.map(image => image.photo_id).join(','),
  ].join('|');
}
