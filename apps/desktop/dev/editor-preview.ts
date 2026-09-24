/**
 * A stand-in for the daemon in the UI preview: applies the handful of Editor
 * commands whose effect can be shown without the renderer, so the panels and
 * the timeline respond. Anything else is reported as not saved.
 */
import type { EditIr } from '@clipmill/contracts';
import type { EditCommandJson, PreviewPlan } from '../src/daemon/client.js';

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
    default:
      return null;
  }
}
