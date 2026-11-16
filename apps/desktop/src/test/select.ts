import { act, fireEvent, screen, waitFor } from '@testing-library/react';

/** The labels a styled dropdown (components/ui/select) offers, read by opening and closing it. */
export async function optionsOf(trigger: HTMLElement): Promise<string[]> {
  fireEvent.click(trigger);
  const options = (await screen.findAllByRole('option')).map(option => option.textContent ?? '');
  fireEvent.keyDown(screen.getByRole('listbox'), { key: 'Escape' });
  return options;
}

/** Picks the option with this label in a styled dropdown, the way a click would. */
export async function choose(trigger: HTMLElement, label: string | RegExp): Promise<void> {
  fireEvent.click(trigger);
  fireEvent.click(await screen.findByRole('option', { name: label }));
}

/** Picks the option with this stored value (empty string = "None"), the way a click would. */
export async function chooseValue(trigger: Element, value: string): Promise<void> {
  await act(async () => { fireEvent.click(trigger); });
  const token = value === '' ? '__none__' : value;
  const option = await waitFor(() => {
    const found = document.querySelector(`[role="option"][data-value="${CSS.escape(token)}"]`);
    if (!found) throw new Error(`no option ${value}`);
    return found;
  });
  await act(async () => { fireEvent.click(option); });
}
