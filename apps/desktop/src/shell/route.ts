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

export function placementOf(route: Route): Placement {
  const section = findSection(route.kind === 'section' ? route.sectionId : route.from);
  return {
    section,
    trail: route.kind === 'section' ? [section.breadcrumb] : [section.breadcrumb, 'Analysis'],
  };
}
