import { invoke } from '@tauri-apps/api/core';

export interface RecordingRun {
  readonly projectId: string;
  readonly sourceId: string;
  readonly jobId: string;
}

export interface MindApi {
  /**
   * The id each of a run's planned clips is stored under, by plan id: where
   * its decisions and its edit are kept. The engine derives it from the plan,
   * so it is asked for rather than derived here too.
   */
  planClipIds?(run: RecordingRun): Promise<Readonly<Record<string, string>>>;
}

export const mindApi: Required<MindApi> = {
  planClipIds(run) {
    return invoke<Record<string, string>>('plan_clip_ids', { ...run });
  },
};
