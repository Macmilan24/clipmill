import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ModelReadiness } from '../src/screens/ModelReadiness.js';
import { emptyWorld, fakeApi } from './support/library.js';

describe('local model readiness', () => {
  it('counts a shared model once and separates installation from worker readiness', async () => {
    const api = fakeApi(emptyWorld());
    api.fetchReadiness = async () => ({
      ready: false,
      decoderPresent: true,
      decoderPath: '/ffmpeg',
      workers: [],
      stages: ['editorial-propose', 'editorial-review'].map((stage) => ({
        stage,
        model: 'Qwen 3.5',
        modelPresent: true,
        workerPresent: false,
        ready: false,
        capability: stage,
        implementation: 'test',
        backend: 'mlx',
        missingFiles: [],
        remedy: 'Start the editorial worker.',
      })),
    });
    render(<ModelReadiness api={api} />);
    expect(await screen.findByText('1 installed')).toBeTruthy();
    expect(screen.getAllByText('Qwen 3.5')).toHaveLength(1);
    expect(screen.getByText('Needs attention')).toBeTruthy();
    expect(screen.getByText('Start the editorial worker.')).toBeTruthy();
  });

  it('does not invent an installation count when the engine cannot answer', async () => {
    const api = fakeApi(emptyWorld());
    api.fetchReadiness = async () => {
      throw new Error('Disconnected');
    };
    render(<ModelReadiness api={api} />);
    expect(await screen.findByText('Status unavailable')).toBeTruthy();
    expect(screen.queryByText('0 installed')).toBeNull();
  });
  it('keeps missing-file recovery explicit and excludes cloud routes from local model totals', async () => {
    const api = fakeApi(emptyWorld());
    api.fetchReadiness = async () => ({
      ready: false,
      decoderPresent: true,
      decoderPath: '/ffmpeg',
      workers: [],
      stages: [
        {
          stage: 'speech-asr',
          model: 'whisper-large-v3',
          modelPresent: false,
          workerPresent: true,
          ready: false,
          capability: 'asr',
          implementation: 'speech@1.0',
          backend: 'mlx',
          missingFiles: ['/weights/model.safetensors'],
          remedy: 'Fetch the pinned speech weights.',
        },
        {
          stage: 'editorial-propose-cloud',
          model: 'provider-model',
          modelPresent: true,
          workerPresent: false,
          ready: false,
          capability: 'cloud',
          implementation: 'cloud@1.0',
          backend: 'https',
          missingFiles: [],
          remedy: 'Enable cloud processing.',
        },
      ],
    });
    render(<ModelReadiness api={api} />);
    expect(await screen.findByText('0 installed')).toBeTruthy();
    expect(screen.getByText('Not installed')).toBeTruthy();
    expect(screen.getByText('Worker connected')).toBeTruthy();
    expect(screen.getByText('/weights/model.safetensors')).toBeTruthy();
    expect(screen.getByText('Fetch the pinned speech weights.')).toBeTruthy();
    expect(screen.queryByText('provider-model')).toBeNull();
  });

  it('does not leave a ready badge on stale evidence after a failed refresh', async () => {
    const api = fakeApi(emptyWorld());
    let fail = false;
    api.fetchReadiness = async () => {
      if (fail) throw new Error('Disconnected');
      return {
        ready: true,
        decoderPresent: true,
        decoderPath: '/ffmpeg',
        workers: [],
        stages: [
          {
            stage: 'speech-asr',
            model: 'whisper',
            modelPresent: true,
            workerPresent: true,
            ready: true,
            capability: 'asr',
            implementation: 'speech@1.0',
            backend: 'mlx',
            missingFiles: [],
            remedy: '',
          },
        ],
      };
    };
    render(<ModelReadiness api={api} />);
    await screen.findByText('Ready for analysis');
    fail = true;
    fireEvent.click(screen.getByRole('button', { name: 'Refresh model readiness' }));
    await screen.findByText(/Model readiness is unavailable/);
    expect(screen.queryByText('Ready for analysis')).toBeNull();
    expect(screen.getByText('Last known status')).toBeTruthy();
  });
});
