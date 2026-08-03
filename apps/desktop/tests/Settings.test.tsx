/**
 * The Settings screen, and the claim it exists to make checkable.
 *
 * The Local Lock card is not a badge — it is three counts. These check that the
 * card follows the counts rather than a word: a daemon reporting a
 * network-allowed stage, or an egress attempt that already happened, must not
 * be able to render as "engaged".
 */
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { LocalLock, StorageStats } from '../src/daemon/client.js';
import { Settings } from '../src/screens/Settings.js';

function storage(overrides: Partial<StorageStats> = {}): StorageStats {
  return {
    categories: [
      { key: 'artifacts', bytes: 20_078_895_104, items: 412, path: '/data/artifacts' },
      { key: 'models', bytes: 24_588_615_680, items: 6, path: '/data/models' },
      { key: 'state', bytes: 43_008_000, items: 4, path: '/data/state' },
    ],
    availableBytes: 335_007_449_088,
    retentionGraceSeconds: 604_800,
    ...overrides,
  };
}

function lock(overrides: Partial<LocalLock> = {}): LocalLock {
  return {
    engaged: true,
    stages: 28,
    networkAllowedStages: 0,
    egressAttempts: 0,
    ...overrides,
  };
}

function show(overrides: Partial<Parameters<typeof Settings>[0]> = {}) {
  render(
    <Settings storage={storage()} lock={lock()} loading={false} error={null} {...overrides} />,
  );
}

describe('the settings screen', () => {
  it('points at where each category lives, not just how big it is', () => {
    show();
    expect(screen.getByText('/data/artifacts')).toBeTruthy();
    expect(screen.getByText('/data/models')).toBeTruthy();
    expect(screen.getByText('/data/state')).toBeTruthy();
  });

  it('reads a retention window in days rather than seconds', () => {
    show();
    expect(screen.getByText('7 days')).toBeTruthy();
  });

  it('says a free-space figure could not be read rather than showing zero', () => {
    show({ storage: storage({ availableBytes: undefined }) });
    expect(screen.getByText('not readable')).toBeTruthy();
  });

  it('shows the counts behind the lock, not only its state', () => {
    show();
    expect(screen.getByText('Engaged')).toBeTruthy();
    expect(screen.getByText('28')).toBeTruthy();
  });

  it('does not read as engaged when something reached out', () => {
    show({ lock: lock({ engaged: false, egressAttempts: 3 }) });
    expect(screen.getByText('Not engaged')).toBeTruthy();
    expect(screen.getByText('3')).toBeTruthy();
    expect(screen.queryByText('Engaged')).toBeNull();
  });

  it('does not read as engaged when a stage is allowed the network', () => {
    show({ lock: lock({ engaged: false, networkAllowedStages: 1 }) });
    expect(screen.getByText('Not engaged')).toBeTruthy();
  });

  it('reports a daemon that measures no storage rather than showing nothing', () => {
    show({ storage: null });
    expect(screen.getByText('This daemon measures no storage.')).toBeTruthy();
    // The other half still answered, so it is still on screen.
    expect(screen.getByText('Engaged')).toBeTruthy();
  });

  it('names the phase for what is not built rather than showing dead controls', () => {
    show();
    expect(screen.getByText('Phase 2')).toBeTruthy();
  });
});
