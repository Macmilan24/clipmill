/**
 * "A new version is out": asked at most once a day, never on the first day,
 * and never while the person has turned it off.
 */
import { act, fireEvent, render, renderHook, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { TooltipProvider } from '@/components/ui/tooltip';

import { TopBar } from '../src/shell/TopBar.js';
import {
  CHECK_EVERY_MS,
  type UpdatesApi,
  isNewer,
  setUpdatesEnabled,
  useUpdateNotice,
} from '../src/shell/updates.js';

function fakeApi(latestVersion: string) {
  return {
    checkForUpdate: vi.fn(async () => ({ currentVersion: '0.3.0', latestVersion, newer: true })),
    openReleasePage: vi.fn(async () => {}),
  } satisfies UpdatesApi;
}

describe('the new-version notice', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('compares releases by number, not by text', () => {
    expect(isNewer('0.10.0', '0.9.9')).toBe(true);
    expect(isNewer('0.3.0', '0.3.0')).toBe(false);
    expect(isNewer('0.2.9', '0.3.0')).toBe(false);
    expect(isNewer('nightly', '0.3.0')).toBe(false);
  });

  it('waits a day after installing, asks once a day, and hides what is dismissed', async () => {
    let now = 1_000;
    const api = fakeApi('0.4.0');
    const first = renderHook(() => useUpdateNotice('0.3.0', api, () => now));
    expect(api.checkForUpdate).not.toHaveBeenCalled();
    first.unmount();

    now += CHECK_EVERY_MS;
    const later = renderHook(() => useUpdateNotice('0.3.0', api, () => now));
    await waitFor(() => expect(later.result.current?.version).toBe('0.4.0'));
    later.rerender();
    expect(api.checkForUpdate).toHaveBeenCalledTimes(1);

    later.result.current?.open();
    expect(api.openReleasePage).toHaveBeenCalledWith('0.4.0');
    act(() => later.result.current?.dismiss());
    expect(later.result.current).toBeNull();
  });

  it('asks nothing while it is off, and says nothing to a build already that new', async () => {
    let now = 1_000;
    const api = fakeApi('0.4.0');
    renderHook(() => useUpdateNotice('0.4.0', api, () => now)).unmount();
    now += CHECK_EVERY_MS;
    const current = renderHook(() => useUpdateNotice('0.4.0', api, () => now));
    await waitFor(() => expect(api.checkForUpdate).toHaveBeenCalledTimes(1));
    expect(current.result.current).toBeNull();
    current.unmount();

    act(() => setUpdatesEnabled(false));
    now += 2 * CHECK_EVERY_MS;
    renderHook(() => useUpdateNotice('0.3.0', api, () => now));
    expect(api.checkForUpdate).toHaveBeenCalledTimes(1);
    act(() => setUpdatesEnabled(true));
  });

  it('shows in the top bar, opening the page or going away', () => {
    const notice = { version: '0.4.0', open: vi.fn(), dismiss: vi.fn() };
    render(
      <TooltipProvider>
        <TopBar
          trail={['Library']}
          theme="dark"
          onToggleTheme={() => {}}
          state={{ status: 'connecting' }}
          profile={null}
          update={notice}
        />
      </TooltipProvider>,
    );
    fireEvent.click(screen.getByText('ClipMill 0.4.0 is out'));
    expect(notice.open).toHaveBeenCalled();
    fireEvent.click(screen.getByLabelText('Not now: hide ClipMill 0.4.0'));
    expect(notice.dismiss).toHaveBeenCalled();
  });
});
