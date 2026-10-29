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
