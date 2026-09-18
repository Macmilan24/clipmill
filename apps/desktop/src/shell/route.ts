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

/**
 * One clip, named by everything a screen needs to open it.
 *
 * The editor and the export used to open "the newest document of the newest
 * project", which is right for one project with one approval and wrong the
 * moment a second of either exists: approving a clip in an older project
 * opened another project's edit. So the identity travels. The document is what
 * is opened; the project scopes every call about it; the source is where its
 * proxy and face tracks come from; the candidate and the run are which clip of
 * which analysis it was cut from, kept so a screen can say so and so a re-run
 * that renumbered the candidates cannot quietly swap the clip underneath.
 */
export interface ClipRef {
  readonly projectId: string;
  readonly docId: string;
  readonly sourceId: string;
  /** Absent for a document handed in whole rather than directed from a clip. */
  readonly candidateId?: string;
  /** The analysis run the clip came out of, when the opener knew it. */
  readonly jobId?: string;
  /** What the breadcrumb should call the project and the clip. */
  readonly labels?: { readonly project?: string; readonly clip?: string };
}

export type Route =
  /**
   * A section, and optionally the project it was opened for.
   *
   * Results is the reason the argument exists. Opening a finished project from
   * the Library used to navigate to the section and drop which project was
   * clicked, so the screen fell back to the newest one and an editor who chose
   * a recording was shown a different recording. A section id has nowhere to
   * put that, so it goes here.
   */
  | {
      readonly kind: 'section';
      readonly sectionId: string;
      readonly projectId?: string;
      readonly sourceId?: string;
      readonly jobId?: string;
    }
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
      /**
       * What the breadcrumb should call the project and the clip.
       *
       * Ids are what the route needs to work; names are what a person needs to
       * read it. The screen that opens the inspector already has both, so it
       * hands the names along rather than making the top bar look them up.
       */
      readonly labels?: { readonly project?: string; readonly clip?: string };
      /**
       * The analysis run whose candidate this is. Absent when the opener did
       * not know — the board then shows the newest finished run of the source.
       */
      readonly jobId?: string;
    }
  /**
   * One clip, in the editor or on the export screen.
   *
   * Both answer to their own navigation row, so unlike the two above they are
   * not a section with a hidden argument: the row is lit and the breadcrumb
   * names the clip. Reaching the row with no clip open is the plain section
   * route, and the screen says what to do about that.
   */
  | { readonly kind: 'editor'; readonly clip: ClipRef }
  | { readonly kind: 'export'; readonly clip: ClipRef };

export const DEFAULT_ROUTE: Route = { kind: 'section', sectionId: 'library' };

export function sectionRoute(sectionId: string, projectId?: string): Route {
  return projectId === undefined
    ? { kind: 'section', sectionId }
    : { kind: 'section', sectionId, projectId };
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
    return { section: findSection('results'), trail: clipTrail('results', route.labels) };
  }
  if (route.kind === 'editor' || route.kind === 'export') {
    return { section: findSection(route.kind), trail: clipTrail(route.kind, route.clip.labels) };
  }
  const section = findSection(route.from);
  return { section, trail: [section.breadcrumb, 'Analysis'] };
}

/** Section, then the project if it was named, then the clip. */
function clipTrail(
  sectionId: string,
  labels: { readonly project?: string; readonly clip?: string } | undefined,
): readonly string[] {
  const section = findSection(sectionId);
  const trail = [section.breadcrumb];
  if (labels?.project) {
    trail.push(labels.project);
  }
  trail.push(labels?.clip ?? 'Clip');
  return trail;
}

export function inspectorRoute(
  projectId: string,
  sourceId: string,
  candidateId: string,
  labels?: { readonly project?: string; readonly clip?: string },
  jobId?: string,
): Route {
  return {
    kind: 'inspector',
    projectId,
    sourceId,
    candidateId,
    ...(labels ? { labels } : {}),
    ...(jobId ? { jobId } : {}),
  };
}

export function editorRoute(clip: ClipRef): Route {
  return { kind: 'editor', clip };
}

export function exportRoute(clip: ClipRef): Route {
  return { kind: 'export', clip };
}

/** The clip a route is about, when it is about one. */
export function clipOf(route: Route): ClipRef | null {
  return route.kind === 'editor' || route.kind === 'export' ? route.clip : null;
}

/** Return to the same recording and analysis, even when a newer run exists. */
export function resultsRouteFor(route: Route): Route {
  const selection = route.kind === 'editor' || route.kind === 'export' ? route.clip : route;
  return {
    kind: 'section',
    sectionId: 'results',
    ...('projectId' in selection && selection.projectId ? { projectId: selection.projectId } : {}),
    ...('sourceId' in selection && selection.sourceId ? { sourceId: selection.sourceId } : {}),
    ...('jobId' in selection && selection.jobId ? { jobId: selection.jobId } : {}),
  };
}
