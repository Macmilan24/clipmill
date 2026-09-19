import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SettingsScreen } from '../src/screens/SettingsScreen.js';
import { emptyWorld, fakeApi } from './support/library.js';

const measured = {
  categories: [{ key: 'state', bytes: 2048, items: 1, path: '/data/projects' }],
  availableBytes: 10_000,
  retentionGraceSeconds: 604_800,
};
const locked = { engaged: true, stages: 28, networkAllowedStages: 2, egressAttempts: 0 };

describe('settings reads and preferences', () => {
  it('surfaces a partial failure and clears it after a successful refresh', async () => {
    const api = fakeApi(emptyWorld());
    api.fetchStorageStats = vi
      .fn()
      .mockRejectedValueOnce(new Error('Disk unavailable'))
      .mockResolvedValue(measured);
    api.fetchLocalLock = async () => locked;
    render(<SettingsScreen api={api} />);
    expect(
      await screen.findByText(/Storage could not be refreshed. Disk unavailable/),
    ).toBeTruthy();
    expect(screen.getByText('Engaged')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Refresh status' }));
    expect(await screen.findByText('/data/projects')).toBeTruthy();
    expect(screen.queryByText(/Storage could not be refreshed/)).toBeNull();
  });

  it('labels cached privacy evidence when refreshing it fails', async () => {
    const api = fakeApi(emptyWorld());
    api.fetchStorageStats = async () => measured;
    api.fetchLocalLock = vi
      .fn()
      .mockResolvedValueOnce(locked)
      .mockRejectedValue(new Error('Engine disconnected'));
    render(<SettingsScreen api={api} />);
    await screen.findByText('Engaged');
    fireEvent.click(screen.getByRole('button', { name: 'Refresh status' }));
    await screen.findByText(/Privacy status could not be refreshed/);
    expect(screen.getByText('Last known status')).toBeTruthy();
    expect(screen.queryByText('Engaged')).toBeNull();
    expect(screen.getByText('/data/projects')).toBeTruthy();
  });

  it('ignores a late answer from a replaced engine client', async () => {
    const slow = fakeApi(emptyWorld());
    let finish!: (value: typeof measured) => void;
    slow.fetchStorageStats = () =>
      new Promise((resolve) => {
        finish = resolve;
      });
    slow.fetchLocalLock = async () => locked;
    const next = fakeApi(emptyWorld());
    next.fetchStorageStats = async () => ({
      ...measured,
      categories: [{ ...measured.categories[0]!, path: '/new/projects' }],
    });
    next.fetchLocalLock = async () => locked;
    const view = render(<SettingsScreen api={slow} />);
    view.rerender(<SettingsScreen api={next} />);
    await screen.findByText('/new/projects');
    await act(async () => finish(measured));
    expect(screen.queryByText('/data/projects')).toBeNull();
  });

  it('passes theme choices to the shell and keeps connected-account content usable', async () => {
    const api = fakeApi(emptyWorld());
    api.fetchStorageStats = async () => measured;
    api.fetchLocalLock = async () => locked;
    const changed = vi.fn();
    render(
      <SettingsScreen
        api={api}
        theme="dark"
        onThemeChange={changed}
        integrations={<button>Connect YouTube</button>}
      />,
    );
    const light = screen.getByRole('radio', { name: 'Light studio' });
    fireEvent.click(light);
    expect(changed).toHaveBeenCalledWith('light');
    expect(screen.getByRole('button', { name: 'Connect YouTube' })).toBeTruthy();
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Refresh status' })).toBeTruthy(),
    );
  });
});
