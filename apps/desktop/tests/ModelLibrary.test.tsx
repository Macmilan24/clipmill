import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { ModelLibraryPanel } from '../src/models/ModelLibraryPanel.js';
import type { LibraryModel, ModelLibrary } from '../src/daemon/models.js';
import { emptyWorld, fakeApi, readiness, stageReadiness } from './support/library.js';
import {
  FakeModelLibrary,
  freshLibrary,
  installedLibrary,
  libraryModel,
} from './support/models.js';

const GIB = 1024 ** 3;

function show(library: ModelLibrary, world = emptyWorld()) {
  const models = new FakeModelLibrary(library);
  const api = fakeApi({ ...world, models });
  render(<ModelLibraryPanel api={api} />);
  return models;
}

function withModel(library: ModelLibrary, name: string, change: Partial<LibraryModel>) {
  return {
    ...library,
    models: library.models.map((model) => (model.name === name ? { ...model, ...change } : model)),
  };
}

function row(title: string): HTMLElement {
  return screen.getByRole('listitem', { name: title });
}

describe('the model library', () => {
  it('offers a fresh install everything analysis needs in one download', async () => {
    const models = show(freshLibrary());

    const setup = await screen.findByTestId('model-setup');
    expect(within(setup).getByText(/Qwen3.5 9B, Whisper Base and Silero VAD/)).toBeTruthy();
    expect(within(setup).getByText(/Downloads from huggingface.co/)).toBeTruthy();
    fireEvent.click(within(setup).getByRole('button', { name: /Download all · 5.7 GB/ }));

    await waitFor(() => {
      expect(models.asked('download')).toEqual([
        ['download', ['qwen3-5-editorial-mlx', 'whisper-base', 'silero-vad']],
      ]);
    });
    expect(await within(row('Whisper Base')).findByText('Waiting to start')).toBeTruthy();
  });

  it('says how many models are here without inventing a count it could not read', async () => {
    const models = new FakeModelLibrary();
    models.refusals.set('list', 'Disconnected');
    render(<ModelLibraryPanel api={fakeApi({ ...emptyWorld(), models })} />);
    expect(await screen.findByText(/The model library is unavailable. Disconnected/)).toBeTruthy();
    expect(screen.queryByText(/installed ·/)).toBeNull();
  });

  it('lists every model for its job with what it costs and where it runs', async () => {
    show(installedLibrary());
    const transcription = await screen.findByRole('region', { name: 'Transcription' });
    expect(within(transcription).getByText('Whisper Base')).toBeTruthy();
    expect(within(transcription).getByText('Whisper Large v3 Turbo')).toBeTruthy();
    expect(within(row('Whisper Base')).getByText('In use')).toBeTruthy();
    expect(within(row('Whisper Large v3 Turbo')).getByText(/1.5 GB download/)).toBeTruthy();
    expect(within(row('Whisper Large v3 Turbo')).getByText(/CPU · MIT/)).toBeTruthy();
    expect(screen.getByText(/3 of 4 installed/)).toBeTruthy();
    expect(screen.queryByTestId('model-setup')).toBeNull();
  });

  it('shows download progress as the daemon reports it, and cancels', async () => {
    const models = show(
      withModel(installedLibrary(), 'whisper-large-v3-turbo', {
        download: {
          state: 'downloading',
          receivedBytes: 812_277_638,
          totalBytes: 1_624_555_275,
          currentFile: 'ggml-large-v3-turbo.bin',
          error: '',
          updatedUnixMillis: 1,
        },
      }),
    );
    const large = await screen.findByRole('listitem', { name: 'Whisper Large v3 Turbo' });
    expect(
      within(large).getByText(/Downloading ggml-large-v3-turbo.bin · 775 MB of 1.5 GB/),
    ).toBeTruthy();
    expect(within(large).getByText('50%')).toBeTruthy();
    fireEvent.click(
      within(large).getByRole('button', { name: 'Cancel Whisper Large v3 Turbo download' }),
    );
    await waitFor(() => {
      expect(models.asked('cancel')).toEqual([['cancel', 'whisper-large-v3-turbo']]);
    });
    expect(await within(row('Whisper Large v3 Turbo')).findByText(/Paused at/)).toBeTruthy();
    expect(
      within(row('Whisper Large v3 Turbo')).getByRole('button', { name: /Resume/ }),
    ).toBeTruthy();
  });

  it('lets a person choose an installed model for a job, and hand it back to automatic', async () => {
    const models = show(
      withModel(installedLibrary(), 'whisper-large-v3-turbo', {
        installState: 'installed',
        installedBytes: 1_624_555_275,
      }),
    );
    const large = await screen.findByRole('listitem', { name: 'Whisper Large v3 Turbo' });
    fireEvent.click(
      within(large).getByRole('button', { name: 'Use Whisper Large v3 Turbo for transcription' }),
    );
    await waitFor(() => {
      expect(models.asked('choose')).toEqual([['choose', 'asr', 'whisper-large-v3-turbo']]);
    });
    const transcription = screen.getByRole('region', { name: 'Transcription' });
    expect(await within(transcription).findByText('Your choice')).toBeTruthy();
    fireEvent.click(within(transcription).getByRole('button', { name: 'Use automatic' }));
    await waitFor(() => {
      expect(models.asked('choose').at(-1)).toEqual(['choose', 'asr', '']);
    });
  });

  it('confirms a removal in place, naming what is lost, before asking the daemon', async () => {
    const models = show(installedLibrary());
    const base = await screen.findByRole('listitem', { name: 'Whisper Base' });
    fireEvent.click(within(base).getByRole('button', { name: 'Remove Whisper Base' }));
    const confirm = within(base).getByRole('group', { name: 'Remove Whisper Base' });
    expect(within(confirm).getByText(/141 MB of files are deleted/)).toBeTruthy();
    expect(within(confirm).getByText(/in use for transcription/)).toBeTruthy();
    expect(models.asked('remove')).toEqual([]);
    fireEvent.click(within(confirm).getByRole('button', { name: 'Remove' }));
    await waitFor(() => {
      expect(models.asked('remove')).toEqual([['remove', 'whisper-base']]);
    });
  });

  it("shows the daemon's refusal instead of pretending the removal happened", async () => {
    const models = show(installedLibrary());
    models.refusals.set(
      'remove',
      'An analysis that has not finished uses Whisper Base. Let it finish or cancel it, then remove the model.',
    );
    const base = await screen.findByRole('listitem', { name: 'Whisper Base' });
    fireEvent.click(within(base).getByRole('button', { name: 'Remove Whisper Base' }));
    fireEvent.click(within(base).getByRole('button', { name: 'Remove' }));
    expect(
      await screen.findByText(/An analysis that has not finished uses Whisper Base/),
    ).toBeTruthy();
    expect(within(row('Whisper Base')).getByText('In use')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByText(/An analysis that has not finished/)).toBeNull();
  });

  it('forgets a model the person added rather than only deleting its files', async () => {
    const library = installedLibrary();
    const models = show({
      ...library,
      models: [
        ...library.models,
        libraryModel('whisper-small', {
          title: 'Whisper small',
          custom: true,
          installState: 'installed',
          installedBytes: 487_601_967,
        }),
      ],
      jobs: library.jobs.map((job) =>
        job.capability === 'asr' ? { ...job, models: [...job.models, 'whisper-small'] } : job,
      ),
    });
    const small = await screen.findByRole('listitem', { name: 'Whisper small' });
    expect(within(small).getByText('Added by you')).toBeTruthy();
    fireEvent.click(within(small).getByRole('button', { name: 'Remove Whisper small' }));
    expect(within(small).getByText(/leaves the library/)).toBeTruthy();
    fireEvent.click(within(small).getByRole('button', { name: 'Remove' }));
    await waitFor(() => {
      expect(models.asked('forget')).toEqual([['forget', 'whisper-small']]);
    });
  });

  it('warns about a model too large for this computer without refusing to download it', async () => {
    show(
      withModel(
        withModel(installedLibrary(), 'whisper-large-v3-turbo', {
          memoryFit: 'too_large',
          memoryBytes: 26 * GIB,
        }),
        'qwen3-5-editorial-mlx',
        { memoryFit: 'tight' },
      ),
    );
    const large = await screen.findByRole('listitem', { name: 'Whisper Large v3 Turbo' });
    expect(
      within(large).getByText(
        /Needs about 26 GB of memory; this computer has 24 GB. It is unlikely to run here./,
      ),
    ).toBeTruthy();
    expect(
      within(large)
        .getByRole('button', { name: 'Download Whisper Large v3 Turbo' })
        .hasAttribute('disabled'),
    ).toBe(false);
    expect(
      within(row('Qwen3.5 9B')).getByText(/analysis can use 18 GB on this computer/),
    ).toBeTruthy();
  });

  it('says why a model cannot run here and does not offer the download', async () => {
    show(
      withModel(freshLibrary(), 'qwen3-5-editorial-mlx', {
        supported: false,
        unsupportedReason: 'Runs only on Macs with Apple silicon.',
      }),
    );
    const qwen = await screen.findByRole('listitem', { name: 'Qwen3.5 9B' });
    expect(within(qwen).getByText('Runs only on Macs with Apple silicon.')).toBeTruthy();
    expect(
      within(qwen).getByRole('button', { name: 'Download Qwen3.5 9B' }).hasAttribute('disabled'),
    ).toBe(true);
  });

  it('reports a failed download and offers to try again', async () => {
    const models = show(
      withModel(installedLibrary(), 'whisper-large-v3-turbo', {
        installState: 'partial',
        installedBytes: 0,
        download: {
          state: 'failed',
          receivedBytes: 0,
          totalBytes: 1_624_555_275,
          currentFile: 'ggml-large-v3-turbo.bin',
          error: 'ggml-large-v3-turbo.bin does not match its pinned SHA-256, so it was discarded.',
          updatedUnixMillis: 1,
        },
      }),
    );
    const large = await screen.findByRole('listitem', { name: 'Whisper Large v3 Turbo' });
    expect(within(large).getByRole('alert').textContent).toMatch(
      /does not match its pinned SHA-256/,
    );
    fireEvent.click(within(large).getByRole('button', { name: 'Retry Whisper Large v3 Turbo' }));
    await waitFor(() => {
      expect(models.asked('download')).toEqual([['download', ['whisper-large-v3-turbo']]]);
    });
  });

  it('warns beside a model whose worker is not running before it is chosen', async () => {
    const library = installedLibrary();
    show({
      ...library,
      models: [
        ...library.models,
        libraryModel('qwen3-asr-mlx', {
          title: 'Qwen3-ASR 1.7B',
          backend: 'mlx',
          installState: 'installed',
          installedBytes: 150 * 1024 ** 2,
          worker: 'speech-mlx',
          workerTitle: 'MLX speech worker',
          workerConnected: false,
        }),
      ],
      jobs: library.jobs.map((job) =>
        job.capability === 'asr' ? { ...job, models: [...job.models, 'qwen3-asr-mlx'] } : job,
      ),
    });
    const mlx = await screen.findByRole('listitem', { name: 'Qwen3-ASR 1.7B' });
    expect(
      within(mlx).getByText(
        "Runs in the MLX speech worker, which isn't running, so transcription would wait for it. Restart the workers before choosing it.",
      ),
    ).toBeTruthy();
    expect(
      within(row('Whisper Base')).queryByText(/isn't running/),
      'the model in use has a running worker',
    ).toBeNull();
  });

  it('names a worker problem under the job whose model is here, and ignores cloud routes', async () => {
    show(installedLibrary(), {
      ...emptyWorld(),
      readiness: readiness([
        stageReadiness('speech-asr', {
          capability: 'asr',
          workerPresent: false,
          remedy:
            'No worker is connected that runs speech-asr: start the workers with `just workers`.',
        }),
        stageReadiness('editorial-propose-cloud', {
          capability: 'editorial',
          workerPresent: false,
          remedy: 'Optional cloud worker is not running.',
        }),
      ]),
    });
    const transcription = await screen.findByRole('region', { name: 'Transcription' });
    expect(
      await within(transcription).findByText(/No worker is connected that runs speech-asr/),
    ).toBeTruthy();
    expect(screen.queryByText('Optional cloud worker is not running.')).toBeNull();
  });
});

describe('adding a model from Hugging Face', () => {
  it('shows what would be pinned, then pins exactly that', async () => {
    const models = show(installedLibrary());
    models.inspection = {
      repo: 'ggerganov/whisper.cpp',
      commit: '5359861c739e955e79d9a303bcbc70fb988958b1',
      capability: 'asr',
      license: 'mit',
      licenseSpdx: 'MIT',
      licenseAllowed: true,
      problem: '',
      weightChoices: [
        {
          path: 'ggml-small.bin',
          bytes: 487_601_967,
          suggestedName: 'whisper-small',
          suggestedTitle: 'Whisper small',
        },
        {
          path: 'ggml-medium.bin',
          bytes: 1_533_763_059,
          suggestedName: 'whisper-medium',
          suggestedTitle: 'Whisper medium',
        },
      ],
      files: [],
      suggestedName: 'whisper-small',
      suggestedTitle: 'Whisper small',
    };

    fireEvent.click(await screen.findByRole('button', { name: /Add from Hugging Face/ }));
    fireEvent.change(await screen.findByLabelText('Repository'), {
      target: { value: 'ggerganov/whisper.cpp' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Look up' }));

    expect(await screen.findByText('5359861c739e')).toBeTruthy();
    expect(screen.getByText('MIT')).toBeTruthy();
    expect((screen.getByLabelText('Name in ClipMill') as HTMLInputElement).value).toBe(
      'whisper-small',
    );
    fireEvent.click(screen.getByRole('radio', { name: 'ggml-medium.bin' }));
    expect((screen.getByLabelText('Name in ClipMill') as HTMLInputElement).value).toBe(
      'whisper-medium',
    );
    fireEvent.click(screen.getByRole('button', { name: /Add and download · 1.4 GB/ }));

    await waitFor(() => {
      expect(models.asked('add')).toEqual([
        [
          'add',
          {
            repo: 'ggerganov/whisper.cpp',
            commit: '5359861c739e955e79d9a303bcbc70fb988958b1',
            capability: 'asr',
            weightsFile: 'ggml-medium.bin',
            name: 'whisper-medium',
            title: 'Whisper medium',
          },
        ],
      ]);
    });
    expect(models.asked('inspect')).toEqual([['inspect', 'ggerganov/whisper.cpp', '', 'asr']]);
    expect(await screen.findByRole('listitem', { name: 'Whisper medium' })).toBeTruthy();
  });

  it('will not pin a repository whose licence ClipMill does not permit', async () => {
    const models = show(installedLibrary());
    models.inspection = {
      repo: 'someone/restricted',
      commit: 'a'.repeat(40),
      capability: 'asr',
      license: 'cc-by-nc-4.0',
      licenseSpdx: '',
      licenseAllowed: false,
      problem: 'Its licence (cc-by-nc-4.0) is not one ClipMill permits.',
      weightChoices: [],
      files: [],
      suggestedName: '',
      suggestedTitle: '',
    };
    fireEvent.click(await screen.findByRole('button', { name: /Add from Hugging Face/ }));
    fireEvent.change(await screen.findByLabelText('Repository'), {
      target: { value: 'someone/restricted' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Look up' }));
    expect(await screen.findByText(/is not one ClipMill permits/)).toBeTruthy();
    expect(screen.getByRole('button', { name: /Add and download/ }).hasAttribute('disabled')).toBe(
      true,
    );
  });

  it('warns that editorial models other than Qwen may not follow the format', async () => {
    const models = show(installedLibrary());
    models.inspection = {
      repo: 'mlx-community/Qwen3.5-4B-4bit',
      commit: 'b'.repeat(40),
      capability: 'editorial',
      license: 'apache-2.0',
      licenseSpdx: 'Apache-2.0',
      licenseAllowed: true,
      problem: '',
      weightChoices: [],
      files: [
        { path: 'config.json', bytes: 3_000, suggestedName: '', suggestedTitle: '' },
        { path: 'model.safetensors', bytes: 2_400_000_000, suggestedName: '', suggestedTitle: '' },
      ],
      suggestedName: 'qwen3.5-4b-4bit',
      suggestedTitle: 'Qwen3.5-4B-4bit',
    };
    fireEvent.click(await screen.findByRole('button', { name: /Add from Hugging Face/ }));
    fireEvent.click(screen.getByRole('radio', { name: 'Editorial AI' }));
    fireEvent.change(screen.getByLabelText('Repository'), {
      target: { value: 'mlx-community/Qwen3.5-4B-4bit' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Look up' }));
    expect(await screen.findByText(/2 files pinned together/)).toBeTruthy();
    expect(screen.getByText(/tuned for Qwen3.5 9B/)).toBeTruthy();
    fireEvent.change(screen.getByLabelText('Name in ClipMill'), {
      target: { value: 'Not A Name' },
    });
    expect(screen.getByText('Lowercase letters, digits, hyphens and dots.')).toBeTruthy();
    expect(screen.getByRole('button', { name: /Add and download/ }).hasAttribute('disabled')).toBe(
      true,
    );
  });
});
