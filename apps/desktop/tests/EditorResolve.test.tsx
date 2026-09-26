import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { TooltipProvider } from '../src/components/ui/tooltip.js';
import type { ShellApi } from '../src/daemon/api.js';
import type { EditorState } from '../src/editor/useEditor.js';
import { EditorScreen } from '../src/screens/EditorScreen.js';
import { plan } from './support/clips.js';
import { seekTo } from './support/timeline.js';

/** The clip's frame, which every solve is asked to fit. */
const FRAME = { width: 1080, height: 1920 };

let state: EditorState;
vi.mock('../src/editor/useEditor.js', () => ({ useEditor: () => state }));
const clip = { projectId: 'project', docId: 'edit', sourceId: 'source', candidateId: 'candidate' };
function show(solveCropPath: ShellApi['solveCropPath'], more: Partial<ShellApi> = {}) {
  const props = {
    clip,
    onOpenResults: vi.fn(),
    onOpen: vi.fn(),
    onExport: vi.fn(),
    api: { solveCropPath, ...more } as ShellApi,
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
    document: null,
    proxyUrls: new Map([[segment.sourceFingerprint, 'http://localhost/proxy.mp4']]),
    faceTrack: { projectId: 'project', artifactId: 'faces' },
    transcript: null,
    filmstrip: null,
    peaks: null,
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
  seekTo(state.plan!, 450);
  fireEvent.mouseDown(screen.getByRole('tab', { name: /framing/i }));
  fireEvent.click(screen.getByRole('button', { name: 'Recalculate framing' }));
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
    expect(solve).toHaveBeenCalledWith('project', 'faces', 1_800_000, 2_700_000, { aspect: FRAME });
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
      expect(screen.getByRole('button', { name: 'Recalculate framing' })).toBeTruthy(),
    );
    expect(state.apply).not.toHaveBeenCalled();
  });
});

describe('re-solving a two-person section', () => {
  beforeEach(() => {
    const segments = state.plan!.segments.map((segment) =>
      Object.assign({}, segment, { hasTwoUpPaths: true }),
    );
    state = { ...state, plan: { ...state.plan!, segments } };
  });

  it('keeps both people, each portrait solved for half the frame', async () => {
    const solve = vi.fn().mockResolvedValue({
      fit: false,
      keyframes: [{ tTicks: 1_800_000, centerX: 0.25, centerY: 0.5, scale: 0.5 }],
      secondaryKeyframes: [{ tTicks: 1_800_000, centerX: 0.75, centerY: 0.5, scale: 0.5 }],
      containment: 1,
    });
    show(solve);
    resolveSecond();
    await waitFor(() => expect(state.apply).toHaveBeenCalled());
    expect(solve).toHaveBeenCalledWith('project', 'faces', 1_800_000, 2_700_000, {
      twoUp: true,
      aspect: FRAME,
    });
    // Half the output's height: a 9:8 portrait, 608 wide for 540 tall.
    expect(state.apply).toHaveBeenCalledWith({
      op: 'batch',
      commands: [
        { op: 'set_layout', segment_id: 'second', state: 'two_up' },
        {
          op: 'replace_crop_path',
          segment_id: 'second',
          path: [{ t_ticks: 0, rect: { x: 176, y: 270, width: 608, height: 540 } }],
        },
        {
          op: 'replace_secondary_crop_path',
          segment_id: 'second',
          path: [{ t_ticks: 0, rect: { x: 1136, y: 270, width: 608, height: 540 } }],
        },
      ],
    });
  });

  it('lets the camera decide afresh when the pair is no longer clear', async () => {
    const solve = vi
      .fn()
      .mockResolvedValueOnce({
        fit: true,
        fitReason: 'two people are not both clearly in this section',
        keyframes: [],
        containment: 0,
      })
      .mockResolvedValueOnce({
        fit: false,
        keyframes: [{ tTicks: 1_800_000, centerX: 0.5, centerY: 0.5, scale: 1 }],
        containment: 1,
      });
    show(solve);
    resolveSecond();
    await waitFor(() => expect(state.apply).toHaveBeenCalled());
    expect(solve).toHaveBeenLastCalledWith('project', 'faces', 1_800_000, 2_700_000, {
      aspect: FRAME,
    });
    expect(state.apply).toHaveBeenCalledWith(
      expect.objectContaining({
        commands: expect.arrayContaining([
          { op: 'set_layout', segment_id: 'second', state: 'speaker_fill' },
        ]),
      }),
    );
  });
});

describe('following a person picked on the whole frame', () => {
  const sightings = [0, 1].flatMap((trackId) =>
    [2_235_000, 2_250_000, 2_265_000].map((tTicks) => ({
      trackId,
      tTicks,
      x: trackId === 0 ? 0.2 : 0.7,
      y: 0.3,
      width: 0.1,
      height: 0.2,
    })),
  );

  it('offers each face in the Original view and follows the one clicked', async () => {
    const solve = vi.fn().mockResolvedValue({
      fit: false,
      keyframes: [{ tTicks: 1_800_000, centerX: 0.75, centerY: 0.5, scale: 1 }],
      containment: 1,
      trackId: 1,
    });
    const listFaces = vi.fn().mockResolvedValue(sightings);
    show(solve, { listFaces });
    seekTo(state.plan!, 450);
    fireEvent.click(screen.getByRole('button', { name: 'Original' }));
    await waitFor(() =>
      expect(listFaces).toHaveBeenCalledWith('project', 'faces', 1_800_000, 2_700_000),
    );
    const people = await screen.findAllByRole('button', { name: /^Follow person/ });
    expect(people).toHaveLength(2);
    fireEvent.click(people[1]!);
    await waitFor(() => expect(state.apply).toHaveBeenCalled());
    expect(solve).toHaveBeenCalledWith('project', 'faces', 1_800_000, 2_700_000, {
      trackId: 1,
      aspect: FRAME,
    });
    expect(state.apply).toHaveBeenCalledWith(
      expect.objectContaining({
        commands: expect.arrayContaining([
          { op: 'set_layout', segment_id: 'second', state: 'speaker_fill' },
        ]),
      }),
    );
    // Back on the edit, where the new framing shows.
    expect(screen.getByRole('button', { name: 'Edit' }).getAttribute('aria-pressed')).toBe('true');
  });

  it('says why when that person cannot be followed through the section', async () => {
    const solve = vi.fn().mockResolvedValue({
      fit: true,
      fitReason: 'the clearest face appears in too little of this clip to follow',
      keyframes: [],
      containment: 0,
    });
    show(solve, { listFaces: vi.fn().mockResolvedValue(sightings) });
    seekTo(state.plan!, 450);
    fireEvent.click(screen.getByRole('button', { name: 'Original' }));
    fireEvent.click((await screen.findAllByRole('button', { name: /^Follow person/ }))[0]!);
    expect((await screen.findByRole('alert')).textContent).toMatch(/cannot be followed/);
    expect(state.apply).not.toHaveBeenCalled();
  });
});

const still = (x: number) => [{ t_ticks: 0, rect: { x, y: 140, width: 900, height: 800 } }];

describe('choosing a layout in the Framing tab', () => {
  beforeEach(() => {
    const segments = state.plan!.segments.map((segment) =>
      Object.assign({}, segment, { hasTwoUpPaths: true }),
    );
    const layout = { state: 'two_up', crop_path: still(0), secondary_crop_path: still(1000) };
    state = {
      ...state,
      plan: { ...state.plan!, segments },
      document: {
        video: {
          segments: segments.map((segment) => ({
            segment_id: segment.segmentId,
            source_fingerprint: segment.sourceFingerprint,
            in_ticks: segment.inTicks,
            out_ticks: segment.outTicks,
            layout,
          })),
        },
        captions: { cues: [], options: {} },
      } as unknown as EditorState['document'],
    };
  });
  function openFraming() {
    seekTo(state.plan!, 450);
    fireEvent.mouseDown(screen.getByRole('tab', { name: /framing/i }));
  }
  const applied = () =>
    (
      vi.mocked(state.apply).mock.calls.at(-1)![0] as unknown as {
        commands: Record<string, unknown>[];
      }
    ).commands;

  it('turns two speakers into a picture in picture, the second person inset', () => {
    show(vi.fn());
    openFraming();
    fireEvent.click(screen.getByRole('button', { name: 'Picture in picture' }));
    const commands = applied();
    expect(commands[0]).toEqual({
      op: 'set_layout',
      segment_id: 'second',
      state: 'picture_in_picture',
    });
    const inset = commands.find((command) => command.op === 'replace_secondary_crop_path') as {
      path: { rect: { width: number; height: number } }[];
    };
    expect(inset.path[0]!.rect.width).toBe(inset.path[0]!.rect.height);
  });

  it('changes the whole clip’s shape in one step, the two people side by side', () => {
    show(vi.fn());
    openFraming();
    expect(screen.getByRole('button', { name: 'Vertical 9:16' }).getAttribute('aria-pressed')).toBe(
      'true',
    );
    fireEvent.click(screen.getByRole('button', { name: 'Landscape 16:9' }));
    const commands = applied();
    expect(commands[0]).toEqual({ op: 'set_frame_shape', shape: 'landscape' });
    const halves = commands.filter((command) =>
      String(command.op).startsWith('replace_'),
    ) as unknown as { path: { rect: { width: number; height: number } }[] }[];
    expect(halves).toHaveLength(4);
    for (const half of halves) {
      const { width, height } = half.path[0]!.rect;
      expect(Math.abs(width / height - 960 / 1080)).toBeLessThan(0.01);
    }
  });

  it('puts the whole recording on top for a screen and a face', () => {
    show(vi.fn());
    openFraming();
    fireEvent.click(screen.getByRole('button', { name: 'Screen and face' }));
    expect(applied()).toContainEqual({
      op: 'set_layout_style',
      segment_id: 'second',
      split: 316,
    });
  });

  it('copies a style to every section, reshaping two-person crops for the split', () => {
    const segments = state.document!.video.segments!.map((segment) =>
      Object.assign({}, segment, { layout: Object.assign({}, segment.layout, { split: 400 }) }),
    );
    state = {
      ...state,
      document: { ...state.document!, video: { ...state.document!.video, segments } },
    };
    show(vi.fn());
    openFraming();
    fireEvent.click(screen.getByRole('button', { name: 'Use this style in every section' }));
    const commands = applied();
    expect(commands).toContainEqual({ op: 'set_layout_style', segment_id: 'first', split: 400 });
    expect(commands.filter((command) => command.segment_id === 'first')).toHaveLength(3);
  });
});
