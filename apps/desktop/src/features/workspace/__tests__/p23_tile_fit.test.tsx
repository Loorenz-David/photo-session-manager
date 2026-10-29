import { expect, it } from 'vitest';
// @ts-expect-error The frontend tsconfig omits Node types; Vitest runs this test in Node.
import { readFileSync } from 'node:fs';
const css = readFileSync('src/features/workspace/workspace.css', 'utf8');
it('C5(a) fits the whole image inside the existing tile and stage', () => {
  const rule = (selector: string) => {
    const start = css.indexOf(`${selector} {`);
    expect(start).toBeGreaterThanOrEqual(0);
    return css.slice(start).split('}')[0];
  };
  expect(rule('.thumb-image')).toMatch(/object-fit:\s*contain/);
  expect(rule('.thumb-image')).not.toMatch(/object-fit:\s*cover/);
  expect(rule('.stage-layer')).toMatch(/object-fit:\s*contain/);
  expect(rule('.tile-grid')).toMatch(/aspect-ratio:\s*4\s*\/\s*3/);
});
