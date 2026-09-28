import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { TooltipProvider } from '../src/components/ui/tooltip.js';
import { ManualClip, type ManualClipProps } from '../src/results/ManualClip.js';
import { manualSpanProblem, parseSourceTime, sourceTime } from '../src/results/manual.js';

const TICKS = 90_000;
function createButton() {
  return screen.getByRole('button', { name: 'Create manual edit' });
}
function props(overrides: Partial<ManualClipProps> = {}): ManualClipProps {
  return {
    open: true,
    onOpenChange: vi.fn(),
    sourceName: 'An older recording',
    sourceDurationTicks: 720 * TICKS,
    proxyUrl: '/recording.mp4',
    busy: false,
    notice: null,
    onCreate: vi.fn(async () => true),
    ...overrides,
  };
}

describe('source time entry', () => {
  it.each([
    ['600.125', 600.125],
    ['10:00.125', 600.125],
    ['1:02:03.5', 3723.5],
    [' 0:00 ', 0],
    ['62:03', 3723],
  ])('parses %s as an absolute source position', (value, seconds) => {
    expect(parseSourceTime(value)).toBe(seconds * TICKS);
  });
  it.each(['', '-1', '1:60', '1:60:01', '1:01:60', 'NaN', '1.1234', '1:2:3:4', '1e3'])(
    'refuses invalid time %s',
    (value) => expect(parseSourceTime(value)).toBeNull(),
  );
  it('round trips millisecond positions, including long recordings', () => {
    for (const seconds of [0, 0.001, 59.999, 600.125, 7200.005]) {
      const ticks = Math.round(seconds * TICKS);
      expect(parseSourceTime(sourceTime(ticks))).toBe(ticks);
    }
  });
  it('requires known duration and ordered positions inside the recording', () => {
    expect(manualSpanProblem(0, TICKS, null)).toContain('unavailable');
    expect(manualSpanProblem(0, Number.NaN, TICKS)).toContain('Use seconds');
    expect(manualSpanProblem(TICKS, TICKS, TICKS)).toContain('after the start');
    expect(manualSpanProblem(0, 2 * TICKS, TICKS)).toContain('beyond the recording');
    expect(manualSpanProblem(0, TICKS, TICKS)).toBeNull();
  });
});

const sheet = (input: ManualClipProps) => (
  <TooltipProvider>
    <ManualClip {...input} />
  </TooltipProvider>
);

describe('manual clip recovery', () => {
  it('creates the typed source interval and closes only after success', async () => {
    const input = props();
    render(sheet(input));
    fireEvent.change(screen.getByLabelText('End time'), { target: { value: '10:30.5' } });
    fireEvent.change(screen.getByLabelText('Start time'), { target: { value: '10:00.125' } });
    fireEvent.click(screen.getByRole('button', { name: 'Create manual edit' }));
    await waitFor(() => expect(input.onCreate).toHaveBeenCalledWith(54_011_250, 56_745_000));
    expect(input.onOpenChange).toHaveBeenCalledWith(false);
    expect(screen.getByText(/model has not recommended or reviewed it/)).toBeTruthy();
  });
  it('blocks unavailable duration and out-of-source spans, even without a proxy', () => {
    const input = props({ sourceDurationTicks: null, proxyUrl: null });
    const view = render(sheet(input));
    expect(createButton().hasAttribute('disabled')).toBe(true);
    // When evidence arrives, a short source gets a valid default end.
    view.rerender(sheet({ ...input, sourceDurationTicks: 4 * TICKS }));
    expect((screen.getByLabelText('End time') as HTMLInputElement).value).toBe('0:04');
    expect(createButton().hasAttribute('disabled')).toBe(false);
    fireEvent.change(screen.getByLabelText('End time'), { target: { value: '0:05' } });
    expect(createButton().hasAttribute('disabled')).toBe(true);
  });
  it('marks the span at the playhead the shared transport moves, on word edges', () => {
    render(sheet(props()));
    fireEvent.change(screen.getByLabelText('End time'), { target: { value: '10:30' } });
    fireEvent.change(screen.getByLabelText('Start time'), { target: { value: '10:00' } });
    fireEvent.click(screen.getByRole('button', { name: 'Go to the end of the cut' }));
    fireEvent.click(screen.getByRole('button', { name: 'Start here' }));
    // Never past a second before the end.
    expect((screen.getByLabelText('Start time') as HTMLInputElement).value).toBe('10:29');
    fireEvent.click(screen.getByRole('button', { name: 'Go to the start of the cut' }));
    fireEvent.keyDown(screen.getByRole('group', { name: 'Choose the span' }), { key: 'i' });
    expect((screen.getByLabelText('Start time') as HTMLInputElement).value).toBe('10:29');
  });
  it('lands a mark on the nearest word edge when the words are known', () => {
    const words = [
      { text: 'Hello', startTicks: 600 * TICKS, endTicks: 600.4 * TICKS },
      { text: 'there.', startTicks: 600.5 * TICKS, endTicks: 601 * TICKS },
    ];
    render(
      sheet(
        props({
          transcript: {
            words,
            sentences: [
              { startTicks: 600 * TICKS, endTicks: 601 * TICKS, firstWord: 0, wordCount: 2 },
            ],
          },
        }),
      ),
    );
    fireEvent.change(screen.getByLabelText('End time'), { target: { value: '10:30' } });
    fireEvent.change(screen.getByLabelText('Start time'), { target: { value: '10:00.2' } });
    fireEvent.click(screen.getByRole('button', { name: 'Go to the start of the cut' }));
    fireEvent.click(screen.getByRole('button', { name: 'Start here' }));
    // Inside "Hello", nearer its end: the start lands on the edge before "there."
    expect((screen.getByLabelText('Start time') as HTMLInputElement).value).toBe('10:00.400');
  });
  it('keeps the selection after refusal and reports an unexpected failure without closing', async () => {
    const input = props({
      onCreate: vi.fn(async () => {
        throw new Error('Run evidence unavailable');
      }),
    });
    render(sheet(input));
    fireEvent.click(screen.getByRole('button', { name: 'Create manual edit' }));
    expect(await screen.findByRole('status')).toHaveProperty(
      'textContent',
      'Run evidence unavailable',
    );
    expect(input.onOpenChange).not.toHaveBeenCalled();
  });
});
