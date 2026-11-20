/**
 * C4 (09B §7 "The jump and the way back", master plan MC-C3): route and visit are **one** state, changed only by
 * `visitStep`. `App.tsx` hands `useCaptureTarget` the derived `captureTarget(state)`, so a visit never moves the capture
 * folder: there is no render in which the route is a visited session while the visit is not set (or the reverse).
 */

/** MC-C3.1: the four hash routes. `sessionId` is set iff `kind === 'session'`. */
export type Route = { kind: 'sessions' | 'session' | 'settings' | 'health'; sessionId?: number };

/** `capture` = the session the capture folder pointed at at the first jump (`undefined` = Unassigned). */
export type Visit = { origin: Route; capture: number | undefined; visited: number };

export type VisitState = { route: Route; visit: Visit | null };

export type VisitAction =
  | { type: 'open-session'; sessionId: number }
  | { type: 'back' }
  | { type: 'shoot-here' }
  | { type: 'route-changed'; route: Route };

const SESSIONS: Route = { kind: 'sessions' };

export function sessionRoute(sessionId: number): Route {
  return { kind: 'session', sessionId };
}

/** The route of a hash; an unknown hash is the Sessions page (as before C4). */
export function routeOf(hash: string): Route {
  if (hash === '#/settings') return { kind: 'settings' };
  if (hash === '#/health') return { kind: 'health' };
  const match = hash.match(/^#\/session\/(\d+)$/);
  return match ? sessionRoute(Number(match[1])) : SESSIONS;
}

export function hashOf(route: Route): string {
  if (route.kind === 'settings') return '#/settings';
  if (route.kind === 'health') return '#/health';
  if (route.kind === 'session' && route.sessionId !== undefined) return `#/session/${route.sessionId}`;
  return '#/';
}

export function sameRoute(a: Route, b: Route): boolean {
  return a.kind === b.kind && (a.kind !== 'session' || a.sessionId === b.sessionId);
}

/** The route's session id (`undefined` off a session). */
export function routeSession(route: Route): number | undefined {
  return route.kind === 'session' ? route.sessionId : undefined;
}

/** MC-C3.3: the one way route and visit change. */
export function visitStep(state: VisitState, action: VisitAction): VisitState {
  const { route, visit } = state;
  switch (action.type) {
    case 'open-session': {
      const target = sessionRoute(action.sessionId);
      if (sameRoute(route, target)) return state;
      if (visit === null) {
        return { route: target, visit: { origin: route, capture: routeSession(route), visited: action.sessionId } };
      }
      if (action.sessionId === visit.capture) return visitStep(state, { type: 'back' });
      return { route: target, visit: { ...visit, visited: action.sessionId } };
    }
    case 'back':
      return visit === null ? state : { route: visit.origin, visit: null };
    case 'shoot-here':
      return visit === null ? state : { route, visit: null };
    case 'route-changed':
      if (visit !== null && sameRoute(action.route, sessionRoute(visit.visited))) return state;
      if (visit === null && sameRoute(action.route, route)) return state;
      return { route: action.route, visit: null };
  }
}

/** MC-C3.2: where new captures go: the visit's capture while visiting, else the route's session. */
export function captureTarget(state: VisitState): number | undefined {
  return state.visit !== null ? state.visit.capture : routeSession(state.route);
}
