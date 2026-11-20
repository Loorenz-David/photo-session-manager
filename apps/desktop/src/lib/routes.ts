/** Parent and label of the current hash route, shared by header links and keyboard Back. */
export function parentRoute(hash: string): string | null {
  if (hash === '' || hash === '#/') return null;
  if (hash === '#/health') return '#/settings';
  return '#/';
}

export function routeLabel(hash: string): string {
  if (hash === '#/settings') return 'Settings';
  if (hash === '#/health') return 'Health';
  if (/^#\/session\/\d+$/.test(hash)) return 'Workspace';
  return 'Sessions';
}

/**
 * A page that must be asked before it is left (Settings with unsaved changes, P63): returns true to let the user go
 * now, or false to stay; on false the page keeps `proceed` and calls it later if the user still leaves (C4: the
 * continuation may be a visit's jump rather than a hash write, so the page never writes the hash itself).
 */
export type LeaveGuard = (target: string, proceed: () => void) => boolean;
let leaveGuard: LeaveGuard | null = null;

/** Installs (or with null removes) the guard asked by `leaveThen`. */
export function setLeaveGuard(guard: LeaveGuard | null): void {
  leaveGuard = guard;
}

/** Runs `proceed` now unless the current page's leave guard keeps the user there (it then holds `proceed`). */
export function leaveThen(target: string, proceed: () => void): void {
  if (!leaveGuard || leaveGuard(target, proceed)) proceed();
}

/** Navigates to `hash` unless the current page's leave guard keeps the user there. */
export function goTo(hash: string): void {
  leaveThen(hash, () => {
    window.location.hash = hash;
  });
}
