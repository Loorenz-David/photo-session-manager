import { describe, expect, it } from 'vitest';
import { outputLabel, outputNumbers, outputTotal, type Numberable } from '../outputs';

const photo = (id: number, classification: Numberable['classification'], kind: 'normal' | 'split' = 'normal'): Numberable => ({ id, classification, composition: { kind } });

describe('P33 C1(b) output numbering: a split photo takes two consecutive numbers, outputs expand in photo order (MC-29)', () => {
  const main = [photo(1, 'main', 'split'), photo(2, 'main'), photo(3, 'main', 'split'), photo(4, 'unreviewed')];
  const condition = [photo(5, 'condition', 'split'), photo(6, 'condition')];

  it('Main [a (split), b, c (split), u (unreviewed)] → a {1, 2}, b {3, 3}, c {4, 5}, u {6, 6}', () => {
    const numbers = outputNumbers(main);
    expect(numbers.get(1)).toEqual({ first: 1, last: 2 });
    expect(numbers.get(2)).toEqual({ first: 3, last: 3 });
    expect(numbers.get(3)).toEqual({ first: 4, last: 5 });
    expect(numbers.get(4)).toEqual({ first: 6, last: 6 });
    expect(numbers.size).toBe(4);
    expect(outputTotal(main)).toBe(6);
  });

  it('Condition [d (split), e] → d {1, 2}, e {3, 3}', () => {
    const numbers = outputNumbers(condition);
    expect(numbers.get(5)).toEqual({ first: 1, last: 2 });
    expect(numbers.get(6)).toEqual({ first: 3, last: 3 });
    expect(outputTotal(condition)).toBe(3);
  });

  it('outputLabel(1, 2) is "01–02" (en dash) and outputLabel(3, 3) is "03"', () => {
    expect(outputLabel(1, 2)).toBe('01–02');
    expect(outputLabel(3, 3)).toBe('03');
    expect(outputLabel(10, 11)).toBe('10–11');
  });

  it('an unreviewed photo counts one place whatever its kind (not an output until reviewed)', () => {
    const numbers = outputNumbers([photo(1, 'main', 'split'), photo(2, 'unreviewed', 'split'), photo(3, 'unreviewed')]);
    expect(numbers.get(2)).toEqual({ first: 3, last: 3 });
    expect(numbers.get(3)).toEqual({ first: 4, last: 4 });
  });
});
