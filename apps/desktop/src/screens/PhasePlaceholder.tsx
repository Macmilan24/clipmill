import type { JSX } from 'react';

import { FilmIcon } from '../shell/icons.js';
import type { NavSection } from '../shell/navigation.js';

/**
 * Phase 0 builds the harness, not the product surface. Every section that is
 * not yet real says which phase builds it instead of showing a convincing
 * mockup — a fake project grid here would be the exact "demo-tier" behaviour
 * ClipMill exists to avoid.
 */
export function PhasePlaceholder({ section }: { readonly section: NavSection }): JSX.Element {
  if (section.availability.kind === 'live') {
    throw new Error(`${section.id} is live and must render its own screen`);
  }
  const { phase, summary } = section.availability;

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="t-page-title">{section.breadcrumb}</h1>
          <p>Not built yet — and deliberately not mocked.</p>
        </div>
      </div>

      <section className="glass card empty" aria-label={`${section.label} is not available yet`}>
        <span className="empty-well">
          <FilmIcon />
        </span>
        <h2 className="t-card-title">Arrives in Phase {phase}</h2>
        <p>{summary}</p>
        <p className="mono muted">phase 0 · harness only · contracts, daemon, artifacts, workers</p>
      </section>
    </>
  );
}
