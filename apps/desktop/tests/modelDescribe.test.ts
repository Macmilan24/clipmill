import { describe, expect, it } from 'vitest';

import { validModelName } from '../src/models/AddModelSheet.js';
import {
  fitWarning,
  jobReason,
  jobSentence,
  modelFacts,
  percentOf,
  progressText,
  recommendedTitles,
  runsOn,
} from '../src/models/describe.js';
import { freshLibrary, libraryModel, modelJob } from './support/models.js';

const GIB = 1024 ** 3;

describe('model library wording', () => {
  it('says where a model runs in words, not backend names', () => {
    expect(runsOn(libraryModel('a', { backend: 'mlx' }))).toBe('Apple silicon GPU');
    expect(runsOn(libraryModel('b', { backend: 'onnx-cpu' }))).toBe('CPU');
    expect(runsOn(libraryModel('c', { backend: 'cuda' }))).toBe('cuda');
  });

  it('puts the size, the memory, the hardware and the licence on one line', () => {
    const model = libraryModel('whisper-base', {
      downloadBytes: 147_951_465,
      memoryBytes: 416_386_921,
      licenseSpdx: 'MIT',
    });
    expect(modelFacts(model)).toBe('141 MB download · needs about 397 MB of memory · CPU · MIT');
  });

  it('warns on every scale of memory and stays quiet when a model fits', () => {
    const library = freshLibrary({ memoryTotalBytes: 16 * GIB, memoryBudgetBytes: 12 * GIB });
    expect(fitWarning(libraryModel('a', { memoryFit: 'fits' }), library)).toBeNull();
    expect(
      fitWarning(libraryModel('b', { memoryFit: 'tight', memoryBytes: 14 * GIB }), library),
    ).toEqual({
      tone: 'warning',
      text: 'Needs about 14 GB of memory; analysis can use 12 GB on this computer. It may run slowly or stop.',
    });
    expect(
      fitWarning(libraryModel('c', { memoryFit: 'too_large', memoryBytes: 26 * GIB }), library),
    ).toEqual({
      tone: 'danger',
      text: 'Needs about 26 GB of memory; this computer has 16 GB. It is unlikely to run here.',
    });
    const unmeasured = freshLibrary();
    const { memoryTotalBytes: _total, memoryBudgetBytes: _budget, ...rest } = unmeasured;
    expect(
      fitWarning(libraryModel('d', { memoryFit: 'too_large', memoryBytes: 26 * GIB }), rest)?.text,
    ).toBe('Needs about 26 GB of memory, more than this computer has. It is unlikely to run here.');
  });

  it('names why a job runs its model', () => {
    expect(jobReason(modelJob('asr', { selectedBy: 'chosen' }))).toBe('Your choice');
    expect(jobReason(modelJob('asr', { selectedBy: 'measured' }))).toBe('Fastest measured');
    expect(jobReason(modelJob('asr', { selectedBy: 'installed_fallback' }))).toBe('Stand-in');
    expect(jobReason(modelJob('asr', { selectedBy: 'portable' }))).toBe('Automatic');
    expect(jobReason(modelJob('asr', { selectedBy: 'unavailable' }))).toBe('No model');
  });

  it('tells the person what the next analysis will use, and what it still needs', () => {
    const base = libraryModel('whisper-base', { title: 'Whisper Base' });
    expect(jobSentence(modelJob('asr', { selectedBy: 'portable' }), base)).toBe(
      'Analyses need Whisper Base. Download it to run this job.',
    );
    expect(
      jobSentence(modelJob('asr', { selectedBy: 'portable' }), {
        ...base,
        installState: 'installed',
      }),
    ).toBe('Analyses use Whisper Base.');
    expect(jobSentence(modelJob('asr', { selectedBy: 'installed_fallback' }), base)).toBe(
      'Analyses use Whisper Base, because the default is not installed.',
    );
    expect(jobSentence(modelJob('asr'), undefined)).toBe('No model is registered for this job.');
  });

  it('reports progress without inventing a total it was not given', () => {
    const download = {
      state: 'downloading' as const,
      receivedBytes: 512 * 1024 ** 2,
      totalBytes: 2 * GIB,
      currentFile: 'model.safetensors',
      error: '',
      updatedUnixMillis: 1,
    };
    expect(percentOf(download)).toBe(25);
    expect(progressText(download)).toBe('Downloading model.safetensors · 512 MB of 2 GB');
    expect(percentOf({ ...download, totalBytes: 0 })).toBeUndefined();
    expect(progressText({ ...download, state: 'queued' })).toBe('Waiting to start');
    expect(progressText({ ...download, state: 'verifying', currentFile: '' })).toBe(
      'Checking files',
    );
    expect(progressText({ ...download, state: 'cancelled' })).toBe('Paused at 512 MB of 2 GB');
    expect(progressText({ ...download, state: 'failed', error: 'It broke.' })).toBe('It broke.');
  });

  it('lists the recommended downloads as a sentence', () => {
    expect(recommendedTitles(freshLibrary())).toBe('Qwen3.5 9B, Whisper Base and Silero VAD');
    expect(recommendedTitles(freshLibrary({ recommendedMissing: ['whisper-base'] }))).toBe(
      'Whisper Base',
    );
  });

  it('holds a new model name to the registry rules before the daemon has to', () => {
    for (const good of ['whisper-small', 'qwen3.5-4b-4bit', 'a']) {
      expect(validModelName(good)).toBe(true);
    }
    for (const bad of ['', 'Upper', 'has space', '-lead', 'trail-', 'a..b', 'x'.repeat(65)]) {
      expect(validModelName(bad)).toBe(false);
    }
  });
});
