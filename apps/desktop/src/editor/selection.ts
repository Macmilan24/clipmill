/** What the Editor has selected: one thing, which decides the panel and what Delete removes. */

/** A run of spoken words, by their position in the program's word list. */
export interface WordRange {
  readonly first: number;
  readonly last: number;
}

export type EditorSelection =
  | { readonly kind: 'clip' }
  | { readonly kind: 'cue'; readonly cueId: string }
  | { readonly kind: 'section'; readonly segmentId: string }
  | {
      readonly kind: 'keyframe';
      readonly segmentId: string;
      readonly tTicks: number;
      readonly secondary: boolean;
    }
  | { readonly kind: 'gain'; readonly tTicks: number }
  | { readonly kind: 'overlay'; readonly overlayId: string }
  | { readonly kind: 'cutaway'; readonly cutawayId: string }
  | { readonly kind: 'words'; readonly range: WordRange };

export const NOTHING: EditorSelection = { kind: 'clip' };

/** The properties tab a selection belongs on. */
export type PropertiesTab = 'captions' | 'text' | 'framing' | 'audio' | 'brand';

export function tabFor(selection: EditorSelection): PropertiesTab | null {
  switch (selection.kind) {
    case 'cue':
      return 'captions';
    case 'section':
    case 'keyframe':
    case 'cutaway':
      return 'framing';
    case 'gain':
      return 'audio';
    case 'overlay':
      return 'text';
    default:
      return null;
  }
}

export function positions(range: WordRange): number[] {
  return Array.from({ length: range.last - range.first + 1 }, (_, index) => range.first + index);
}
