// LV7 — the loading cue on a tile whose picture is still arriving (owner request R2; plan LV7_tile_loading_cue):
// rows LV7·C1–C3. The fixture session (helpers), B's photo (201, tile "02"). jsdom runs no CSS animation, so the
// rows read the class the placeholder carries and the rule `phone.css` gives it.
// @ts-expect-error The frontend tsconfig omits Node types; Vitest runs this test in Node.
import { readFileSync } from 'node:fs';
import { fireEvent, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, chairSession, lift, macChanges, phoneEnv, press, startPhone } from './helpers';

/** The phone's stylesheet as shipped, without its comments. */
const CSS = readFileSync('src/phone/phone.css', 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
const X_B = 44;
const Y = 742;
const dock = () => screen.getByTestId('dock');
const tileB = () => dock().querySelector('[aria-label="02 · Back"]') as HTMLElement;
const wait = () => tileB().querySelector('[data-testid="photo-wait"]');
const withPreview = (preview: string | null) => chairSession(s => {
  s.shots[0].photo = { photo_id: 201, revision: 3, ratio: 1, preview };
});

type Rule = { selector: string; body: string };
/** Every plain rule in `css` (innermost blocks): its selector and its declarations. */
function rulesOf(css: string): Rule[] {
  const found: Rule[] = [];
  const pattern = /([^{}]+)\{([^{}]*)\}/g;
  for (let match = pattern.exec(css); match; match = pattern.exec(css)) found.push({ selector: match[1].trim(), body: match[2] });
  return found;
}
/** The declarations of every rule whose selector is the placeholder or one of its pseudo-elements. */
const cueRules = (css: string) => rulesOf(css).filter(rule => /^\.ph-photo-wait(::(before|after))?$/.test(rule.selector)).map(rule => rule.body);
/** The values `property` is given in those declarations. */
function declared(bodies: string[], property: string): string[] {
  const values: string[] = [];
  for (const body of bodies) {
    const pattern = new RegExp(`(?:^|[;\\s])${property}:\\s*([^;]+);`, 'g');
    for (let match = pattern.exec(body); match; match = pattern.exec(body)) values.push(match[1].trim());
  }
  return values;
}
/** The stylesheet without its `@media` blocks, and the inside of its reduced-motion block. */
const REDUCED = /@media \(prefers-reduced-motion: reduce\)\s*\{((?:[^{}]*\{[^{}]*\})*)\s*\}/;
const always = CSS.replace(/@media[^{]*\{(?:[^{}]*\{[^{}]*\})*\s*\}/g, '');

test('lv7_c1a_a_tile_without_a_preview_carries_the_cue_a_slow_endless_sweep', async () => {
  const env = phoneEnv();
  await startPhone(env, withPreview(null));
  expect(wait()!.classList.contains('ph-photo-wait')).toBe(true);
  const animations = declared(cueRules(always), 'animation');
  expect(animations).toHaveLength(1);
  const [, name, seconds, rest] = /^([\w-]+) ([\d.]+)s (.*)$/.exec(animations[0])!;
  expect(CSS).toContain(`@keyframes ${name} {`);
  expect(Number(seconds)).toBeGreaterThanOrEqual(1.2);
  expect(Number(seconds)).toBeLessThanOrEqual(2.5);
  expect(rest.split(' ')).toContain('infinite');
});

test('lv7_c1b_the_cue_leaves_when_the_picture_loads', async () => {
  const env = phoneEnv();
  await startPhone(env, withPreview(null));
  expect(wait()).not.toBeNull();
  await macChanges(env, withPreview('k1'));
  const picture = tileB().querySelector('img')!;
  fireEvent.load(picture);
  expect(wait()).toBeNull();
  expect(tileB().querySelector('img')!.getAttribute('src')).toBe('/api/phone/v1/photos/201/preview?pair=T&r=3&v=k1');
});

test('lv7_c2a_reduced_motion_turns_the_sweep_off', () => {
  const reduced = REDUCED.exec(CSS);
  expect(reduced).not.toBeNull();
  // The same selector that moves is the one stopped.
  const moving = rulesOf(always).filter(rule => rule.selector.startsWith('.ph-photo-wait') && /animation:/.test(rule.body)).map(rule => rule.selector);
  const stopped = rulesOf(reduced![1]).filter(rule => /(?:^|[;\s])animation:\s*none;/.test(rule.body)).map(rule => rule.selector);
  expect(moving).toHaveLength(1);
  expect(stopped).toEqual(moving);
});

test('lv7_c3a_a_waiting_tile_does_not_hold_up_the_shutter_and_still_opens_on_a_tap', async () => {
  const env = phoneEnv();
  await startPhone(env, withPreview(null));
  expect(wait()).not.toBeNull();
  // The placeholder and its sweep take no pointer events: the tile under them does.
  const rules = cueRules(always);
  expect(rules).toHaveLength(2);
  expect(declared(rules, 'pointer-events')).toEqual(['none', 'none']);
  const shutter = screen.getByRole('button', { name: 'Capture' });
  fireEvent.pointerDown(shutter, { pointerId: 3 });
  fireEvent.pointerUp(shutter, { pointerId: 3 });
  await advance(0);
  expect(env.mac.posts('/capture')).toEqual([{ session_id: 7 }]);
  press(dock(), X_B, Y);
  lift(dock(), X_B, Y);
  await advance(0);
  expect(screen.getByTestId('preview').querySelector('.ph-kicker')!.textContent).toBe('SHOT 02 · CAPTURED');
});
