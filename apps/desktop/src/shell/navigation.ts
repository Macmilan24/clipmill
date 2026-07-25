import {
  CpuIcon,
  type IconComponent,
  LibraryIcon,
  PlusSquareIcon,
  SearchSparkleIcon,
  SettingsIcon,
  SlidersIcon,
  SparkleRectIcon,
  SwatchIcon,
  UploadIcon,
} from './icons.js';

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
  readonly icon: IconComponent;
  readonly availability: Availability;
}

/**
 * Order and labels are fixed by the design and must not be regrouped, renamed,
 * or reordered.
 */
export const NAV_SECTIONS: readonly NavSection[] = [
  {
    id: 'library',
    label: 'Library',
    breadcrumb: 'Library',
    icon: LibraryIcon,
    availability: {
      kind: 'planned',
      phase: 1,
      summary: 'Projects appear here once the import and analysis pipeline can produce them.',
    },
  },
  {
    id: 'new-project',
    label: 'New Project',
    breadcrumb: 'New Project',
    icon: PlusSquareIcon,
    availability: {
      kind: 'planned',
      phase: 1,
      summary:
        'Importing a long-form source and compiling creative direction into scoring parameters.',
    },
  },
  {
    id: 'results',
    label: 'Results',
    breadcrumb: 'Results',
    icon: SparkleRectIcon,
    availability: {
      kind: 'planned',
      phase: 1,
      summary: 'Ranked clip candidates with the evidence behind each score.',
    },
  },
  {
    id: 'editor',
    label: 'Editor',
    breadcrumb: 'Editor',
    icon: SlidersIcon,
    availability: {
      kind: 'planned',
      phase: 1,
      summary: 'Timeline, vertical reframing, captions, and the render path.',
    },
  },
  {
    id: 'discovery',
    label: 'Discovery',
    breadcrumb: 'Discovery',
    icon: SearchSparkleIcon,
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
    icon: SwatchIcon,
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
    icon: CpuIcon,
    availability: { kind: 'live' },
  },
  {
    id: 'export',
    label: 'Export',
    breadcrumb: 'Export',
    icon: UploadIcon,
    availability: {
      kind: 'planned',
      phase: 1,
      summary: 'Validating and rendering approved clip packages.',
    },
  },
  {
    id: 'settings',
    label: 'Settings',
    breadcrumb: 'Settings',
    icon: SettingsIcon,
    availability: {
      kind: 'planned',
      phase: 1,
      summary: 'Storage, privacy, security, and application behaviour.',
    },
  },
];

export const DEFAULT_SECTION_ID = 'models';

export function findSection(id: string): NavSection {
  return NAV_SECTIONS.find((section) => section.id === id) ?? NAV_SECTIONS[6]!;
}
