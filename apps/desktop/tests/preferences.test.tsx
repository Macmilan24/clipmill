/**
 * Settings' review and export defaults reach what the Inspector, Export and
 * New Project start from.
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { recallPattern } from '../src/export/format.js';
import { PlaybackController } from '../src/inspector/playback.js';
import { autoAdvance, reviewSpeed } from '../src/inspector/preferences.js';
import { startingLook } from '../src/results/captionLook.js';
import { EditingPreferences } from '../src/screens/EditingPreferences.js';
import { ReviewPreferences } from '../src/screens/ReviewPreferences.js';

describe('review and export defaults', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('sets the speed review starts at and whether a decision moves on', () => {
    render(<ReviewPreferences />);
    expect(reviewSpeed()).toBe(1);
    fireEvent.click(screen.getByRole('button', { name: '1.5×' }));
    expect(reviewSpeed()).toBe(1.5);
    expect(new PlaybackController(0, reviewSpeed()).getState().speed).toBe(1.5);
    fireEvent.click(screen.getByRole('switch', { name: 'Next clip after deciding' }));
    expect(autoAdvance()).toBe(false);
  });

  it('keeps a file-name pattern and the caption look new projects start with', () => {
    render(<EditingPreferences />);
    const pattern = screen.getByLabelText('Files are named');
    fireEvent.change(pattern, { target: { value: 'episode-{index}' } });
    fireEvent.blur(pattern);
    expect(recallPattern()).toBe('episode-{index}');
    fireEvent.change(pattern, { target: { value: '' } });
    fireEvent.blur(pattern);
    expect(recallPattern()).toBe('{index}-{clip}');
    fireEvent.click(screen.getByRole('button', { name: 'Boxed' }));
    expect(startingLook()).toBe('clipmill.captions.boxed.v1');
  });
});
