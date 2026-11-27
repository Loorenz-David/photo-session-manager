// LV9 — the plan's core mechanisms (`arrange.ts`; master_plan_live §5 PH9.4, PH9.5a): input → output on the
// real functions, one case per rule line. Fixture D (helpers `dragSession`): shots B (photo 201), A (204),
// C (empty); extras 202, 205, 206; Condition 203, 207, 208.
import { expect, test } from 'vitest';
import { arrangementKey } from '../arrange';
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
