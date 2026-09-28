import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import type { JSX } from 'react';
import { describe, expect, it } from 'vitest';

import type { ComponentsApi } from '../src/daemon/components.js';
import { UNMANAGED } from '../src/daemon/components.js';
import type { ModelLibraryApi } from '../src/daemon/models.js';
import { ComponentsCard } from '../src/setup/ComponentsCard.js';
import { SetupCard } from '../src/setup/SetupCard.js';
import { useSetup } from '../src/setup/useSetup.js';
import { FakeComponents, component, packaged } from './support/components.js';
import { FakeModelLibrary, freshLibrary, installedLibrary } from './support/models.js';

function Harness({ api }: { readonly api: ComponentsApi & ModelLibraryApi }): JSX.Element {
  return <SetupCard setup={useSetup(api)} />;
}

function show(components: FakeComponents, models = new FakeModelLibrary(freshLibrary())) {
  render(<Harness api={{ ...models.api(), ...components.api() }} />);
  return models;
}

const FRESH = () =>
  new FakeComponents(
    packaged([
      component('vad', { title: 'Speech detection' }),
      component('asr-whispercpp', { title: 'Transcription' }),
    ]),
  );

describe('setting up a packaged app', () => {
  it('shows nothing in a development checkout, whose workers are started by hand', async () => {
    const components = new FakeComponents(UNMANAGED);
    const models = show(components);
    await waitFor(() => expect(components.asked('list')).toHaveLength(1));
    expect(screen.queryByTestId('setup')).toBeNull();
    expect(models.asked('list')).toHaveLength(0);
  });

  it('downloads the components and the recommended models with one click', async () => {
    const components = FRESH();
    const models = show(components);
    const setup = await screen.findByTestId('setup');
    expect(within(setup).getByText('Set up ClipMill')).toBeTruthy();
    expect(within(setup).getByText('Speech detection')).toBeTruthy();
    expect(within(setup).getByText(/Models: Qwen3.5 9B, Whisper Base, Silero VAD/)).toBeTruthy();
    expect(within(setup).getByText(/Nothing about your projects is sent/)).toBeTruthy();

    fireEvent.click(within(setup).getByRole('button', { name: /Set up · about/ }));

    await waitFor(() => {
      expect(components.asked('install')).toEqual([['install', []]]);
      expect(models.asked('download')).toEqual([
        ['download', ['qwen3-5-editorial-mlx', 'whisper-base', 'silero-vad']],
      ]);
    });
    expect(await within(setup).findAllByText('Waiting its turn')).toHaveLength(2);
    expect(within(setup).getByRole('button', { name: /Stop/ })).toBeTruthy();
  });

  it('says what each component is doing while it installs', async () => {
    const components = new FakeComponents(
      packaged([
        component('vad', { title: 'Speech detection', state: 'installed', process: 'running' }),
        component('faces', {
          title: 'Face tracking',
          state: 'installing',
          detail: 'Downloading its packages',
        }),
      ]),
    );
    show(components, new FakeModelLibrary(installedLibrary()));
    const setup = await screen.findByTestId('setup');
    expect(within(setup).getByText('Downloading its packages')).toBeTruthy();
    expect(within(setup).getByText('Installed')).toBeTruthy();
    expect(within(setup).getByText('Downloading')).toBeTruthy();
  });

  it('says why a component failed and offers another try', async () => {
    const components = new FakeComponents(
      packaged([
        component('asr-whispercpp', {
          title: 'Transcription',
          state: 'failed',
          detail:
            'No wheel for this platform (the full output is in /data/logs/engine-install.log)',
        }),
      ]),
    );
    show(components, new FakeModelLibrary(installedLibrary()));
    const setup = await screen.findByTestId('setup');
    expect(within(setup).getByText(/No wheel for this platform/)).toBeTruthy();
    expect(within(setup).getByText('Stopped')).toBeTruthy();
    fireEvent.click(within(setup).getByRole('button', { name: 'Try again' }));
    await waitFor(() => expect(components.asked('install')).toEqual([['install', []]]));
  });

  it('offers an update when an earlier version installed them', async () => {
    const components = new FakeComponents(
      packaged([
        component('vad', { title: 'Speech detection', state: 'outdated', process: 'running' }),
      ]),
    );
    show(components, new FakeModelLibrary(installedLibrary()));
    const setup = await screen.findByTestId('setup');
    expect(within(setup).getByText("Update ClipMill's components")).toBeTruthy();
    expect(within(setup).getByRole('button', { name: /Update · about 60 MB/ })).toBeTruthy();
  });

  it('goes away once everything is here', async () => {
    const components = new FakeComponents(
      packaged([component('vad', { title: 'Speech detection', state: 'installed' })]),
    );
    show(components, new FakeModelLibrary(installedLibrary()));
    await waitFor(() => expect(components.asked('list')).toHaveLength(1));
    expect(screen.queryByTestId('setup')).toBeNull();
  });
});

describe('the components on the Models page', () => {
  it('lists each component with its process, and retries one that failed', async () => {
    const components = new FakeComponents(
      packaged([
        component('vad', {
          title: 'Speech detection',
          state: 'installed',
          installedBytes: 183 * 1024 ** 2,
          process: 'running',
        }),
        component('shots', {
          title: 'Shot detection',
          state: 'installed',
          process: 'waiting',
          restarts: 3,
          detail: 'it stopped (exit status: 1)',
        }),
        component('faces', { title: 'Face tracking', state: 'failed', detail: 'no network' }),
      ]),
    );
    render(<ComponentsCard api={components.api()} />);
    const card = await screen.findByTestId('components');
    expect(within(card).getByText('Running')).toBeTruthy();
    expect(within(card).getByText('Restarting (3)')).toBeTruthy();
    expect(within(card).getByText('183 MB')).toBeTruthy();
    expect(within(card).getByText(/Python 3.12.13/)).toBeTruthy();
    fireEvent.click(within(card).getByRole('button', { name: 'Try again' }));
    await waitFor(() => expect(components.asked('install')).toEqual([['install', ['faces']]]));
  });

  it('is not shown in a development checkout', async () => {
    const components = new FakeComponents(UNMANAGED);
    render(<ComponentsCard api={components.api()} />);
    await waitFor(() => expect(components.asked('list')).toHaveLength(1));
    expect(screen.queryByTestId('components')).toBeNull();
  });
});
