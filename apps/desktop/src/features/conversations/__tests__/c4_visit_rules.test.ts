import { expect, it } from 'vitest';
import { captureTarget, visitStep, type Route, type VisitState } from '../visit';

const S = (sessionId: number): Route => ({ kind: 'session', sessionId });
const SESSIONS: Route = { kind: 'sessions' };
const SETTINGS: Route = { kind: 'settings' };

const at = (route: Route): VisitState => ({ route, visit: null });
const visiting7From3: VisitState = { route: S(7), visit: { origin: S(3), capture: 3, visited: 7 } };
const visiting7FromSessions: VisitState = { route: S(7), visit: { origin: SESSIONS, capture: undefined, visited: 7 } };

it('c4_c1a_open_session_from_a_session_starts_a_visit_keeping_its_capture', () => {
  expect(visitStep(at(S(3)), { type: 'open-session', sessionId: 7 })).toEqual(visiting7From3);
});

it('c4_c1b_open_session_from_the_sessions_page_captures_to_unassigned', () => {
  expect(visitStep(at(SESSIONS), { type: 'open-session', sessionId: 7 })).toEqual(visiting7FromSessions);
});

it('c4_c1c_open_session_from_settings_has_settings_as_origin', () => {
  expect(visitStep(at(SETTINGS), { type: 'open-session', sessionId: 7 })).toEqual({
    route: S(7),
    visit: { origin: SETTINGS, capture: undefined, visited: 7 },
  });
});

it('c4_c1d_jumping_again_keeps_the_origin_and_capture', () => {
  expect(visitStep(visiting7From3, { type: 'open-session', sessionId: 9 })).toEqual({
    route: S(9),
    visit: { origin: S(3), capture: 3, visited: 9 },
  });
});

it('c4_c1e_opening_the_session_being_shot_ends_the_visit', () => {
  expect(visitStep(visiting7From3, { type: 'open-session', sessionId: 3 })).toEqual({ route: S(3), visit: null });
});

it('c4_c1f_back_after_two_jumps_returns_to_the_first_origin', () => {
  const twice = visitStep(visiting7From3, { type: 'open-session', sessionId: 9 });
  expect(twice.route).toEqual(S(9));
  expect(visitStep(twice, { type: 'back' })).toEqual({ route: S(3), visit: null });
});

it('c4_c1g_back_from_a_visit_started_on_the_sessions_page_returns_there', () => {
  expect(visitStep(visiting7FromSessions, { type: 'back' })).toEqual({ route: SESSIONS, visit: null });
});

it('c4_c1h_shoot_here_ends_the_visit_and_stays', () => {
  expect(visitStep(visiting7From3, { type: 'shoot-here' })).toEqual({ route: S(7), visit: null });
});

it('c4_c1i_a_route_change_to_the_visited_session_changes_nothing', () => {
  expect(visitStep(visiting7From3, { type: 'route-changed', route: S(7) })).toEqual(visiting7From3);
});

it('c4_c1j_a_route_change_to_the_sessions_page_ends_the_visit', () => {
  expect(visitStep(visiting7From3, { type: 'route-changed', route: SESSIONS })).toEqual({ route: SESSIONS, visit: null });
});

it('c4_c1k_a_route_change_to_another_session_ends_the_visit', () => {
  expect(visitStep(visiting7From3, { type: 'route-changed', route: S(5) })).toEqual({ route: S(5), visit: null });
});

it('c4_c1l_without_a_visit_back_shoot_here_and_opening_the_session_on_screen_change_nothing', () => {
  const state = at(S(3));
  expect(visitStep(state, { type: 'back' })).toEqual(state);
  expect(visitStep(state, { type: 'shoot-here' })).toEqual(state);
  expect(visitStep(state, { type: 'open-session', sessionId: 3 })).toEqual(state);
});

it('c4_c2a_while_visiting_from_a_session_captures_go_to_that_session', () => {
  expect(captureTarget(visiting7From3)).toBe(3);
});

it('c4_c2b_while_visiting_from_the_sessions_page_captures_go_to_unassigned', () => {
  expect(captureTarget(visiting7FromSessions)).toBeUndefined();
});

it('c4_c2c_without_a_visit_captures_follow_the_route', () => {
  expect(captureTarget(at(S(5)))).toBe(5);
  expect(captureTarget(at(SESSIONS))).toBeUndefined();
  expect(captureTarget(at(SETTINGS))).toBeUndefined();
});
