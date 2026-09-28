/**
 * Routes carry screen arguments independently of sidebar selection.
 * Analysis Progress has no navigation row and retains its originating section
 * while identifying the run and breadcrumb to display.
 */
import { type NavSection, findSection } from './navigation.js';

/**
 * Clip identity shared by editor and export routes.
 * The project scopes requests; the source identifies media; the run and candidate
 * identify the analysis result so re-analysis cannot substitute another clip.
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
   * Section route with an optional project so Results preserves the Library selection.
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
   * Inspector route carrying project, source, and candidate identity while
   * keeping Results selected in the sidebar.
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
  | { readonly kind: 'editor'; readonly clip: ClipRef; readonly focus?: EditorFocus }
  | { readonly kind: 'export'; readonly clip: ClipRef };

/**
 * What the editor should open on, when the screen that sent a person there
 * knows.
 *
 * The export strip names the caption it refused; a button that then dropped
 * the person at the top of the editor would leave them to find it again by
 * its timestamp. So the route carries the caption track and the cue, and the
 * editor lands on that cue with the playhead there. Only captions take an
 * argument today; the shape is a union so the next panel that needs one can
 * add its own without the editor guessing from a string.
 */
export type EditorFocus = {
  readonly panel: 'captions';
  /** The sidecar grouping or the burned-in one — two lists of the same words. */
  readonly track: 'reading' | 'on-screen';
  /** The cue to select. Absent lands on the track's first problem, if any. */
  readonly cueId?: string;
};

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

export function editorRoute(clip: ClipRef, focus?: EditorFocus): Route {
  return focus ? { kind: 'editor', clip, focus } : { kind: 'editor', clip };
}

/** What the editor was asked to open on, when the route said. */
export function focusOf(route: Route): EditorFocus | null {
  return route.kind === 'editor' ? (route.focus ?? null) : null;
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
