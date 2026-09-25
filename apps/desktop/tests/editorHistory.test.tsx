/**
 * The clip's history and name: undo that outlives the window, a title that is
 * an edit, one time format, and the keys in one sheet.
 */
import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { ClipTitle } from '../src/editor/EditorHistory.js';
import { describeCommand, historySteps, revertTo } from '../src/editor/history.js';
import { ShortcutSheet, useShortcutSheet } from '../src/shell/ShortcutSheet.js';
import { formatTime, setTimeFormat, timeFormat } from '../src/shell/timeFormat.js';

afterEach(() => localStorage.clear());

const entry = (revision: number, command: object, inverse: object) => ({
  revision,
  commandJson: JSON.stringify(command),
  inverseJson: JSON.stringify(inverse),
  appliedUnixMillis: 1_000 * revision,
});

describe('the history', () => {
  const steps = historySteps([
    entry(
      1,
      { op: 'set_layout', segment_id: 's', state: 'fit' },
      { op: 'set_layout', segment_id: 's', state: 'speaker_fill' },
    ),
    entry(2, { op: 'set_title', title: 'Better' }, { op: 'set_title' }),
    entry(3, { op: 'regroup_on_screen', max_words: 2 }, { op: 'batch', commands: [] }),
  ]);

  it('names each edit the way the controls do', () => {
    expect(steps.map((step) => step.label)).toEqual([
      'Framing: Whole frame',
      'Rename: Better',
      'Words on screen: 2',
    ]);
    expect(
      describeCommand({
        op: 'batch',
        commands: [
          { op: 'set_word_emphasis', word_id: 'a', emphasis: true },
          { op: 'set_word_emphasis', word_id: 'b', emphasis: true },
        ],
      }),
    ).toBe('Key word');
  });

  it('goes back to a point by undoing everything since, newest first', () => {
    expect(revertTo(steps, 1)).toEqual({
      op: 'batch',
      commands: [{ op: 'batch', commands: [] }, { op: 'set_title' }],
    });
    expect(revertTo(steps, 2)).toEqual({ op: 'batch', commands: [] });
    expect(revertTo(steps, 3)).toBeNull();
  });
});

describe('the clip title', () => {
  it('renames in place and clears back to the headline', () => {
    const onApply = vi.fn();
    const { rerender } = render(
      <ClipTitle title={null} fallback="The headline" busy={false} onApply={onApply} />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'The headline' }));
    const input = screen.getByRole('textbox', { name: 'Clip title' });
    fireEvent.change(input, { target: { value: '  A better name ' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onApply).toHaveBeenLastCalledWith({ op: 'set_title', title: 'A better name' });

    rerender(
      <ClipTitle title="A better name" fallback="The headline" busy={false} onApply={onApply} />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'A better name' }));
    const again = screen.getByRole('textbox', { name: 'Clip title' });
    fireEvent.change(again, { target: { value: '' } });
    fireEvent.keyDown(again, { key: 'Enter' });
    expect(onApply).toHaveBeenLastCalledWith({ op: 'set_title' });
  });

  it('leaves the name alone on Escape', () => {
    const onApply = vi.fn();
    render(<ClipTitle title={null} fallback="The headline" busy={false} onApply={onApply} />);
    fireEvent.click(screen.getByRole('button', { name: 'The headline' }));
    const input = screen.getByRole('textbox', { name: 'Clip title' });
    fireEvent.change(input, { target: { value: 'Nope' } });
    fireEvent.keyDown(input, { key: 'Escape' });
    expect(onApply).not.toHaveBeenCalled();
  });
});

describe('one time format', () => {
  it('reads as a clock by default and counts frames at the clip’s rate on request', () => {
    expect(timeFormat()).toBe('clock');
    expect(formatTime(90_000 * 12.4, 'clock', 24)).toBe('0:12.4');
    // Half a second at 23.976 is frame 11, not a broadcast frame count.
    expect(formatTime(90_000 * 12.5, 'frames', 24_000 / 1_001)).toBe('0:12:11');
    expect(formatTime(90_000 * 3_725, 'frames', 25)).toBe('1:02:05:00');
    setTimeFormat('frames');
    expect(timeFormat()).toBe('frames');
  });
});

describe('the shortcut sheet', () => {
  function Host() {
    const sheet = useShortcutSheet();
    return <ShortcutSheet open={sheet.open} onOpenChange={sheet.setOpen} />;
  }

  it('opens with the command key and slash, and lists the keys', () => {
    render(<Host />);
    expect(screen.queryByText('Keyboard shortcuts')).toBeNull();
    act(() => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '/', metaKey: true }));
    });
    expect(screen.getByText('Keyboard shortcuts')).toBeTruthy();
    expect(screen.getByText('Approve and open in the Editor')).toBeTruthy();
  });
});
