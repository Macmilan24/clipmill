/**
 * The Export screen, and the two things it must not do.
 *
 * It must not resolve the naming pattern itself — the names on screen are the
 * daemon's answer, because the daemon is what writes the files — and it must
 * not let an export start when the strip found something blocking. Both are
 * checked by giving the screen a daemon whose answers contradict what a local
 * implementation would have produced.
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { ExportPlan } from '../src/daemon/client.js';
import { Export } from '../src/screens/Export.js';

function plan(overrides: Partial<ExportPlan> = {}): ExportPlan {
  return {
    passes: true,
    findings: [],
    stem: '01-charging-less',
    fileNames: [
      '01-charging-less.mp4',
      '01-charging-less.srt',
      '01-charging-less.vtt',
      '01-charging-less.render-manifest.json',
      '01-charging-less.jpg',
      '01-charging-less.metadata.json',
      '01-charging-less.sha256',
    ],
    estimatedBytes: 130_000_000,
    availableBytes: 335_007_449_088,
    ...overrides,
  };
}

function show(overrides: Partial<Parameters<typeof Export>[0]> = {}) {
  const onExport = vi.fn();
  const props = {
    docId: 'edt_1',
    destination: '/Users/sami/Movies/clips',
    pattern: '{index}-{clip}',
    title: 'Charging less',
    attestation: 'own_content',
    rightsGateNeeded: false,
    rightsGatePassed: false,
    plan: plan(),
    planning: false,
    busy: false,
    error: null,
    queued: null,
    archive: null,
    onDestinationChange: vi.fn(),
    onPatternChange: vi.fn(),
    onChooseFolder: vi.fn(),
    onRightsGateChange: vi.fn(),
    onExport,
    onArchive: vi.fn(),
    ...overrides,
  };
  render(<Export {...props} />);
  return { onExport, props };
}

describe('the export screen', () => {
  it('shows the names the daemon resolved rather than names of its own', () => {
    // The pattern says {index}-{clip} and the title is "Charging less", so a
    // local implementation would draw "01-Charging-less". The daemon said
    // otherwise, and the daemon is what writes the files.
    show({ plan: plan({ stem: 'totally-different', fileNames: ['totally-different.mp4'] }) });
    expect(screen.getByText('totally-different.mp4')).toBeTruthy();
    expect(screen.queryByText('01-charging-less.mp4')).toBeNull();
  });

  it('lists every file an export writes, not just the clip', () => {
    show();
    for (const suffix of ['mp4', 'srt', 'vtt', 'jpg', 'metadata.json', 'sha256']) {
      expect(screen.getByText(`01-charging-less.${suffix}`)).toBeTruthy();
    }
  });

  it('refuses to start when the strip found something blocking', () => {
    const { onExport } = show({
      plan: plan({
        passes: false,
        findings: [
          {
            code: 'boundary.inside_word',
            severity: 'blocking',
            detail: 'A cut at 12.40 s lands inside “pricing”.',
          },
        ],
      }),
    });
    expect(screen.getByText(/lands inside/)).toBeTruthy();
    const button = screen.getByRole('button', { name: 'Export' });
    fireEvent.click(button);
    expect(onExport).not.toHaveBeenCalled();
  });

  it('shows an advisory without blocking the export', () => {
    const { onExport } = show({
      plan: plan({
        passes: true,
        findings: [
          {
            code: 'captions.burn_in.reading_rate',
            severity: 'advisory',
            detail: 'Burned-in caption: hot_3 runs at 46 characters a second.',
          },
        ],
      }),
    });
    expect(screen.getByText(/46 characters a second/)).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Export' }));
    expect(onExport).toHaveBeenCalledOnce();
  });

  it('asks for the rights confirmation only past the minute mark', () => {
    show({ rightsGateNeeded: false });
    expect(screen.queryByText(/I hold the rights/)).toBeNull();
  });

  it('records the attestation it will write, in the words it will write', () => {
    show({ rightsGateNeeded: true });
    expect(screen.getByText(/own_content/)).toBeTruthy();
  });

  it('says a free-space figure could not be read rather than showing zero', () => {
    // Absent, not zero. The two are different answers and the type says so, so
    // the key is left off rather than set to undefined.
    const { availableBytes: _unread, ...unknown } = plan();
    show({ plan: unknown });
    expect(screen.getByText('not readable')).toBeTruthy();
  });

  it('says nothing is approved rather than showing an empty form', () => {
    show({ docId: null });
    expect(screen.getByText('Nothing approved yet')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Export' })).toBeNull();
  });
});
