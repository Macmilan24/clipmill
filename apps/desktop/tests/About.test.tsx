/**
 * About, in Settings: which build this is, the licence it is shared under, and
 * the details a bug report needs.
 */
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { AboutSection, SOURCE_URL, debugDetails } from '../src/screens/AboutSection.js';

afterEach(() => {
  vi.restoreAllMocks();
});

describe('About ClipMill', () => {
  it('names the build, the engine, the source and the licence', () => {
    render(<AboutSection engineVersion="0.4.2" appVersion="0.4.1" />);
    expect(screen.getByText('0.4.1')).toBeTruthy();
    expect(screen.getByText('0.4.2')).toBeTruthy();
    expect(screen.getByText(SOURCE_URL)).toBeTruthy();
    expect(screen.getByText(/GNU Affero General Public License, version 3/)).toBeTruthy();
  });

  it('says the engine is not connected rather than showing nothing', () => {
    render(<AboutSection engineVersion={null} appVersion="0.4.1" />);
    expect(screen.getByText('Not connected')).toBeTruthy();
  });

  it('copies the details a bug report needs', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
    render(<AboutSection engineVersion="0.4.2" appVersion="0.4.1" />);
    fireEvent.click(screen.getByRole('button', { name: 'Copy details' }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Copied' })).toBeTruthy());
    expect(writeText.mock.calls[0]?.[0]).toContain('ClipMill 0.4.1');
    expect(writeText.mock.calls[0]?.[0]).toContain('Engine 0.4.2');
  });

  it('shows the details to select by hand when the clipboard refuses', async () => {
    const writeText = vi.fn().mockRejectedValue(new Error('denied'));
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
    render(<AboutSection engineVersion={null} appVersion={null} />);
    fireEvent.click(screen.getByRole('button', { name: 'Copy details' }));
    expect(await screen.findByText(/ClipMill development build/)).toBeTruthy();
    expect(debugDetails(null, null)).toContain('Engine not connected');
  });
});
