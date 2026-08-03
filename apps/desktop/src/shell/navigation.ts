import {
  Cpu,
  Library,
  type LucideIcon,
  Palette,
  Search,
  Settings,
  SlidersHorizontal,
  Sparkles,
  SquarePlus,
  Upload,
} from 'lucide-react';

/**
 * Either the section is backed by something real today, or it names the phase
 * that will build it. Phase 0 ships one working screen; the honest thing is to
 * say so on the other eight rather than mock up features that do not exist.
 */
export type Availability =
  | { readonly kind: 'live' }
  | { readonly kind: 'planned'; readonly phase: number; readonly summary: string };

export interface NavSection {
  readonly id: string;
  readonly label: string;
  readonly breadcrumb: string;
  readonly icon: LucideIcon;
  readonly availability: Availability;
}

/**
 * Order, labels, and icons are fixed by the design and must not be regrouped,
 * renamed, or reordered.
 */
export const NAV_SECTIONS: readonly NavSection[] = [
  {
    id: 'library',
    label: 'Library',
    breadcrumb: 'Library',
    icon: Library,
    availability: { kind: 'live' },
  },
  {
    id: 'new-project',
    label: 'New Project',
    breadcrumb: 'New Project',
    icon: SquarePlus,
    availability: { kind: 'live' },
  },
  {
    id: 'results',
    label: 'Results',
    breadcrumb: 'Results',
    icon: Sparkles,
    availability: { kind: 'live' },
  },
  {
    id: 'editor',
    label: 'Editor',
    breadcrumb: 'Editor',
    icon: SlidersHorizontal,
    availability: { kind: 'live' },
  },
  {
    id: 'discovery',
    label: 'Discovery',
    breadcrumb: 'Discovery',
    icon: Search,
    availability: {
      kind: 'planned',
      phase: 2,
      summary: 'Natural-language search across the transcript for further moments.',
    },
  },
  {
    id: 'brand',
    label: 'Brand',
    breadcrumb: 'Brand',
    icon: Palette,
    availability: {
      kind: 'planned',
      phase: 4,
      summary: 'Reusable visual and metadata presets applied across exports.',
    },
  },
  {
    id: 'models',
    label: 'Models',
    breadcrumb: 'Models & Device',
    icon: Cpu,
    availability: { kind: 'live' },
  },
  {
    id: 'export',
    label: 'Export',
    breadcrumb: 'Export',
    icon: Upload,
    availability: { kind: 'live' },
  },
  {
    id: 'settings',
    label: 'Settings',
    breadcrumb: 'Settings',
    icon: Settings,
    availability: { kind: 'live' },
  },
];

export const DEFAULT_SECTION_ID = 'models';

export function findSection(id: string): NavSection {
  return NAV_SECTIONS.find((section) => section.id === id) ?? NAV_SECTIONS[6]!;
}
