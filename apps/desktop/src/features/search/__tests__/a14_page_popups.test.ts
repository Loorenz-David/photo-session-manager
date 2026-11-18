import { afterEach, beforeEach, expect, it, vi } from 'vitest';
// The script the page webview runs in every frame (search_window.rs: `(<this>)(<go>)`).
import shim from '../../../../src-tauri/src/page_popups.js?raw';

const nativeOpen = window.open;
let go: ReturnType<typeof vi.fn>;
let native: ReturnType<typeof vi.fn>;

function install() {
  // eslint-disable-next-line @typescript-eslint/no-implied-eval
  const wrapped = new Function(`return (${shim.trim()})`)() as (go: (url: string) => void) => void;
  wrapped(go as unknown as (url: string) => void);
}

beforeEach(() => {
  go = vi.fn();
  native = vi.fn(() => null);
  window.open = native as unknown as typeof window.open;
  document.body.innerHTML = '';
  install();
});

afterEach(() => {
  window.open = nativeOpen;
});

it('a14_window_open_of_an_address_opens_it_here', () => {
  const opened = window.open('https://www.rusta.com/sv/p?id=1', '_blank', 'noopener');
  expect(go).toHaveBeenCalledWith('https://www.rusta.com/sv/p?id=1');
  expect(native).not.toHaveBeenCalled();
  expect(opened).not.toBeNull();
  // Relative addresses are made absolute against the page.
  window.open('/search?q=x');
  expect(go).toHaveBeenLastCalledWith(new URL('/search?q=x', document.baseURI).href);
});

it('a14_an_empty_window_whose_address_is_set_afterwards_opens_that_address_here', () => {
  const first = window.open('', '_blank')!;
  expect(go).not.toHaveBeenCalled();
  (first as unknown as { location: string }).location = 'https://www.amazon.se/dp/1';
  expect(go).toHaveBeenLastCalledWith('https://www.amazon.se/dp/1');
  const second = window.open('about:blank')!;
  second.location.href = 'https://www.rusta.com/a';
  expect(go).toHaveBeenLastCalledWith('https://www.rusta.com/a');
  const third = window.open()!;
  third.location.assign('https://e1.example/b');
  expect(go).toHaveBeenLastCalledWith('https://e1.example/b');
  third.close();
  expect(third.closed).toBe(true);
});

it('a14_only_web_addresses_are_followed', () => {
  window.open('javascript:alert(1)', '_blank');
  window.open('data:text/html,<b>x</b>', '_blank');
  const stand = window.open('', 'x')!;
  stand.location.href = 'file:///etc/hosts';
  expect(go).not.toHaveBeenCalled();
});

it('a14_opening_in_this_window_is_left_to_the_page', () => {
  window.open('https://e1.example/self', '_self');
  expect(native).toHaveBeenCalledTimes(1);
  expect(go).not.toHaveBeenCalled();
});

it('a14_links_and_forms_aimed_at_a_new_window_are_aimed_at_this_page', () => {
  document.body.innerHTML = `
    <a id="blank" href="https://e1.example/a" target="_blank"><span id="inner">Rusta +1</span></a>
    <a id="named" href="https://e1.example/b" target="results">b</a>
    <a id="self" href="https://e1.example/c" target="_self">c</a>
    <form id="form" target="_blank" action="https://e1.example/f"></form>`;
  const prevent = (event: Event) => event.preventDefault();
  document.addEventListener('click', prevent);
  document.getElementById('inner')!.click();
  expect(document.getElementById('blank')!.getAttribute('target')).toBe('_top');
  document.getElementById('named')!.click();
  expect(document.getElementById('named')!.getAttribute('target')).toBe('_top');
  document.getElementById('self')!.click();
  expect(document.getElementById('self')!.getAttribute('target')).toBe('_self');
  document.removeEventListener('click', prevent);
  const form = document.getElementById('form') as HTMLFormElement;
  form.addEventListener('submit', prevent);
  form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
  expect(form.getAttribute('target')).toBe('_top');
});
