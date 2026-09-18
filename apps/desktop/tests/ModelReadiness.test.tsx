import { render, screen } from '@testing-library/react';
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
});
