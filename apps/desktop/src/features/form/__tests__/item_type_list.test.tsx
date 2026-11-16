import '@testing-library/jest-dom/vitest';
import { useState } from 'react';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { ItemTypePicker, matchingTypes } from '../ItemTypePicker';

vi.mock('@/services/items', () => ({ listItemTypes: vi.fn().mockResolvedValue([]) }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

function Picker({ types, hints, onChoose, onBlur }: { types: string[]; hints?: Record<string, string>; onChoose?: (v: string) => void; onBlur?: () => void }) {
  const [value, setValue] = useState('');
  return <ItemTypePicker id="t" value={value} onChange={setValue} types={types} failed={false} hints={hints} onChoose={onChoose} onBlur={onBlur} />;
}

it('the list filters on what is typed and shows every type once a listed one is in the field', () => {
  expect(matchingTypes(['Armchairs', 'Chairs', 'Lamps'], 'chair')).toEqual(['Armchairs', 'Chairs']);
  expect(matchingTypes(['Armchairs', 'Chairs', 'Lamps'], 'Chairs')).toEqual(['Armchairs', 'Chairs', 'Lamps']);
  expect(matchingTypes(['Armchairs'], '')).toEqual(['Armchairs']);
  expect(matchingTypes(['Armchairs'], undefined)).toEqual(['Armchairs']);
});

it('the chevron opens the app list; a click picks a type with its note shown', () => {
  const onChoose = vi.fn();
  render(<Picker types={['Armchairs', 'Lamps']} hints={{ Armchairs: '6 containers' }} onChoose={onChoose} />);
  const field = screen.getByRole('combobox', { name: 'Item type' });
  expect(field).toHaveAttribute('aria-expanded', 'false');
  fireEvent.click(screen.getByRole('button', { name: 'Show item types' }));
  expect(field).toHaveAttribute('aria-expanded', 'true');
  const list = screen.getByRole('listbox', { name: 'Item types' });
  expect(within(list).getAllByRole('option').map(o => o.textContent)).toEqual(['Armchairs6 containers', 'Lamps']);
  fireEvent.click(within(list).getByRole('option', { name: /Lamps/ }));
  expect(field).toHaveValue('Lamps');
  expect(onChoose).toHaveBeenCalledWith('Lamps');
  expect(field).toHaveAttribute('aria-expanded', 'false');
});

it('typing narrows the list; arrows and Return pick; Escape closes', () => {
  render(<Picker types={['Armchairs', 'Chairs', 'Lamps']} />);
  const field = screen.getByRole('combobox', { name: 'Item type' });
  fireEvent.change(field, { target: { value: 'chai' } });
  expect(screen.getAllByRole('option').map(o => o.textContent)).toEqual(['Armchairs', 'Chairs']);
  fireEvent.keyDown(field, { key: 'ArrowDown' });
  fireEvent.keyDown(field, { key: 'Enter' });
  expect(field).toHaveValue('Chairs');
  fireEvent.keyDown(field, { key: 'ArrowDown' });
  expect(field).toHaveAttribute('aria-expanded', 'true');
  fireEvent.keyDown(field, { key: 'Escape' });
  expect(field).toHaveAttribute('aria-expanded', 'false');
});
