/**
 * The caption controls: trying a look saves nothing, choosing one is one edit.
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { editorDocument, editorPlan } from '../dev/editor-fixtures.js';
import { CaptionStyleControls } from '../src/editor/CaptionStyle.js';

function show(overrides: Partial<Parameters<typeof CaptionStyleControls>[0]> = {}) {
  const onApply = vi.fn();
  const onTryLook = vi.fn();
  render(
    <CaptionStyleControls
      plan={editorPlan}
      document={editorDocument}
      busy={false}
      fonts={[
        { family: 'Inter', label: 'Inter', file: 'Inter-Bold.ttf', installed: true },
        { family: 'Anton', label: 'Anton', file: 'Anton-Regular.ttf', installed: true },
        {
          family: 'Bebas Neue',
          label: 'Bebas Neue',
          file: 'BebasNeue-Regular.ttf',
          installed: false,
        },
      ]}
      onApply={onApply}
      onTryLook={onTryLook}
      {...overrides}
    />,
  );
  return { onApply, onTryLook };
}

describe('the caption controls', () => {
  it('draws a look on hover and saves it only when clicked', () => {
    const { onApply, onTryLook } = show();
    const boxed = screen.getByRole('button', { name: /Boxed/ });
    fireEvent.pointerEnter(boxed);
    expect(onTryLook).toHaveBeenLastCalledWith(
      expect.objectContaining({ styleRef: 'clipmill.captions.boxed.v1' }),
    );
    expect(onApply).not.toHaveBeenCalled();
    fireEvent.click(boxed);
    expect(onApply).toHaveBeenCalledOnce();
    expect(JSON.stringify(onApply.mock.calls[0]![0])).toContain('clipmill.captions.boxed.v1');
  });

  it('chooses how the spoken word is marked', () => {
    const { onApply, onTryLook } = show();
    const box = screen.getByRole('button', { name: 'Box' });
    fireEvent.pointerEnter(box);
    expect(onTryLook).toHaveBeenLastCalledWith({
      options: expect.objectContaining({ highlight_style: 'box' }),
    });
    fireEvent.click(box);
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'set_caption_options',
      options: expect.objectContaining({ highlight_style: 'box' }),
    });
  });

  it('regroups the on-screen captions by words at once', () => {
    const { onApply } = show();
    fireEvent.click(
      screen.getByRole('group', { name: 'Words on screen at once' }).querySelectorAll('button')[0]!,
    );
    expect(onApply).toHaveBeenLastCalledWith({ op: 'regroup_on_screen', max_words: 1 });
  });

  it('offers only installed fonts', () => {
    show();
    expect(screen.getByRole('combobox', { name: 'Caption font' })).toBeTruthy();
  });

  it('offers to remove the recognizer’s non-speech marks only when there are some', () => {
    const noisy = structuredClone(editorDocument);
    noisy.captions.cues![0]!.lines[0]!.words[0]!.text = '-';
    const { onApply } = show({ document: noisy });
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }));
    expect(onApply).toHaveBeenLastCalledWith({ op: 'drop_non_speech_words' });
  });

  it('says nothing about non-speech marks when there are none', () => {
    show();
    expect(screen.queryByRole('button', { name: 'Remove' })).toBeNull();
  });
});
