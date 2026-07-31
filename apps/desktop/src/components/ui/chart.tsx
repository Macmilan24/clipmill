/**
 * The chart surface: a themed container and a tooltip, over Recharts.
 *
 * Charts are drawn by a library rather than by hand. Axes, scales, hit testing
 * and tooltip placement are solved problems with a great many edge cases — an
 * SVG written here would be worse at all of them and would drift from the rest
 * of the interface the first time a value went negative or a label got long.
 *
 * What this file owns is the part a library cannot know: the design's colours and
 * type. Series colours arrive as CSS custom properties set on the container, so a
 * chart reads the same tokens every other surface does and follows the theme
 * without re-rendering.
 */
import type * as React from 'react';
import { ResponsiveContainer } from 'recharts';

import { cn } from '@/lib/utils';

/** One series: what it is called, and which token draws it. */
export interface ChartSeries {
  readonly label: string;
  /** A CSS colour, normally `var(--…)`. */
  readonly color: string;
}

export type ChartConfig = Readonly<Record<string, ChartSeries>>;

/**
 * Publishes each series colour as `--color-<key>`, which is what Recharts marks
 * reference. One place decides colour; the marks name a series and get whatever
 * the theme currently says.
 */
function seriesVariables(config: ChartConfig): React.CSSProperties {
  return Object.fromEntries(
    Object.entries(config).map(([key, series]) => [`--color-${key}`, series.color]),
  ) as React.CSSProperties;
}

export function ChartContainer({
  config,
  className,
  children,
  style,
  ...props
}: React.ComponentProps<'div'> & {
  readonly config: ChartConfig;
  readonly children: React.ComponentProps<typeof ResponsiveContainer>['children'];
}): React.JSX.Element {
  return (
    <div
      data-slot="chart"
      // Merged, not replaced: a caller passing its own style — a height, say —
      // must not silently drop the series colours the marks are drawn with.
      style={{ ...seriesVariables(config), ...style }}
      className={cn(
        'w-full text-[var(--cm-text-secondary)]',
        // Grid and axes are recessive: the marks carry the message and the
        // scaffolding should not compete with them.
        '[&_.recharts-cartesian-grid_line]:stroke-[var(--cm-glass-border)]',
        '[&_.recharts-cartesian-axis-line]:stroke-transparent',
        '[&_.recharts-cartesian-axis-tick_text]:fill-[var(--cm-text-muted)]',
        '[&_.recharts-cartesian-axis-tick_text]:text-technical',
        // Direct labels on marks: the value, in the technical face, in ink.
        '[&_.recharts-label]:fill-[var(--cm-text-primary)]',
        '[&_.recharts-label]:font-(family-name:--cm-font-mono)',
        '[&_.recharts-label]:text-technical',
        '[&_.recharts-rectangle.recharts-tooltip-cursor]:fill-[var(--cm-accent-selected)]',
        '[&_svg]:outline-none',
        className,
      )}
      {...props}
    >
      <ResponsiveContainer>{children}</ResponsiveContainer>
    </div>
  );
}

/**
 * What Recharts hands a tooltip, narrowed to the parts this renders.
 *
 * `dataKey` can be a selector function in the library's own types; a chart here
 * always names a key, so anything else resolves to nothing and the entry falls
 * back to its series name.
 */
interface TooltipEntry {
  readonly dataKey?: string | number | ((row: never) => unknown) | undefined;
  readonly name?: string | number | undefined;
  readonly value?: number | string | readonly (string | number)[] | undefined;
  readonly color?: string | undefined;
}

/**
 * The tooltip, in the shell's own glass rather than the library's default.
 *
 * A coloured mark beside the label carries identity; the text stays in the ink
 * tokens, because a value written in its series colour is a value that stops
 * being readable the moment the series is a pale one.
 */
export function ChartTooltipContent({
  active,
  payload,
  label,
  config,
  unit,
}: {
  readonly active?: boolean;
  readonly payload?: readonly TooltipEntry[];
  readonly label?: React.ReactNode;
  readonly config: ChartConfig;
  readonly unit?: string;
}): React.JSX.Element | null {
  if (active !== true || payload === undefined || payload.length === 0) {
    return null;
  }
  return (
    <div className="glass rounded-[var(--cm-radius-control)] px-2.5 py-2 shadow-md">
      {label === undefined ? null : (
        <div className="mb-1 text-meta font-(--cm-weight-label) text-[var(--cm-text-primary)]">
          {label}
        </div>
      )}
      {payload.map((entry) => {
        const key = String(
          (typeof entry.dataKey === 'function' ? undefined : entry.dataKey) ?? entry.name ?? '',
        );
        const series = config[key];
        return (
          <div key={key} className="flex items-center gap-2">
            <span
              aria-hidden="true"
              className="size-1.5 shrink-0 rounded-[2px]"
              style={{ background: series?.color ?? entry.color }}
            />
            <span className="text-technical text-[var(--cm-text-secondary)]">
              {series?.label ?? key}
            </span>
            <span className="mono ml-auto text-technical text-[var(--cm-text-primary)]">
              {Array.isArray(entry.value) ? entry.value.join('–') : entry.value}
              {unit === undefined ? '' : ` ${unit}`}
            </span>
          </div>
        );
      })}
    </div>
  );
}
