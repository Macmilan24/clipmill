/**
 * What the shell remembers between launches: where it was, and which clip.
 *
 * Reopening the app used to land on Models & Device with no memory of the clip
 * somebody had been editing, and the Editor row then opened whichever document
 * the daemon had written last. Both are answered here. The route is put back
 * as it was, and the last clip opened in the editor or on the export screen is
 * what those rows open when reached from the sidebar with nothing named.
 *
 * Only the identity is remembered — ids and the labels that make them
 * readable — never a document or a plan. Those are the daemon's, and a copy of
 * them here would be a copy that could disagree with it. What comes back is
 * checked field by field before it is believed: this is a string in local
 * storage, and a route with a missing id would send a screen to ask the daemon
 * about nothing.
 */
import { type ClipRef, DEFAULT_ROUTE, type Route } from './route.js';

/** Where it is kept. The suffix is the shape's version, not the app's. */
export const MEMORY_KEY = 'clipmill.shell.v1';

export interface ShellMemory {
  readonly route: Route;
  /** The clip the editor and the export open when reached with none named. */
  readonly clip: ClipRef | null;
}

/** The narrow slice of `Storage` the shell needs, so a test can hand in a map. */
export interface KeyValueStore {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function remember(store: KeyValueStore | null, memory: ShellMemory): void {
  if (!store) {
    return;
  }
  try {
    store.setItem(MEMORY_KEY, JSON.stringify(memory));
  } catch {
    // Storage that is full or forbidden loses the memory, not the session.
  }
}

/** What was remembered, or the defaults when nothing usable was. */
export function recall(store: KeyValueStore | null): ShellMemory {
  const fallback: ShellMemory = { route: DEFAULT_ROUTE, clip: null };
  if (!store) {
    return fallback;
  }
  let parsed: unknown;
  try {
    const raw = store.getItem(MEMORY_KEY);
    if (raw === null) {
      return fallback;
    }
    parsed = JSON.parse(raw);
  } catch {
    return fallback;
  }
  if (!isRecord(parsed)) {
    return fallback;
  }
  const clip = clipRef(parsed['clip']);
  const route = routeOf(parsed['route']);
  return { route: route ?? fallback.route, clip };
}

function routeOf(value: unknown): Route | null {
  if (!isRecord(value)) {
    return null;
  }
  switch (value['kind']) {
    case 'section': {
      if (!isText(value['sectionId'])) {
        return null;
      }
      return {
        kind: 'section',
        sectionId: value['sectionId'],
        ...(isText(value['projectId']) ? { projectId: value['projectId'] } : {}),
        ...(isText(value['sourceId']) ? { sourceId: value['sourceId'] } : {}),
        ...(isText(value['jobId']) ? { jobId: value['jobId'] } : {}),
      };
    }
    case 'analysis': {
      if (!isText(value['projectId']) || !isText(value['jobId']) || !isText(value['from'])) {
        return null;
      }
      return {
        kind: 'analysis',
        projectId: value['projectId'],
        jobId: value['jobId'],
        from: value['from'],
      };
    }
    case 'inspector': {
      if (
        !isText(value['projectId']) ||
        !isText(value['sourceId']) ||
        !isText(value['candidateId'])
      ) {
        return null;
      }
      const labels = labelsOf(value['labels']);
      return {
        kind: 'inspector',
        projectId: value['projectId'],
        sourceId: value['sourceId'],
        candidateId: value['candidateId'],
        ...(labels ? { labels } : {}),
        ...(isText(value['jobId']) ? { jobId: value['jobId'] } : {}),
      };
    }
    case 'editor':
    case 'export': {
      const clip = clipRef(value['clip']);
      return clip ? { kind: value['kind'], clip } : null;
    }
    default:
      return null;
  }
}

function clipRef(value: unknown): ClipRef | null {
  if (
    !isRecord(value) ||
    !isText(value['projectId']) ||
    !isText(value['docId']) ||
    !isText(value['sourceId'])
  ) {
    return null;
  }
  const labels = labelsOf(value['labels']);
  return {
    projectId: value['projectId'],
    docId: value['docId'],
    sourceId: value['sourceId'],
    ...(isText(value['candidateId']) ? { candidateId: value['candidateId'] } : {}),
    ...(isText(value['jobId']) ? { jobId: value['jobId'] } : {}),
    ...(labels ? { labels } : {}),
  };
}

function labelsOf(
  value: unknown,
): { readonly project?: string; readonly clip?: string } | undefined {
  if (!isRecord(value)) {
    return undefined;
  }
  const labels = {
    ...(isText(value['project']) ? { project: value['project'] } : {}),
    ...(isText(value['clip']) ? { clip: value['clip'] } : {}),
  };
  return Object.keys(labels).length > 0 ? labels : undefined;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isText(value: unknown): value is string {
  return typeof value === 'string' && value.length > 0;
}
