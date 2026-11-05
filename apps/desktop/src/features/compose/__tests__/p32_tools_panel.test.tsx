// The harness registers the service mocks; it must be evaluated before this file's own service imports.
import { compose, layoutStage, openWorkspace, stage, twoPhotos } from './harness';
import { act, fireEvent, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

const panel = () => document.querySelector<HTMLElement>('aside.tools-panel');
const firstSection = () => panel()!.querySelector<HTMLElement>('.tools-section')!;
const shown = () => panel() !== null && !panel()!.hidden;

describe('C2 the Tools panel', () => {
  it('C2(a) the panel sits on the left with Composition open first; folding is remembered across a remount; ⌘T hides and shows it', async () => {
    window.localStorage.removeItem('tools.sections');
    window.localStorage.removeItem('tools.hidden');
    layoutStage();
    await openWorkspace([twoPhotos()]);
    await compose();
    expect(panel()).not.toBeNull();
    expect(panel()!.previousElementSibling).toBeNull();
    expect(panel()!.nextElementSibling?.classList.contains('stage-column')).toBe(true);
    const section = firstSection();
    expect(within(section).getByRole('button', { name: 'Composition' })).toBeInTheDocument();
    expect(section.getAttribute('data-open')).toBe('true');
    expect(section.querySelector('#compose-shape')).not.toBeNull();

    fireEvent.click(within(section).getByRole('button', { name: 'Composition' }));
    expect(firstSection().getAttribute('data-open')).toBe('false');
    expect(firstSection().querySelector('#compose-shape')).toBeNull();
    expect(window.localStorage.getItem('tools.sections')).toContain('"composition":false');

    // Navigate away and back: the workspace unmounts and remounts; the fold is remembered.
    act(() => {
      window.location.hash = '#/';
    });
    await waitFor(() => expect(document.querySelector('main.workspace')).toBeNull());
    act(() => {
      window.location.hash = '#/session/5';
    });
    await waitFor(() => expect(document.querySelectorAll('.photo-panel [data-photo-id]').length).toBe(2));
    expect(panel()).toBeNull();
    await compose();
    expect(firstSection().getAttribute('data-open')).toBe('false');

    fireEvent.keyDown(window, { key: 't', metaKey: true });
    expect(shown()).toBe(false);
    expect(window.localStorage.getItem('tools.hidden')).toBe('1');
    expect(stage()).not.toBeNull();
    fireEvent.keyDown(window, { key: 't', metaKey: true });
    expect(shown()).toBe(true);
    expect(window.localStorage.getItem('tools.hidden')).not.toBe('1');
  });

  it('C2(b) the normal workspace has no Tools panel and keeps the photo panel on the right', async () => {
    layoutStage();
    await openWorkspace([twoPhotos()]);
    expect(panel()).toBeNull();
    const photoPanel = document.querySelector('main.workspace > aside.photo-panel');
    expect(photoPanel).not.toBeNull();
    expect(photoPanel!.previousElementSibling?.classList.contains('stage-column')).toBe(true);
    await compose();
    expect(panel()).not.toBeNull();
    fireEvent.keyDown(window, { key: 'Enter' });
    await waitFor(() => expect(stage()).toBeNull());
    expect(panel()).toBeNull();
    expect(document.querySelector('main.workspace > aside.photo-panel')).not.toBeNull();
  });
});
