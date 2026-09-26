/**
 * Changing how a section is laid out, as edit commands.
 *
 * Every viewport has a shape — the whole 9:16 frame, the upper or lower share
 * of a split, a square inset — and every crop drawn into one must have that
 * shape: the render refuses a crop of another shape rather than stretch a
 * face. So a change that gives a viewport a new shape carries its crops
 * along, each keeping its centre and its height (how close it is) while its
 * width follows the new shape.
 */
import type { EditIr } from '@clipmill/contracts';

import type { EditCommandJson } from '../daemon/client.js';
import { type LayoutMode, setLayout, swapPortraits } from './commands.js';

export type SavedLayout = NonNullable<EditIr['video']['segments']>[number]['layout'];
type Keyframe = NonNullable<SavedLayout['crop_path']>[number];
type Path = readonly Keyframe[];
export type FitBackground = NonNullable<SavedLayout['background']>;
export type Inset = NonNullable<SavedLayout['inset']>;
export type InsetCorner = Inset['corner'];

interface Shape {
  readonly width: number;
  readonly height: number;
}
interface Frame {
  readonly displayWidth: number;
  readonly displayHeight: number;
}

/** The split ratios offered, per mille of the height for the upper viewport. */
export const SPLITS = { min: 250, max: 750, even: 500 } as const;
/** How far a fitted picture may be zoomed, in percent. */
export const ZOOMS = { min: 100, max: 250 } as const;
/** How large an inset may be, per mille of the frame width. */
export const INSET_SIZES = { min: 200, max: 600, default: 360 } as const;
export const DEFAULT_INSET: Inset = { corner: 'top_right', size: INSET_SIZES.default };

/** A section's style as the command sets it: all four at once. */
export interface LayoutStyle {
  readonly split?: number | undefined;
  readonly background?: FitBackground | undefined;
  readonly zoom?: number | undefined;
  readonly inset?: Inset | undefined;
}

export function styleOf(layout: SavedLayout | undefined): LayoutStyle {
  return {
    split: layout?.split,
    background: layout?.background,
    zoom: layout?.zoom,
    inset: layout?.inset,
  };
}

export function setLayoutStyle(segmentId: string, style: LayoutStyle): EditCommandJson {
  return {
    op: 'set_layout_style',
    segment_id: segmentId,
    ...(style.split === undefined ? {} : { split: style.split }),
    ...(style.background === undefined ? {} : { background: style.background }),
    ...(style.zoom === undefined ? {} : { zoom: style.zoom }),
    ...(style.inset === undefined ? {} : { inset: style.inset }),
  };
}

/** The two stacked viewports' heights at a split, as the render computes them. */
export function viewportHeights(split: number, height: number): readonly [number, number] {
  const upper = Math.floor((height * split) / 1000) & ~1;
  return [upper, height - upper];
}

/**
 * The split that gives the upper viewport the recording's own shape, so a
 * screen share sits on top whole; clamped to the splits offered.
 */
export function recordingSplit(frame: Frame, output: Shape): number {
  const split = Math.round(
    (1000 * output.width * frame.displayHeight) / Math.max(1, frame.displayWidth * output.height),
  );
  return Math.min(SPLITS.max, Math.max(SPLITS.min, split));
}

/**
 * A path with each crop reshaped to `shape`: same centre, same height, the
 * width following the shape — and where that would leave the frame, the
 * widest crop of the shape that fits, still about the same centre.
 */
export function refit(path: Path, shape: Shape, frame: Frame): Keyframe[] {
  return path.map((keyframe) => ({ ...keyframe, rect: reshape(keyframe.rect, shape, frame) }));
}

function reshape(rect: Keyframe['rect'], shape: Shape, frame: Frame): Keyframe['rect'] {
  const aspect = shape.width / Math.max(1, shape.height);
  let height = Math.max(2, Math.min(frame.displayHeight, rect.height));
  height -= height % 2;
  let width = 2 * Math.round((height * aspect) / 2);
  if (width > frame.displayWidth) {
    width = frame.displayWidth - (frame.displayWidth % 2);
    height = Math.round(width / aspect);
    height -= height % 2;
  }
  width = Math.max(2, width);
  height = Math.max(2, height);
  const centreX = rect.x + rect.width / 2;
  const centreY = rect.y + rect.height / 2;
  return {
    x: clamp(Math.round(centreX - width / 2), 0, Math.max(0, frame.displayWidth - width)),
    y: clamp(Math.round(centreY - height / 2), 0, Math.max(0, frame.displayHeight - height)),
    width,
    height,
  };
}

/** The whole recording as one still crop, reshaped. */
function whole(shape: Shape, frame: Frame): Keyframe[] {
  return refit(
    [
      {
        t_ticks: 0,
        rect: { x: 0, y: 0, width: frame.displayWidth, height: frame.displayHeight },
      },
    ],
    shape,
    frame,
  );
}

/** A still crop about the frame's centre, `share` of its height tall, reshaped. */
function centred(shape: Shape, frame: Frame, share: number): Keyframe[] {
  const height = Math.round(frame.displayHeight * share);
  return refit(
    [
      {
        t_ticks: 0,
        rect: {
          x: Math.round(frame.displayWidth / 2 - height / 2),
          y: Math.round((frame.displayHeight - height) / 2),
          width: height,
          height,
        },
      },
    ],
    shape,
    frame,
  );
}

function replace(segmentId: string, path: Path, secondary: boolean): EditCommandJson {
  return {
    op: secondary ? 'replace_secondary_crop_path' : 'replace_crop_path',
    segment_id: segmentId,
    path,
  };
}

export type LayoutChoice = LayoutMode | 'screen_and_face';

/**
 * The commands that make a section this layout, its crops reshaped for the
 * viewports they will fill. `saved` is the section's layout as the document
 * holds it; `output` is the frame the program renders at.
 */
export function layoutCommands(
  segmentId: string,
  choice: LayoutChoice,
  saved: SavedLayout | undefined,
  output: Shape,
  frame: Frame,
): EditCommandJson[] {
  const primary = saved?.crop_path ?? [];
  const secondary = saved?.secondary_crop_path ?? [];
  const style = styleOf(saved);
  const full = { width: output.width, height: output.height };
  switch (choice) {
    case 'fit':
      return [setLayout('fit', segmentId)];
    case 'speaker_fill':
      return [
        setLayout('speaker_fill', segmentId),
        replace(
          segmentId,
          primary.length > 0 ? refit(primary, full, frame) : centred(full, frame, 1),
          false,
        ),
      ];
    case 'two_up':
    case 'screen_and_face': {
      const split =
        choice === 'screen_and_face' ? recordingSplit(frame, output) : (style.split ?? SPLITS.even);
      const [upper, lower] = viewportHeights(split, output.height);
      const top = { width: output.width, height: upper };
      const bottom = { width: output.width, height: lower };
      // A screen share shows the whole recording on top; the face below is
      // whoever the section was following, until it is dragged elsewhere.
      const upperPath =
        choice === 'screen_and_face'
          ? whole(top, frame)
          : primary.length > 0
            ? refit(primary, top, frame)
            : centred(top, frame, 1);
      const lowerSource = secondary.length > 0 && choice === 'two_up' ? secondary : primary;
      const lowerPath =
        lowerSource.length > 0 ? refit(lowerSource, bottom, frame) : centred(bottom, frame, 1);
      return [
        setLayout('two_up', segmentId),
        setLayoutStyle(segmentId, { ...style, split: split === SPLITS.even ? undefined : split }),
        replace(segmentId, upperPath, false),
        replace(segmentId, lowerPath, true),
      ];
    }
    case 'picture_in_picture': {
      const square = { width: 1, height: 1 };
      // The inset is the second person when there is one; otherwise the
      // middle of the frame, to be dragged onto whoever it should show.
      const insetPath =
        secondary.length > 0 ? refit(secondary, square, frame) : centred(square, frame, 0.6);
      return [
        setLayout('picture_in_picture', segmentId),
        setLayoutStyle(segmentId, { ...style, inset: style.inset ?? DEFAULT_INSET }),
        replace(segmentId, primary.length > 0 ? refit(primary, full, frame) : [], false),
        replace(segmentId, insetPath, true),
      ];
    }
  }
}

/**
 * A style for a two-person section, whose split may have moved: the style,
 * and both crops reshaped for the viewports that split makes.
 */
export function splitCommands(
  segmentId: string,
  style: LayoutStyle,
  saved: SavedLayout | undefined,
  output: Shape,
  frame: Frame,
): EditCommandJson[] {
  const split = style.split ?? SPLITS.even;
  const [upper, lower] = viewportHeights(split, output.height);
  return [
    setLayoutStyle(segmentId, { ...style, split: split === SPLITS.even ? undefined : split }),
    replace(
      segmentId,
      refit(saved?.crop_path ?? [], { width: output.width, height: upper }, frame),
      false,
    ),
    replace(
      segmentId,
      refit(saved?.secondary_crop_path ?? [], { width: output.width, height: lower }, frame),
      true,
    ),
  ];
}

/**
 * The two people exchanged between the viewports. At an even split both
 * viewports share a shape and the crops simply trade places; at any other
 * each crop is reshaped for the viewport it moves into.
 */
export function switchCommands(
  segmentId: string,
  saved: SavedLayout | undefined,
  output: Shape,
  frame: Frame,
): EditCommandJson[] {
  const split = saved?.split ?? SPLITS.even;
  if (split === SPLITS.even) return [swapPortraits(segmentId)];
  const [upper, lower] = viewportHeights(split, output.height);
  return [
    replace(
      segmentId,
      refit(saved?.secondary_crop_path ?? [], { width: output.width, height: upper }, frame),
      false,
    ),
    replace(
      segmentId,
      refit(saved?.crop_path ?? [], { width: output.width, height: lower }, frame),
      true,
    ),
  ];
}

function clamp(value: number, low: number, high: number): number {
  return Math.min(high, Math.max(low, value));
}
