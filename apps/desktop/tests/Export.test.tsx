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
    revision: 4,
    ...overrides,
  };
}

function show(overrides: Partial<Parameters<typeof Export>[0]> = {}) {
  const onExport = vi.fn();
  const props = {
    docId: 'edt_1',
    labels: { project: 'Episode 41', clip: 'Clip 01' },
    picker: null,
    destination: '/Users/sami/Movies/clips',
    pattern: '{index}-{clip}',
    title: 'Charging less',
    attestation: 'own_content',
    rightsGateNeeded: false,
    rightsGatePassed: false,
    hotCaptions: [],
    plan: plan(),
    planning: false,
    busy: false,
    error: null,
    delivery: null,
    archive: null,
    onDestinationChange: vi.fn(),
    onPatternChange: vi.fn(),
    onChooseFolder: vi.fn(),
    onRightsGateChange: vi.fn(),
    onExport,
    onArchive: vi.fn(),
    onReveal: vi.fn(),
    ...overrides,
  };
  render(<Export {...props} />);
  return { onExport, props };
}

describe('what the export screen says is left', () => {
  it('names the one thing still missing and then says it is ready', () => {
    show({ destination: '' });
    expect(screen.getByText('Choose a folder to export into.')).toBeTruthy();
  });

  it('says ready when every check passes', () => {
    show();
    expect(screen.getByText('Ready to export.')).toBeTruthy();
  });

  it('counts the blocking checks', () => {
    show({
      plan: plan({
        passes: false,
        findings: [
          { code: 'destination.unusable', severity: 'blocking', detail: 'Not writable.' },
          { code: 'source.missing', severity: 'blocking', detail: 'The recording moved.' },
        ],
      }),
    });
    expect(screen.getByText('2 checks to fix first.')).toBeTruthy();
  });
});

describe('a moved recording', () => {
  it('offers to locate it on the finding and nowhere else', () => {
    const onRelink = vi.fn();
    show({
      onRelink,
      plan: plan({
        passes: false,
        findings: [
          {
            code: 'source.missing',
            severity: 'blocking',
            detail: 'The recording ep41.mov is no longer where it was imported from.',
          },
        ],
      }),
    });
    const buttons = screen.getAllByRole('button', { name: /Locate recording/ });
    expect(buttons).toHaveLength(1);
    fireEvent.click(buttons[0]!);
    expect(onRelink).toHaveBeenCalledOnce();
  });

  it('shows no relink when the recording is where it was', () => {
    show({ onRelink: vi.fn() });
    expect(screen.queryByRole('button', { name: /Locate recording/ })).toBeNull();
  });
});

describe('the delivery format', () => {
  it('states the recording rate and the chosen size', () => {
    show({
      format: { rate: 'source', height: 2560 },
      sourceFps: 24_000 / 1_001,
      onFormatChange: vi.fn(),
    });
    expect(screen.getByText('1440 × 2560 · 23.98 fps · MP4')).toBeTruthy();
    expect(screen.getByLabelText('Frame rate')).toBeTruthy();
    expect(screen.getByLabelText('Resolution')).toBeTruthy();
  });
});

describe('the export screen', () => {
  it('offers an editor recovery path for a blocked subtitle', () => {
    const onEdit = vi.fn();
    const { onExport } = show({
      onEdit,
      plan: plan({
        passes: false,
        findings: [
          {
            code: 'captions.too_brief',
            severity: 'blocking',
            detail:
              'Subtitle 4 at 0:42.51 — “[BLANK_AUDIO]” is on screen for 0.64s; the minimum is 0.83s.',
            cueId: 'cue_4',
          },
        ],
      }),
    });
    expect(screen.getByText(/Subtitle 4 at 0:42.51/)).toBeTruthy();
    expect(screen.getByText(/cannot be exported as it is/)).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /fix captions/i }));
    // The editor is sent to the cue itself, on the track the finding is about.
    expect(onEdit).toHaveBeenCalledWith({ panel: 'captions', track: 'reading', cueId: 'cue_4' });
    fireEvent.click(screen.getByRole('button', { name: /^export clip$/i }));
    expect(onExport).not.toHaveBeenCalled();
  });

  it('still points at a brief subtitle the strip only advises on, without blocking', () => {
    const onEdit = vi.fn();
    show({
      onEdit,
      plan: plan({
        passes: true,
        findings: [
          {
            code: 'captions.too_brief',
            severity: 'advisory',
            detail:
              'Subtitle 4 at 0:42.51 — “[BLANK_AUDIO]” is on screen for 0.64s; the minimum is 0.83s.',
            cueId: 'cue_4',
          },
        ],
      }),
    });
    expect(screen.getByText(/worth a look before exporting/)).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /fix captions/i }));
    expect(onEdit).toHaveBeenCalledWith({ panel: 'captions', track: 'reading', cueId: 'cue_4' });
    expect(screen.getByRole('button', { name: /^export clip$/i })).toHaveProperty(
      'disabled',
      false,
    );
  });

  it('describes render progress as media time and delivery as the next step', () => {
    show({
      delivery: {
        revision: 4,
        destinationDir: '/tmp/clips',
        settled: false,
        files: null,
        failure: null,
        interruption: null,
        stages: [
          {
            kind: 'render',
            label: 'Render',
            state: 'running',
            progress: { unit: 'media_millis', done: 6336, total: 15900 },
            waitReason: '',
          },
          {
            kind: 'deliver',
            label: 'Deliver',
            state: 'waiting',
            progress: null,
            waitReason: 'waiting: dependencies',
          },
        ],
      },
    });
    expect(screen.getByTestId('stage-render').textContent).toBe('0:06.3 of 0:15.9 rendered');
    expect(screen.getByTestId('stage-deliver').textContent).toBe('After rendering');
    expect(screen.queryByText(/media_millis|dependencies/)).toBeNull();
  });

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
    const button = screen.getByRole('button', { name: /^export clip$/i });
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
    fireEvent.click(screen.getByRole('button', { name: /^export clip$/i }));
    expect(onExport).toHaveBeenCalledOnce();
  });

  it('asks for the rights confirmation only past the minute mark', () => {
    show({ rightsGateNeeded: false });
    expect(screen.queryByText(/I hold the rights/)).toBeNull();
  });

  it('confirms the selected source permission without exposing its metadata code', () => {
    show({ rightsGateNeeded: true });
    expect(screen.getByText(/selected source permission/)).toBeTruthy();
    expect(screen.queryByText(/own_content/)).toBeNull();
  });

  it('says a free-space figure could not be read rather than showing zero', () => {
    // Absent, not zero. The two are different answers and the type says so, so
    // the key is left off rather than set to undefined.
    const { availableBytes: _unread, ...unknown } = plan();
    show({ plan: unknown });
    expect(screen.getByText('Free disk space').nextElementSibling?.textContent).toBe('Unavailable');
  });

  it('says no clip is chosen rather than showing an empty form', () => {
    show({ docId: null, labels: null });
    expect(screen.getByText('No clip is chosen for export')).toBeTruthy();
    expect(screen.queryByRole('button', { name: /^export/i })).toBeNull();
  });
});

it('keeps export disabled while a changed destination or filename is being validated', () => {
  show({ planning: true });
  expect(screen.getByRole('button', { name: /^export clip$/i })).toHaveProperty('disabled', true);
});

it('groups fast captions as advice without holding up export', () => {
  const finding = {
    code: 'captions.reading_rate',
    severity: 'advisory' as const,
    detail: 'Passage one asks for 22.1 characters a second.',
  };
  show({ hotCaptions: [finding], plan: plan({ passes: true, findings: [finding] }) });
  expect(screen.getByText(/1 fast subtitle passage/)).toBeTruthy();
  expect(screen.getAllByText(finding.detail)).toHaveLength(1);
  expect(screen.queryByText(/captions[._]reading_rate/)).toBeNull();
  expect(screen.getByText('Review caption details (1)')).toBeTruthy();
  expect(screen.getByRole('button', { name: /^export clip$/i })).toHaveProperty('disabled', false);
});
