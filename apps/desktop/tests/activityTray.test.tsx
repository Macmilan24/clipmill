/**
 * The activity tray: one row for each long piece of work running now, with
 * how far it has come, leading to where it is shown in full.
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { JobState, TaskState } from '@clipmill/contracts';
import { describe, expect, it, vi } from 'vitest';

import type { Job, Task } from '../src/daemon/client.js';
import type { Component } from '../src/daemon/components.js';
import { type ActivityItem, engineItems, runItems } from '../src/shell/activity.js';
import { ActivityTray } from '../src/shell/ActivityTray.js';
import { announceFinished } from '../src/shell/announce.js';
import { installedLibrary, libraryModel } from './support/models.js';

function task(outputKind: string, state: TaskState, progress?: Task['progress']): Task {
  return {
    taskId: `${outputKind}-task`,
    kind: outputKind,
    outputKind,
    state,
    attempt: 1,
    maxAttempts: 3,
    waitReason: '',
    outputArtifactId: '',
    ...(progress ? { progress } : {}),
  };
}

function run(jobId: string, kind: string, tasks: readonly Task[]): Job {
  return {
    jobId,
    projectId: 'prj',
    sourceId: 'src',
    kind,
    state: JobState.RUNNING,
    createdUnixMillis: 1,
    updatedUnixMillis: 2,
    tasks,
    outputArtifactIds: [],
    failureClass: 0,
    failureDetail: '',
  };
}

function part(name: string, title: string, state: Component['state'], detail: string): Component {
  return {
    name,
    title,
    family: name,
    state,
    detail,
    installedBytes: 0,
    downloadBytes: 0,
    process: 'stopped',
    restarts: 0,
    logPath: '',
    tool: false,
  };
}

describe('the activity tray', () => {
  it('says what each run is doing and how far it has come', () => {
    const items = runItems(
      [
        run('analysis', 'analyze-source', [
          task('evidence.source_map.v1', TaskState.SUCCEEDED),
          task('speech.asr.v1', TaskState.RUNNING),
          task('speech.transcript.v1', TaskState.PLANNED),
        ]),
        run('export', 'export-clip', [
          task('render.clip.v1', TaskState.RUNNING, { unit: 'frames', done: 150, total: 600 }),
        ]),
      ],
      new Map([['prj', 'Episode 12']]),
    );
    expect(items.map(({ title, detail, fraction }) => ({ title, detail, fraction }))).toEqual([
      {
        title: 'Analysing Episode 12',
        detail: 'Recognise speech · 1 of 3 steps',
        fraction: 1 / 3,
      },
      { title: 'Exporting from Episode 12', detail: '150 of 600 frames', fraction: 0.25 },
    ]);
    expect(items[0]!.target).toEqual({ kind: 'analysis', projectId: 'prj', jobId: 'analysis' });
  });

  it('lists the engine’s downloads, installs, imports and uploads, and nothing finished', () => {
    const library = installedLibrary();
    const items = engineItems({
      models: {
        ...library,
        models: [
          ...library.models,
          libraryModel('big', {
            title: 'Big model',
            download: {
              state: 'downloading',
              receivedBytes: 50,
              totalBytes: 200,
              currentFile: 'weights.bin',
              error: '',
              updatedUnixMillis: 1,
            },
          }),
        ],
      },
      components: {
        managed: true,
        pythonVersion: '3.12',
        unavailable: '',
        parts: [
          part('asr', 'Speech recognition', 'installing', 'Downloading its packages'),
          part('vad', 'Speech detection', 'installed', ''),
        ],
      },
      imports: [],
      uploads: [],
    });
    expect(items.map((item) => [item.kind, item.title, item.fraction])).toEqual([
      ['download', 'Big model', 0.25],
      ['install', 'Installing Speech recognition', null],
    ]);
  });

  it('counts what runs and leads to it', () => {
    const onOpen = vi.fn();
    const onOpenChange = vi.fn();
    const item: ActivityItem = {
      id: 'job:a',
      kind: 'analysis',
      title: 'Analysing Episode 12',
      detail: 'Recognise speech · 1 of 3 steps',
      fraction: 1 / 3,
      target: { kind: 'analysis', projectId: 'prj', jobId: 'a' },
    };
    render(<ActivityTray items={[item]} open onOpenChange={onOpenChange} onOpen={onOpen} />);
    expect(screen.getByRole('button', { name: 'Activity: 1 running' })).toBeTruthy();
    fireEvent.click(screen.getByText('Analysing Episode 12'));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onOpen).toHaveBeenCalledWith(item.target);
  });

  it('tells a person who looked away once per run', () => {
    const focus = vi.spyOn(document, 'hasFocus').mockReturnValue(false);
    document.title = 'ClipMill';
    announceFinished('job-1', 'Clips ready · Episode 12');
    expect(document.title).toBe('Clips ready · Episode 12');
    document.title = 'ClipMill';
    announceFinished('job-1', 'Clips ready · Episode 12');
    expect(document.title).toBe('ClipMill');
    focus.mockRestore();
  });
});
