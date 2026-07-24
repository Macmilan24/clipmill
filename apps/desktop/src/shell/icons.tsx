/**
 * Thin line icons drawn inline.
 *
 * An icon package would be a network dependency at install time and dead weight
 * in the bundle; these are the fifteen glyphs the shell actually uses, at the
 * design's 1.5px stroke.
 */
import type { JSX, SVGProps } from 'react';

export type IconProps = SVGProps<SVGSVGElement> & { readonly size?: number };

function Icon({ size = 18, children, ...rest }: IconProps): JSX.Element {
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
      {...rest}
    >
      {children}
    </svg>
  );
}

export type IconComponent = (props: IconProps) => JSX.Element;

/** The scissor-blade mark in the sidebar header. */
export const MarkIcon: IconComponent = (props) => (
  <Icon {...props} size={props.size ?? 20}>
    <circle cx="6" cy="6" r="2.5" />
    <circle cx="6" cy="18" r="2.5" />
    <path d="M8.1 7.6 20 18M8.1 16.4 20 6" />
  </Icon>
);

export const LibraryIcon: IconComponent = (props) => (
  <Icon {...props}>
    <path d="M4 5v14M8.5 5v14M13 6l4.5 12.5" />
    <path d="M3 20h18" />
  </Icon>
);

export const PlusSquareIcon: IconComponent = (props) => (
  <Icon {...props}>
    <rect x="3.5" y="3.5" width="17" height="17" rx="3" />
    <path d="M12 8.5v7M8.5 12h7" />
  </Icon>
);

export const SparkleRectIcon: IconComponent = (props) => (
  <Icon {...props}>
    <rect x="3.5" y="4.5" width="17" height="15" rx="3" />
    <path d="M12 8.2l1.1 2.5 2.5 1.1-2.5 1.1L12 15.5l-1.1-2.6-2.5-1.1 2.5-1.1z" />
  </Icon>
);

export const SlidersIcon: IconComponent = (props) => (
  <Icon {...props}>
    <path d="M3 7h11M18 7h3M3 17h4M11 17h10" />
    <circle cx="16" cy="7" r="2" />
    <circle cx="9" cy="17" r="2" />
  </Icon>
);

export const SearchSparkleIcon: IconComponent = (props) => (
  <Icon {...props}>
    <circle cx="10.5" cy="10.5" r="6" />
    <path d="M15 15l5 5" />
    <path d="M18.5 3.5l.7 1.6 1.6.7-1.6.7-.7 1.6-.7-1.6-1.6-.7 1.6-.7z" />
  </Icon>
);

export const SwatchIcon: IconComponent = (props) => (
  <Icon {...props}>
    <path d="M4 18V6a2 2 0 0 1 2-2h5v14a2 2 0 0 1-4 0" />
    <path d="M11 9h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H9" />
    <circle cx="7" cy="17" r="0.6" fill="currentColor" />
  </Icon>
);

export const CpuIcon: IconComponent = (props) => (
  <Icon {...props}>
    <rect x="6.5" y="6.5" width="11" height="11" rx="2" />
    <rect x="10" y="10" width="4" height="4" rx="0.8" />
    <path d="M10 3.5v3M14 3.5v3M10 17.5v3M14 17.5v3M3.5 10h3M3.5 14h3M17.5 10h3M17.5 14h3" />
  </Icon>
);

export const UploadIcon: IconComponent = (props) => (
  <Icon {...props}>
    <path d="M12 15V4M8.5 7.5 12 4l3.5 3.5" />
    <path d="M4 15v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3" />
  </Icon>
);

export const SettingsIcon: IconComponent = (props) => (
  <Icon {...props}>
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 14.5a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-1.8-.3 1.6 1.6 0 0 0-1 1.5v.2a2 2 0 1 1-4 0v-.1a1.6 1.6 0 0 0-1-1.5 1.6 1.6 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.6 1.6 0 0 0 .3-1.8 1.6 1.6 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.6 1.6 0 0 0 1.5-1 1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.6 1.6 0 0 0 1.8.3h.1a1.6 1.6 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.6 1.6 0 0 0 1 1.5 1.6 1.6 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0-.3 1.8v.1a1.6 1.6 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1z" />
  </Icon>
);

export const ShieldCheckIcon: IconComponent = (props) => (
  <Icon {...props} size={props.size ?? 14}>
    <path d="M12 3l7 3v5.5c0 4.3-3 8.2-7 9.5-4-1.3-7-5.2-7-9.5V6z" />
    <path d="M9 12l2 2 4-4" />
  </Icon>
);

export const SunIcon: IconComponent = (props) => (
  <Icon {...props} size={props.size ?? 16}>
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
  </Icon>
);

export const MoonIcon: IconComponent = (props) => (
  <Icon {...props} size={props.size ?? 16}>
    <path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5z" />
  </Icon>
);

export const RefreshIcon: IconComponent = (props) => (
  <Icon {...props} size={props.size ?? 16}>
    <path d="M20 11a8 8 0 1 0-.7 4.3" />
    <path d="M20 5v6h-6" />
  </Icon>
);

export const AlertIcon: IconComponent = (props) => (
  <Icon {...props} size={props.size ?? 16}>
    <path d="M12 4.5 21 19.5H3z" />
    <path d="M12 10v4M12 17h.01" />
  </Icon>
);

export const FilmIcon: IconComponent = (props) => (
  <Icon {...props} size={props.size ?? 24}>
    <rect x="3" y="5" width="18" height="14" rx="2" />
    <path d="M7 5v14M17 5v14M3 12h18" />
  </Icon>
);
