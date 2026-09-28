/**
 * Components for the preview harness: `?setup=fresh|update|failed` on New
 * Project and Models. Installs advance one step each time they are read, as
 * the daemon's would between two polls.
 */
import type { Component, Components, ComponentsApi } from '../src/daemon/components.js';

const MIB = 1024 ** 2;

const PARTS: readonly Omit<
  Component,
  'state' | 'detail' | 'process' | 'restarts' | 'installedBytes'
>[] = [
  {
    name: 'vad',
    title: 'Speech detection',
    family: 'speech-vad',
    downloadBytes: 61 * MIB,
    logPath: '',
  },
  {
    name: 'asr-whispercpp',
    title: 'Transcription',
    family: 'speech-asr',
    downloadBytes: 74 * MIB,
    logPath: '',
  },
  {
    name: 'align',
    title: 'Word timing',
    family: 'speech-align',
    downloadBytes: 61 * MIB,
    logPath: '',
  },
  {
    name: 'speakers',
    title: 'Telling voices apart',
    family: 'speech-speakers',
    downloadBytes: 61 * MIB,
    logPath: '',
  },
  {
    name: 'shots',
    title: 'Shot detection',
    family: 'detect-shots',
    downloadBytes: 112 * MIB,
    logPath: '',
  },
  {
    name: 'faces',
    title: 'Face tracking',
    family: 'detect-faces',
    downloadBytes: 61 * MIB,
    logPath: '',
  },
].map((part) => ({
  ...part,
  logPath: `/Users/you/Library/Application Support/dev.clipmill.ClipMill/logs/workers/${part.name}.log`,
}));

const STEPS = [
  'Getting Python 3.12.13',
  'Preparing its environment',
  'Downloading its packages',
  "Adding ClipMill's worker",
  'Checking that it starts',
];

type Scenario = 'fresh' | 'update' | 'failed' | 'installed';

export function setupScenario(): Scenario | null {
  const value = new URLSearchParams(location.search).get('setup');
  return value === 'fresh' || value === 'update' || value === 'failed' || value === 'installed'
    ? value
    : null;
}

export function previewComponentsApi(scenario: Scenario): ComponentsApi {
  let parts: Component[] = PARTS.map((part, index) => {
    const installed = scenario === 'installed' || scenario === 'update';
    const failed = scenario === 'failed' && index === 1;
    return {
      ...part,
      state: failed
        ? 'failed'
        : scenario === 'update'
          ? 'outdated'
          : installed || (scenario === 'failed' && index === 0)
            ? 'installed'
            : 'missing',
      detail: failed
        ? 'No solution found when resolving dependencies: pywhispercpp==1.5.0 has no wheel for this platform (the full output is in …/logs/engine-install.log)'
        : '',
      installedBytes: installed ? part.downloadBytes * 3 : 0,
      process: installed || (scenario === 'failed' && index === 0) ? 'running' : 'stopped',
      restarts: 0,
    };
  });
  // Each read moves the install along: one step of the part in progress.
  let step = 0;
  const read = (): Components => {
    const current = parts.findIndex((part) => part.state === 'installing');
    if (current >= 0) {
      step += 1;
      if (step >= STEPS.length) {
        step = 0;
        parts = parts.map((part, index) =>
          index === current
            ? {
                ...part,
                state: 'installed',
                detail: '',
                process: 'running',
                installedBytes: part.downloadBytes * 3,
              }
            : part,
        );
        const next = parts.findIndex((part) => part.state === 'queued');
        if (next >= 0)
          parts = parts.map((part, index) =>
            index === next ? { ...part, state: 'installing', detail: STEPS[0] ?? '' } : part,
          );
      } else {
        parts = parts.map((part, index) =>
          index === current ? { ...part, detail: STEPS[step] ?? '' } : part,
        );
      }
    }
    return { managed: true, parts, pythonVersion: '3.12.13', unavailable: '' };
  };
  return {
    listComponents: () => Promise.resolve(read()),
    installComponents: (names) => {
      let first = true;
      parts = parts.map((part) => {
        const wanted =
          names.length === 0
            ? ['missing', 'outdated', 'failed'].includes(part.state)
            : names.includes(part.name);
        if (!wanted) return part;
        const state = first ? 'installing' : 'queued';
        first = false;
        return { ...part, state, detail: state === 'installing' ? (STEPS[0] ?? '') : '' };
      });
      step = 0;
      return Promise.resolve(read());
    },
    cancelComponentInstall: () => {
      parts = parts.map((part) =>
        ['queued', 'installing'].includes(part.state)
          ? { ...part, state: 'missing', detail: '' }
          : part,
      );
      return Promise.resolve(read());
    },
  };
}
