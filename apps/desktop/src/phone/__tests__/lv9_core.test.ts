// LV9 — the plan's core mechanisms (`arrange.ts`; master_plan_live §5 PH9.4, PH9.5a): input → output on the
// real functions, one case per rule line. Fixture D (helpers `dragSession`): shots B (photo 201), A (204),
// C (empty); extras 202, 205, 206; Condition 203, 207, 208.
import { expect, test } from 'vitest';
import { applyDrop, arrangementKey } from '../arrange';
import { dragSession, dragWaitingSession } from './helpers';

// ---- arrangementKey: photos on their way are not part of which photo is where (U2) ----

test('lv9_core_arrangementkey_ignores_photos_on_their_way', () => {
  const key = arrangementKey(dragSession());
  expect(arrangementKey(dragWaitingSession())).toBe(key);
  expect(arrangementKey(dragSession(s => {
    s.shots[2].incoming = true;
    s.incoming = { extras: 2, condition: 3 };
  }))).toBe(key);
});

// ---- applyDrop, kind `remove`: a denied photo leaves its place (U3; PH9.5a) ----

const ids = (images: { photo_id: number }[]) => images.map(image => image.photo_id);
const inShots = (view: { shots: { photo: { photo_id: number } | null }[] }) => view.shots.map(shot => shot.photo?.photo_id ?? null);

test('lv9_core_applydrop_remove_a_template_shots_photo_leaves_its_shot_empty', () => {
  const given = dragSession();
  const shown = applyDrop(given, 201, { kind: 'remove' });
  expect(inShots(shown)).toEqual([null, 204, null]);
  expect(shown.shots.map(shot => shot.container_id)).toEqual([12, 11, 13]);
  expect([ids(shown.extras), ids(shown.condition)]).toEqual([[202, 205, 206], [203, 207, 208]]);
  expect([shown.current, shown.counts]).toEqual([given.current, given.counts]);
});

test('lv9_core_applydrop_remove_an_extra_leaves_the_extras', () => {
  const given = dragSession();
  const shown = applyDrop(given, 205, { kind: 'remove' });
  expect([inShots(shown), ids(shown.extras), ids(shown.condition)]).toEqual([[201, 204, null], [202, 206], [203, 207, 208]]);
  expect([shown.current, shown.counts]).toEqual([given.current, given.counts]);
});

test('lv9_core_applydrop_remove_a_condition_image_leaves_the_condition_images', () => {
  const given = dragSession();
  const shown = applyDrop(given, 203, { kind: 'remove' });
  expect([inShots(shown), ids(shown.extras), ids(shown.condition)]).toEqual([[201, 204, null], [202, 205, 206], [207, 208]]);
  expect([shown.current, shown.counts]).toEqual([given.current, given.counts]);
});
