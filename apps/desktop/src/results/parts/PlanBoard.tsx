/**
 * The mind's clips as the Results board. Each card shows a still from where
 * the clip opens, its form, its keep score and length, and where its parts
 * sit in the recording. The rail shows the selected clip's parts, why it was
 * chosen, and what the review found at its joins. Edit makes an edit document
 * from the plan and Review opens its review; nothing here approves or exports
 * a clip by itself.
 */
import { ArrowRight, LayoutGrid, List, Scissors, SquarePen, TriangleAlert } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';

import { MediaStill } from '../../components/MediaStill.js';
import { Button } from '../../components/ui/button.js';
import { clock, duration } from '../model.js';
import type { PlannedClip } from '../plans.js';
import { PartStrip, PlanList, partLine } from './MindPlans.js';

type View = 'grid' | 'list';

export interface PlanBoardProps {
  readonly plans: readonly PlannedClip[];
  readonly sourceDurationTicks: number | null;
  readonly tileUrl: (atTicks: number) => string | null;
  readonly busy: boolean;
  readonly onEdit: (plan: PlannedClip) => void;
  readonly onInspect?: ((plan: PlannedClip) => void) | undefined;
}

/** Where a clip opens: its first part, which for a cold open is the hook. */
function opening(plan: PlannedClip): number {
  return plan.parts[0]?.startTicks ?? 0;
}

/** Why the clip was made, in a line: the review's summary, or the plan's own. */
function reasonOf(plan: PlannedClip): string {
  return plan.assessment?.summary.trim() || plan.summary;
}

function rank(plans: readonly PlannedClip[], plan: PlannedClip): string {
  return String(plans.indexOf(plan) + 1).padStart(2, '0');
}

export function PlanBoard({
  plans,
  sourceDurationTicks,
  tileUrl,
  busy,
  onEdit,
  onInspect,
}: PlanBoardProps) {
  const [view, setView] = useState<View>('grid');
  const [focusedId, setFocusedId] = useState<string | null>(null);
  const focused = useMemo(
    () => plans.find((plan) => plan.planId === focusedId) ?? plans[0] ?? null,
    [plans, focusedId],
  );
  const gridRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const active = gridRef.current?.querySelector('[aria-selected="true"]');
    if (active instanceof HTMLElement && typeof active.scrollIntoView === 'function') {
      active.scrollIntoView({ block: 'nearest' });
    }
  }, [focusedId]);

  const scale =
    sourceDurationTicks !== null && sourceDurationTicks > 0 ? sourceDurationTicks : null;

  return (
    <section aria-label="Planned clips" className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <p className="text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
          Best first. Each clip is cut from wherever its moment is: a question put before its
          answer, a strong line moved to the front, or a straight stretch where nothing needs
          removing.
        </p>
        <div
          className="glass flex h-[var(--cm-control-standard)] items-center gap-0.5 rounded-[var(--cm-radius-control)] p-1"
          role="group"
          aria-label="Layout"
        >
          {(
            [
              ['list', List, 'List'],
              ['grid', LayoutGrid, 'Grid'],
            ] as const
          ).map(([id, Icon, label]) => (
            <button
              key={id}
              type="button"
              aria-label={`${label} view`}
              aria-pressed={view === id}
              onClick={() => setView(id)}
              className="grid size-6 place-items-center rounded-[6px] transition-colors"
              style={
                view === id
                  ? { background: 'var(--cm-accent-selected)', color: 'var(--cm-text-primary)' }
                  : { color: 'var(--cm-text-muted)' }
              }
            >
              <Icon className="size-3.5" aria-hidden />
            </button>
          ))}
        </div>
      </div>

      <div className="results-layout" data-view={view}>
        {view === 'list' ? (
          <PlanList
            plans={plans}
            sourceDurationTicks={scale}
            focusedId={focused?.planId ?? null}
            busy={busy}
            onFocus={(plan) => setFocusedId(plan.planId)}
            onEdit={onEdit}
            onInspect={onInspect}
          />
        ) : (
          <div
            ref={gridRef}
            role="listbox"
            aria-label="Clips built by the mind"
            aria-activedescendant={focused ? `plan-${focused.planId}` : undefined}
            tabIndex={0}
            className="results-grid"
            onKeyDown={(event) => {
              if (event.target !== event.currentTarget || plans.length === 0) return;
              const at = focused ? plans.indexOf(focused) : 0;
              if (['ArrowDown', 'ArrowRight', 'j'].includes(event.key)) {
                event.preventDefault();
                setFocusedId(plans[Math.min(plans.length - 1, at + 1)]!.planId);
              } else if (['ArrowUp', 'ArrowLeft', 'k'].includes(event.key)) {
                event.preventDefault();
                setFocusedId(plans[Math.max(0, at - 1)]!.planId);
              } else if (event.key === 'Enter' && focused) {
                event.preventDefault();
                onEdit(focused);
              }
            }}
          >
            {plans.map((plan) => (
              <div
                key={plan.planId}
                id={`plan-${plan.planId}`}
                role="option"
                aria-selected={plan.planId === focused?.planId}
                onClick={() => setFocusedId(plan.planId)}
                onDoubleClick={() => onEdit(plan)}
                className="results-card"
              >
                <div className="results-card-picture" data-vertical="true">
                  <MediaStill src={tileUrl(opening(plan))} />
                  <span className="results-card-duration mono">
                    {duration(plan.durationSeconds)}
                  </span>
                </div>
                <div className="results-card-copy">
                  <div className="results-card-meta">
                    <span className="mono">
                      {rank(plans, plan)} · {clock(opening(plan))}
                    </span>
                    <span className="text-[var(--cm-accent-ink)]">{plan.formLabel}</span>
                  </div>
                  <p className="results-card-title">{plan.title}</p>
                  <p className="results-card-reason" title={reasonOf(plan)}>
                    {reasonOf(plan)}
                  </p>
                  {scale !== null && <PartStrip parts={plan.parts} sourceDurationTicks={scale} />}
                  <div className="results-card-bottom">
                    <span className="results-card-review">
                      {plan.assessment
                        ? `Keep score ${plan.assessment.keepScore}`
                        : `${plan.parts.length} ${plan.parts.length === 1 ? 'part' : 'parts'}`}
                    </span>
                    <button
                      type="button"
                      className="results-card-open"
                      aria-label={`Edit: ${plan.title}`}
                      disabled={busy}
                      onClick={(event) => {
                        event.stopPropagation();
                        onEdit(plan);
                      }}
                    >
                      Edit <SquarePen size={14} aria-hidden />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
        <PlanRail
          plans={plans}
          plan={focused}
          tileUrl={tileUrl}
          busy={busy}
          onEdit={onEdit}
          onInspect={onInspect}
        />
      </div>
    </section>
  );
}

/** The selected clip: how it is built, why it was chosen, and what was checked. */
function PlanRail({
  plans,
  plan,
  tileUrl,
  busy,
  onEdit,
  onInspect,
}: {
  readonly plans: readonly PlannedClip[];
  readonly plan: PlannedClip | null;
  readonly tileUrl: (atTicks: number) => string | null;
  readonly busy: boolean;
  readonly onEdit: (plan: PlannedClip) => void;
  readonly onInspect?: ((plan: PlannedClip) => void) | undefined;
}) {
  if (!plan) return null;
  const assessment = plan.assessment;
  const cautions = [...(assessment?.issues ?? []), ...plan.doubts];
  return (
    <aside className="results-detail" aria-label="Selected clip">
      <div className="results-detail-body">
        <div className="results-detail-heading">
          <span>Selected clip</span>
          <span className="mono">{rank(plans, plan)}</span>
        </div>
        <div className="results-detail-picture">
          <MediaStill src={tileUrl(opening(plan))} />
          <span className="results-card-duration mono">{duration(plan.durationSeconds)}</span>
        </div>
        <h2 className="results-detail-title">{plan.title}</h2>
        <div className="results-detail-meta">
          <span className="text-[var(--cm-accent-ink)]">{plan.formLabel}</span>
          {assessment && <span className="mono">Keep score {assessment.keepScore}/100</span>}
        </div>
        {plan.hookText && (
          <p className="text-[11px] italic leading-relaxed text-[var(--cm-text-secondary)]">
            “{plan.hookText}”
          </p>
        )}
        <p className="text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
          {reasonOf(plan)}
        </p>
        {cautions[0] && (
          <p className="results-caution">
            <TriangleAlert size={14} aria-hidden />
            <span>{cautions[0]}</span>
          </p>
        )}
        <details className="results-disclosure" open key={`parts-${plan.planId}`}>
          <summary>How it is built</summary>
          <ol className="results-disclosure-content">
            {plan.parts.map((part, index) => (
              <li key={part.id}>
                <span className="mono">{partLine(part, index)}</span>
                {part.why && <p>{part.why}</p>}
              </li>
            ))}
          </ol>
        </details>
        {assessment && assessment.reasons.length > 0 && (
          <details className="results-disclosure" key={`why-${plan.planId}`}>
            <summary>Why it was chosen</summary>
            <ul className="results-disclosure-content">
              {assessment.reasons.map((reason, index) => (
                <li key={index}>{reason}</li>
              ))}
            </ul>
          </details>
        )}
        {assessment && assessment.joins.length > 0 && (
          <details className="results-disclosure" key={`joins-${plan.planId}`}>
            <summary>What the review found at the joins</summary>
            <ul className="results-disclosure-content">
              {assessment.joins.map((join, index) => (
                <li key={index}>
                  After {join.after}: {join.verdict}. {join.note}
                </li>
              ))}
            </ul>
          </details>
        )}
      </div>
      <div className="results-detail-actions">
        <Button className="w-full justify-center" disabled={busy} onClick={() => onEdit(plan)}>
          <Scissors />
          Open as an edit
        </Button>
        {onInspect && (
          <Button
            variant="outline"
            className="w-full justify-center"
            disabled={busy}
            onClick={() => onInspect(plan)}
            aria-label={`Review ${plan.title}`}
          >
            Review <ArrowRight />
          </Button>
        )}
      </div>
    </aside>
  );
}
