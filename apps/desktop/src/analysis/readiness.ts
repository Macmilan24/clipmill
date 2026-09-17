/**
 * What an analysis would need, and whether it is here.
 *
 * A missing weight file or a worker fleet nobody started used to show as a
 * stage sitting planned forever, with a spinner beside it and nothing to say.
 * The daemon now answers the question directly — per stage, with the command
 * that fixes it — and two screens ask: New Project before a run is submitted,
 * so the answer is on screen before the wait would be, and Analysis Progress
 * while a stage waits, so the wait says what it is waiting for.
 *
 * The two shortfalls are different and are kept apart. A model that is not
 * installed can never be, however long the run waits, so it blocks the
 * submission. A worker that is not connected may be started at any moment,
 * and the daemon holds the task until it is; that is shown, not blocked.
 */
import { TaskState } from '@clipmill/contracts';
import { useCallback, useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { Job, Readiness, StageReadiness } from '../daemon/client.js';

/** How often readiness is re-read while a screen is waiting on it. */
const POLL_MILLIS = 3_000;

export interface ReadinessState {
  readonly readiness: Readiness | null;
  /** Why nothing could be read, when nothing could. */
  readonly problem: string | null;
  readonly refresh: () => void;
}

/** The stages whose model is not installed: a run that cannot succeed. */
export function missingModels(readiness: Readiness | null): readonly StageReadiness[] {
  return readiness?.stages.filter((stage) => !stage.modelPresent) ?? [];
}

/** The stages no connected worker serves: a run that would wait. */
export function missingWorkers(readiness: Readiness | null): readonly StageReadiness[] {
  return readiness?.stages.filter((stage) => stage.modelPresent && !stage.workerPresent) ?? [];
}

/**
 * Why a run could not be submitted, or null when it could.
 *
 * Only the shortfalls no wait can cure: the decoder and the weights.
 */
export function submissionBlocker(readiness: Readiness | null): string | null {
  if (readiness === null) {
    return null;
  }
  if (!readiness.decoderPresent) {
    return `The pinned decoder is not at ${readiness.decoderPath || 'its path'}; run \`just setup\` to fetch it.`;
  }
  const models = missingModels(readiness);
  if (models.length > 0) {
    const names = models.map((stage) => stage.model).join(', ');
    return `${models.length === 1 ? 'A model is' : 'Models are'} not installed (${names}); run \`tools/fetch-models.sh\` to fetch the pinned weights.`;
  }
  return null;
}

/**
 * Why a task is waiting, when readiness knows.
 *
 * For a task that is planned or admitted and not running, whose kind a stage
 * row says is not ready: the row's remedy. Null for a task that is running,
 * finished, or waiting on something readiness cannot see (a dependency).
 */
export function waitingReason(
  task: { readonly kind: string; readonly state: TaskState },
  readiness: Readiness | null,
): string | null {
  if (task.state !== TaskState.PLANNED && task.state !== TaskState.ADMITTED) {
    return null;
  }
  const stage = readiness?.stages.find((candidate) => candidate.stage === task.kind);
  return stage && !stage.ready ? stage.remedy : null;
}

/**
 * The reason each waiting task in a job is waiting, by the artifact kind it
 * would publish — which is how the progress screen keys its rows.
 */
export function waitingReasons(
  job: Job | null,
  readiness: Readiness | null,
): ReadonlyMap<string, string> {
  const reasons = new Map<string, string>();
  for (const task of job?.tasks ?? []) {
    const reason = waitingReason(task, readiness);
    if (reason !== null && !reasons.has(task.outputKind)) {
      reasons.set(task.outputKind, reason);
    }
  }
  return reasons;
}

/**
 * Read readiness, and keep reading it while asked to.
 *
 * `poll` is on while a screen is waiting on the answer changing — a run that
 * has stages waiting, a submission held back — and off once it cannot change
 * anything, so a finished run does not go on asking.
 */
export function useReadiness(poll: boolean, api: ShellApi = daemonApi): ReadinessState {
  const [readiness, setReadiness] = useState<Readiness | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [tick, setTick] = useState(0);

  const refresh = useCallback(() => setTick((count) => count + 1), []);

  useEffect(() => {
    let live = true;
    let timer: ReturnType<typeof setTimeout> | null = null;
    const read = async () => {
      try {
        const answer = await api.fetchReadiness();
        if (live) {
          setReadiness(answer);
          setProblem(null);
        }
      } catch (error) {
        if (live) {
          setProblem((error as Error).message);
        }
      }
      if (live && poll) {
        timer = setTimeout(() => void read(), POLL_MILLIS);
      }
    };
    void read();
    return () => {
      live = false;
      if (timer !== null) {
        clearTimeout(timer);
      }
    };
  }, [api, poll, tick]);

  return { readiness, problem, refresh };
}
