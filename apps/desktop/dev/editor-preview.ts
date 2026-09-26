/**
 * A stand-in for the daemon in the UI preview: applies the handful of Editor
 * commands whose effect can be shown without the renderer, so the panels and
 * the timeline respond. Anything else is reported as not saved.
 */
import type { EditIr } from '@clipmill/contracts';
import type { EditCommandJson, PreviewPlan } from '../src/daemon/client.js';
import { type FrameShape, frameOfShape } from '../src/editor/layouts.js';

const SECOND = 90_000;

export interface PreviewEdit {
  readonly plan: PreviewPlan;
  readonly document: EditIr;
}

const LOOKS: Record<string, Partial<NonNullable<PreviewPlan['captionStyle']>>> = {
  'clipmill.captions.clean.v1': { bold: true, boxed: false, spoken: '#ffd65c' },
  'clipmill.captions.minimal.v1': { bold: false, boxed: false, spoken: '#ffffff', outlineWidth: 3 },
  'clipmill.captions.boxed.v1': { bold: true, boxed: true, spoken: '#ffd65c', outline: '#000000' },
};

export function applyPreview(edit: PreviewEdit, command: EditCommandJson): PreviewEdit | null {
  const { plan, document } = edit;
  const frame = (ticks: number) => Math.floor((ticks * plan.rateNum) / plan.rateDen / SECOND);
  const bumped = (next: Partial<PreviewPlan>): PreviewPlan => ({
    ...plan,
    ...next,
    revision: plan.revision + 1,
  });
  switch (command.op) {
    case 'batch': {
      let current: PreviewEdit | null = edit;
      for (const inner of command.commands as EditCommandJson[]) {
        current = current ? applyPreview(current, inner) : null;
      }
      return current;
    }
    case 'set_word_text':
      return {
        plan: bumped({
          cues: plan.cues.map((cue) => ({
            ...cue,
            lines: cue.lines.map((line) =>
              line.map((word) =>
                word.wordId === command.word_id ? { ...word, text: String(command.text) } : word,
              ),
            ),
          })),
        }),
        document: {
          ...document,
          captions: {
            ...document.captions,
            ...Object.fromEntries(
              (['cues', 'burn_in'] as const).map((list) => [
                list,
                document.captions[list]?.map((cue) => ({
                  ...cue,
                  lines: cue.lines.map((line) => ({
                    words: line.words.map((word) =>
                      word.word_id === command.word_id
                        ? { ...word, text: String(command.text) }
                        : word,
                    ) as typeof line.words,
                  })),
                })),
              ]),
            ),
          },
        },
      };
    case 'set_caption_style': {
      const styleRef = String(command.style_ref);
      return {
        plan: bumped({
          captionStyle: { ...plan.captionStyle!, ...LOOKS[styleRef], styleRef },
        }),
        document: { ...document, captions: { ...document.captions, style_ref: styleRef } },
      };
    }
    case 'set_caption_options': {
      const options = command.options as NonNullable<EditIr['captions']['options']>;
      const caseOf = (text: string) =>
        options.text_case === 'upper'
          ? text.toUpperCase()
          : options.text_case === 'lower'
            ? text.toLowerCase()
            : text;
      const words = new Map(
        (document.captions.burn_in ?? []).flatMap((cue) =>
          cue.lines.flatMap((line) => line.words.map((word) => [word.word_id, word.text] as const)),
        ),
      );
      return {
        plan: bumped({
          captionStyle: {
            ...plan.captionStyle!,
            ...(options.font_size ? { fontSize: options.font_size } : {}),
            ...(options.spoken ? { spoken: options.spoken } : {}),
            ...(options.unspoken ? { unspoken: options.unspoken } : {}),
            ...(options.outline ? { outline: options.outline } : {}),
          },
          cues: plan.cues.map((cue) => ({
            ...cue,
            lines: cue.lines.map((line) =>
              line.map((word) => ({ ...word, text: caseOf(words.get(word.wordId) ?? word.text) })),
            ),
          })),
        }),
        document: { ...document, captions: { ...document.captions, options } },
      };
    }
    case 'set_cue_region':
      return {
        plan: bumped({
          cues: plan.cues.map((cue) =>
            cue.cueId === command.cue_id ? { ...cue, region: String(command.region) } : cue,
          ),
        }),
        document,
      };
    case 'set_cue_timing': {
      const start = Number(command.start_ticks);
      const end = Number(command.end_ticks);
      const retime = <T extends { cue_id: string }>(cue: T) =>
        cue.cue_id === command.cue_id ? { ...cue, start_ticks: start, end_ticks: end } : cue;
      return {
        plan: bumped({
          cues: plan.cues.map((cue) =>
            cue.cueId === command.cue_id
              ? { ...cue, firstFrame: frame(start), endFrame: frame(end) }
              : cue,
          ),
        }),
        document: {
          ...document,
          captions: {
            ...document.captions,
            cues: (document.captions.cues ?? []).map(retime),
            burn_in: (document.captions.burn_in ?? []).map(retime),
          },
        },
      };
    }
    case 'set_gain':
    case 'remove_gain_point': {
      const ticks = Number(command.t_ticks);
      const kept = (document.audio.gain_curve ?? []).filter((point) => point.t_ticks !== ticks);
      const curve =
        command.op === 'set_gain'
          ? [...kept, { t_ticks: ticks, gain_db: Number(command.gain_db) }].toSorted(
              (a, b) => a.t_ticks - b.t_ticks,
            )
          : kept;
      return {
        plan: bumped({
          gain: curve.map((point) => ({ frame: frame(point.t_ticks), gainDb: point.gain_db })),
        }),
        document: { ...document, audio: { ...document.audio, gain_curve: curve } },
      };
    }
    case 'add_overlay':
    case 'remove_overlay':
    case 'set_overlay': {
      const overlays = [...(document.overlays ?? [])];
      if (command.op === 'add_overlay') {
        const added = command.overlay as (typeof overlays)[number];
        overlays.splice(typeof command.at === 'number' ? command.at : overlays.length, 0, added);
      } else if (command.op === 'remove_overlay') {
        const at = overlays.findIndex((item) => item.overlay_id === command.overlay_id);
        if (at < 0) return null;
        overlays.splice(at, 1);
      } else {
        const changed = command.overlay as (typeof overlays)[number];
        const at = overlays.findIndex((item) => item.overlay_id === changed.overlay_id);
        if (at < 0) return null;
        overlays[at] = changed;
      }
      const next = { ...document, overlays };
      return {
        plan: bumped({
          overlays: overlays.map((overlay) => ({
            overlayId: overlay.overlay_id,
            startTicks: overlay.start_ticks,
            endTicks: overlay.end_ticks,
            firstFrame: frame(overlay.start_ticks),
            endFrame: Math.min(plan.frameCount, frame(overlay.end_ticks)),
            text: overlay.content.text,
            role: overlay.content.role === 'hook' ? 'hook' : 'label',
            x: overlay.content.x,
            y: overlay.content.y,
            size: overlay.content.size,
            colour: overlay.content.colour,
            plate: overlay.content.plate ?? null,
          })),
        }),
        document: next,
      };
    }
    case 'set_frame_shape': {
      const shape = command.shape as FrameShape;
      const next = { ...document, video: { ...document.video, shape } };
      const resized = { ...plan, ...frameOfShape(shape) };
      return {
        plan: bumped({ ...frameOfShape(shape), ...framing(resized, next) }),
        document: next,
      };
    }
    case 'set_layout':
    case 'set_layout_style':
    case 'replace_crop_path':
    case 'replace_secondary_crop_path':
    case 'set_crop_keyframe':
    case 'set_secondary_crop_keyframe':
    case 'swap_portraits': {
      const segments = (document.video.segments ?? []).map((segment) =>
        segment.segment_id === command.segment_id
          ? Object.assign({}, segment, { layout: relaid(segment.layout, command) })
          : segment,
      );
      const next = { ...document, video: { ...document.video, segments } };
      return { plan: bumped(framing(plan, next)), document: next };
    }
    default:
      return null;
  }
}

type Layout = NonNullable<EditIr['video']['segments']>[number]['layout'];
type Keyframe = NonNullable<Layout['crop_path']>[number];
type Crop = readonly [number, number, number, number];

/** A section's layout after one framing command, as the daemon applies it. */
function relaid(layout: Layout, command: EditCommandJson): Layout {
  switch (command.op) {
    case 'set_layout':
      return { ...layout, state: command.state as Layout['state'] };
    case 'set_layout_style': {
      const {
        split: _split,
        background: _background,
        zoom: _zoom,
        inset: _inset,
        ...rest
      } = layout;
      return {
        ...rest,
        ...(command.split === undefined ? {} : { split: command.split as number }),
        ...(command.background === undefined
          ? {}
          : { background: command.background as NonNullable<Layout['background']> }),
        ...(command.zoom === undefined ? {} : { zoom: command.zoom as number }),
        ...(command.inset === undefined
          ? {}
          : { inset: command.inset as NonNullable<Layout['inset']> }),
      };
    }
    case 'replace_crop_path':
      return { ...layout, crop_path: command.path as Keyframe[] };
    case 'replace_secondary_crop_path':
      return { ...layout, secondary_crop_path: command.path as Keyframe[] };
    case 'set_crop_keyframe':
    case 'set_secondary_crop_keyframe': {
      const key = command.op === 'set_crop_keyframe' ? 'crop_path' : 'secondary_crop_path';
      const at = Number(command.t_ticks);
      const path = [
        ...(layout[key] ?? []).filter((keyframe) => keyframe.t_ticks !== at),
        { t_ticks: at, rect: command.rect as Keyframe['rect'] },
      ].toSorted((a, b) => a.t_ticks - b.t_ticks);
      return { ...layout, [key]: path };
    }
    case 'swap_portraits':
      return {
        ...layout,
        crop_path: layout.secondary_crop_path ?? [],
        secondary_crop_path: layout.crop_path ?? [],
      };
    default:
      return layout;
  }
}

/**
 * Each frame's crops and each section's layout geometry, from the document,
 * the way the render's preview plan works them out (a stand-in, for looks).
 */
function framing(plan: PreviewPlan, document: EditIr): Partial<PreviewPlan> {
  const crops: (Crop | null)[] = [...plan.crops];
  const secondaryCrops: (Crop | null)[] = [...(plan.secondaryCrops ?? plan.crops.map(() => null))];
  const segments = plan.segments.map((segment) => {
    const layout = document.video.segments?.find(
      (saved) => saved.segment_id === segment.segmentId,
    )?.layout;
    if (!layout) return segment;
    for (let at = segment.firstFrame; at < segment.endFrame; at += 1) {
      const local = ((at - segment.firstFrame) * SECOND * plan.rateDen) / plan.rateNum;
      crops[at] = layout.state === 'fit' ? null : along(layout.crop_path, local);
      secondaryCrops[at] =
        layout.state === 'two_up' || layout.state === 'picture_in_picture'
          ? along(layout.secondary_crop_path, local)
          : null;
    }
    const inset = layout.inset ?? { corner: 'top_right', size: 360 };
    const short = Math.min(plan.width, plan.height);
    const tall = plan.height > plan.width;
    const side = Math.floor((short * inset.size) / 1000) & ~1;
    const margin = Math.floor((short * 40) / 1000) & ~1;
    const left = inset.corner.endsWith('left') ? margin : plan.width - margin - side;
    const top = inset.corner.startsWith('top')
      ? tall
        ? Math.floor((plan.height * 90) / 1000) & ~1
        : margin
      : (plan.height - Math.floor((plan.height * (tall ? 260 : 200)) / 1000) - side) & ~1;
    const across = plan.width > plan.height;
    return {
      ...segment,
      layout: layout.state,
      hasTwoUpPaths:
        (layout.crop_path?.length ?? 0) > 0 && (layout.secondary_crop_path?.length ?? 0) > 0,
      upperHeight:
        layout.state === 'two_up'
          ? Math.floor(((across ? plan.width : plan.height) * (layout.split ?? 500)) / 1000) & ~1
          : 0,
      inset: layout.state === 'picture_in_picture' ? ([left, top, side] as const) : null,
      backgroundColour: layout.background?.kind === 'colour' ? layout.background.colour : null,
      zoomPercent: layout.zoom ?? 100,
    };
  });
  return { crops, secondaryCrops, segments };
}

/** A path's crop at a segment-local tick, straight between keyframes. */
function along(path: readonly Keyframe[] | undefined, ticks: number): Crop | null {
  if (!path || path.length === 0) return null;
  let before = path[0]!;
  let after = path.at(-1)!;
  for (const keyframe of path) {
    if (keyframe.t_ticks <= ticks) before = keyframe;
    if (keyframe.t_ticks >= ticks) {
      after = keyframe;
      break;
    }
  }
  const span = after.t_ticks - before.t_ticks;
  const share = span > 0 ? (ticks - before.t_ticks) / span : 0;
  const mix = (from: number, to: number) => Math.round(from + (to - from) * share);
  return [
    mix(before.rect.x, after.rect.x),
    mix(before.rect.y, after.rect.y),
    mix(before.rect.width, after.rect.width),
    mix(before.rect.height, after.rect.height),
  ];
}
