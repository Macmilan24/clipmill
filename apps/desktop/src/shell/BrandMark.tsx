import type { JSX, SVGProps } from 'react';

/**
 * The scissor-blade wordmark. The only hand-drawn glyph left: every other icon
 * comes from Lucide, which is what the design calls for.
 */
export function BrandMark({
  size = 20,
  ...props
}: SVGProps<SVGSVGElement> & { size?: number }): JSX.Element {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
      {...props}
    >
      <circle cx="6" cy="6" r="2.5" />
      <circle cx="6" cy="18" r="2.5" />
      <path d="M8.1 7.6 20 18M8.1 16.4 20 6" />
    </svg>
  );
}
