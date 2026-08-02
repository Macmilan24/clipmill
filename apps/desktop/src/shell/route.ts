/**
 * Where the shell is, which is not quite the same question as which navigation
 * row is lit.
 *
 * Eight of the nine screens answer to a navigation section and the two questions
 * collapse into one. Analysis Progress does not. The design gives it no row —
 * the nine are fixed and must not be regrouped or added to — and shows it with
 * the row it was opened from still active and a two-part breadcrumb. It is also
 * about one particular run, so "which screen" carries an argument that a section
 * id has nowhere to put.
 *
 * So a route is what the shell holds and the active section is derived from it.
 * That keeps the sidebar out of the business of knowing which screens are
 * reachable from where, and lets a screen take an argument without the
 * navigation model growing a row nobody designed.
 */
import { type NavSection, findSection } from './navigation.js';

export type Route =
  | { readonly kind: 'section'; readonly sectionId: string }
  /**
   * One analysis run, watched.
   *
   * `from` is the section that opened it — New Project after a submit, Library
   * when an in-flight run is clicked. That is the row the design leaves active
   * and the word its breadcrumb starts with, and carrying it means the shell
   * does not have to guess which of the two you came through.
   */
  | {
      readonly kind: 'analysis';
      readonly projectId: string;
      readonly jobId: string;
      readonly from: string;
    }
  /**
   * One clip, inspected.
   *
   * Like Analysis Progress this has no navigation row of its own: it is reached
   * from Results, keeps that row lit, and carries the arguments a section id has
   * nowhere to put. Unlike Analysis Progress it names three things, because
   * judging a clip means naming which recording and which candidate as well as
   * which project.
   */
  | {
      readonly kind: 'inspector';
      readonly projectId: string;
      readonly sourceId: string;
      readonly candidateId: string;
    };

export const DEFAULT_ROUTE: Route = { kind: 'section', sectionId: 'models' };

export function sectionRoute(sectionId: string): Route {
  return { kind: 'section', sectionId };
}

export interface Placement {
  /** The navigation row that reads as active. */
  readonly section: NavSection;
  /** The breadcrumb, outermost first. One part for a section screen. */
  readonly trail: readonly string[];
}

/** Which navigation row a route lights, and what its breadcrumb reads. */
export function placementOf(route: Route): Placement {
  if (route.kind === 'section') {
    const section = findSection(route.sectionId);
    return { section, trail: [section.breadcrumb] };
  }
  if (route.kind === 'inspector') {
    const section = findSection('results');
    return { section, trail: [section.breadcrumb, 'Clip'] };
  }
  const section = findSection(route.from);
  return { section, trail: [section.breadcrumb, 'Analysis'] };
}

export function inspectorRoute(projectId: string, sourceId: string, candidateId: string): Route {
  return { kind: 'inspector', projectId, sourceId, candidateId };
}
