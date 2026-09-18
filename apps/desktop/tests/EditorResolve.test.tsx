import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { TooltipProvider } from '../src/components/ui/tooltip.js';
import type { ShellApi } from '../src/daemon/api.js';
import type { EditorState } from '../src/editor/useEditor.js';
import { EditorScreen } from '../src/screens/EditorScreen.js';
import { plan } from './support/clips.js';

let state: EditorState;
vi.mock('../src/editor/useEditor.js', () => ({ useEditor: () => state }));
const clip = { projectId: 'project', docId: 'edit', sourceId: 'source', candidateId: 'candidate' };
function show(solveCropPath: ShellApi['solveCropPath']) {
  const props = {
    clip,
    onOpenResults: vi.fn(),
    onOpen: vi.fn(),
    onExport: vi.fn(),
    api: { solveCropPath } as ShellApi,
  };
  const view = render(
    <TooltipProvider>
      <EditorScreen {...props} />
    </TooltipProvider>,
  );
  return () =>
    view.rerender(
      <TooltipProvider>
        <EditorScreen {...props} />
      </TooltipProvider>,
    );
}
beforeEach(() => {
  const base = plan();
  const segment = base.segments[0]!;
  const program = {
    ...base,
    revision: 1,
    rateNum: 30,
    rateDen: 1,
    frameCount: 600,
    crops: Array.from({ length: 600 }, () => [600, 0, 608, 1080] as const),
    segments: [
      {
        ...segment,
        segmentId: 'first',
        inTicks: 900_000,
        outTicks: 1_800_000,
        programStartTicks: 0,
        firstFrame: 0,
        endFrame: 300,
      },
      {
        ...segment,
        segmentId: 'second',
        inTicks: 1_800_000,
        outTicks: 2_700_000,
        programStartTicks: 900_000,
        firstFrame: 300,
        endFrame: 600,
      },
    ],
  };
  state = {
    docId: 'edit',
    revision: 1,
    plan: program,
    proxyUrls: new Map([[segment.sourceFingerprint, 'http://localhost/proxy.mp4']]),
    faceTrack: { projectId: 'project', artifactId: 'faces' },
    loading: false,
    busy: false,
    problem: null,
    canUndo: false,
    canRedo: false,
    apply: vi.fn().mockResolvedValue(undefined),
    undo: vi.fn(),
    redo: vi.fn(),
  };
});
function resolveSecond() {
  fireEvent.change(screen.getByRole('slider', { name: /scrub/i }), { target: { value: '450' } });
  fireEvent.mouseDown(screen.getByRole('tab', { name: 'Reframe' }));
  fireEvent.click(screen.getByRole('button', { name: 'Re-solve the path' }));
}
describe('re-solving the current shot', () => {
  it('targets the shot under the playhead and replaces every old crop keyframe', async () => {
    const solve = vi.fn().mockResolvedValue({
      fit: false,
      keyframes: [{ tTicks: 1_800_000, centerX: 0.5, centerY: 0.5, scale: 1 }],
      containment: 1,
    });
    show(solve);
    resolveSecond();
    await waitFor(() => expect(state.apply).toHaveBeenCalled());
    expect(solve).toHaveBeenCalledWith('project', 'faces', 1_800_000, 2_700_000);
    expect(state.apply).toHaveBeenCalledWith({
      op: 'batch',
      commands: [
        { op: 'set_layout', segment_id: 'second', state: 'speaker_fill' },
        {
          op: 'replace_crop_path',
          segment_id: 'second',
          path: [{ t_ticks: 0, rect: { x: 656, y: 0, width: 608, height: 1080 } }],
        },
      ],
    });
  });
  it('surfaces a solver failure instead of dropping a rejected promise', async () => {
    show(vi.fn().mockRejectedValue(new Error('Face evidence is unavailable.')));
    resolveSecond();
    expect(await screen.findByRole('alert')).toHaveProperty(
      'textContent',
      'Face evidence is unavailable.',
    );
    expect(state.apply).not.toHaveBeenCalled();
  });
  it('discards a solve after another edit changes the revision', async () => {
    let answer: (value: unknown) => void = () => {};
    const solve = vi.fn().mockImplementation(
      () =>
        new Promise((resolve) => {
          answer = resolve;
        }),
    );
    const rerender = show(solve);
    resolveSecond();
    state = { ...state, plan: { ...state.plan!, revision: 2 }, revision: 2 };
    rerender();
    answer({ fit: true, keyframes: [], containment: 0 });
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Re-solve the path' })).toBeTruthy(),
    );
    expect(state.apply).not.toHaveBeenCalled();
  });
});
