/**
 * The rule the registry exists to enforce: a section shows a real screen only if
 * it says it is live, and anything else says which phase will build it.
 *
 * Worth testing rather than assuming, because the failure is silent in both
 * directions. A screen wired up before its section is ready would reach a user
 * half-built; a section marked live with nothing registered would render an
 * empty pane that looks like a bug rather than like work not done yet.
 */
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { renderScreen } from '../src/screens/registry.js';
import { NAV_SECTIONS, findSection } from '../src/shell/navigation.js';

const models = {
  state: { status: 'connecting' } as const,
  profile: null,
  artifactId: null,
  error: null,
  busy: false,
  onRescan: () => undefined,
  onReconnect: () => undefined,
};

describe('the screen registry', () => {
  it('renders the real screen for a live section', () => {
    render(renderScreen({ section: findSection('models'), models }));
    // The Models screen asks the daemon for hardware; the placeholder never
    // mentions a phase.
    expect(screen.queryByText(/Phase \d/)).toBeNull();
  });

  it('names the phase for every section that is not built yet', () => {
    const planned = NAV_SECTIONS.filter((section) => section.availability.kind === 'planned');
    expect(planned.length).toBeGreaterThan(0);
    for (const section of planned) {
      const { unmount } = render(renderScreen({ section, models }));
      expect(screen.getByText(/Phase \d/)).toBeTruthy();
      unmount();
    }
  });

  it('never leaves a section with nothing on screen', () => {
    for (const section of NAV_SECTIONS) {
      const { container, unmount } = render(renderScreen({ section, models }));
      expect(container.textContent?.trim().length ?? 0).toBeGreaterThan(0);
      unmount();
    }
  });
});
