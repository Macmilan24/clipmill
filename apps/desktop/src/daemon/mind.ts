import { invoke } from '@tauri-apps/api/core';

export interface Moment {
  readonly units: string;
  readonly start: number;
  readonly end: number;
  readonly text: string;
  readonly label?: string;
}

export interface RecordingRun {
  readonly projectId: string;
  readonly sourceId: string;
  readonly jobId: string;
}

export interface BriefLimits {
  readonly minSeconds?: number;
  readonly maxSeconds?: number;
  readonly count?: number;
}

export interface PlanDecision {
  readonly decision: string;
  readonly reason: string;
}

export interface MindApi {
  findMoments?(run: RecordingRun, query: string): Promise<readonly Moment[]>;
  planDecision?(
    run: RecordingRun,
    clipId: string,
    decision?: 'kept' | 'rejected' | 'undecided',
    reason?: string,
  ): Promise<PlanDecision>;
  reprompt?(run: RecordingRun, prompt: string, limits?: BriefLimits): Promise<string>;
}

export const mindApi: Required<MindApi> = {
  async findMoments(run, query) {
    const result = await invoke<{ moments: Moment[] }>('find_moments', { ...run, query });
    return result.moments;
  },
  planDecision(run, clipId, decision, reason) {
    return invoke<PlanDecision>('plan_decision', {
      ...run,
      clipId,
      decision: decision ?? null,
      reason: reason ?? null,
    });
  },
  reprompt(run, prompt, limits) {
    return invoke<string>('reprompt', { ...run, prompt, limits: limits ?? null });
  },
};
