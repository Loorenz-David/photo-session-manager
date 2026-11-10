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

/** A page that must be asked before it is left (Settings with unsaved changes, P63): returns false to stay. */
export type LeaveGuard = (target: string) => boolean;
let leaveGuard: LeaveGuard | null = null;

/** Installs (or with null removes) the guard asked by `goTo`. */
export function setLeaveGuard(guard: LeaveGuard | null): void {
  leaveGuard = guard;
}

/** Navigates to `hash` unless the current page's leave guard keeps the user there. */
export function goTo(hash: string): void {
  if (leaveGuard && !leaveGuard(hash)) return;
  window.location.hash = hash;
}
