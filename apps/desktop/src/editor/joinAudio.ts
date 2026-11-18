import type { PreviewPlan } from '../daemon/client.js';

const TICKS = 90_000;

interface WaveFormat {
  readonly channels: number;
  readonly rate: number;
  readonly offset: number;
}

/** Only the signed 16-bit rendition made by ingest is accepted. */
export function waveFormat(bytes: ArrayBuffer): WaveFormat {
  const view = new DataView(bytes);
  function name(at: number) {
    return String.fromCharCode(...new Uint8Array(bytes, at, 4));
  }
  if (bytes.byteLength < 12 || name(0) !== 'RIFF' || name(8) !== 'WAVE') {
    throw new Error('Invalid source audio header.');
  }
  let channels = 0;
  let rate = 0;
  for (let at = 12; at + 8 <= bytes.byteLength;) {
    const kind = name(at);
    const length = view.getUint32(at + 4, true);
    const content = at + 8;
    if (kind === 'fmt ' && length >= 16 && content + 16 <= bytes.byteLength) {
      channels = view.getUint16(content + 2, true);
      rate = view.getUint32(content + 4, true);
      if (
        view.getUint16(content, true) !== 1 ||
        view.getUint16(content + 14, true) !== 16 ||
        channels !== 2 ||
        rate !== 48_000
      )
        throw new Error('Unsupported source audio format.');
    }
    if (kind === 'data' && channels && rate) return { channels, rate, offset: content };
    at = content + length + (length % 2);
  }
  throw new Error('Missing source audio payload.');
}

async function byteRange(url: string, start: number, end: number, signal: AbortSignal) {
  const reply = await fetch(url, { signal, headers: { Range: `bytes=${start}-${end}` } });
  const length = Number(reply.headers.get('Content-Length'));
  const expected = end - start + 1;
  if (!(
    reply.status === 206 ||
    (start === 0 && reply.status === 200 && length > 0 && length <= expected)
  )) {
    void reply.body?.cancel();
    throw new Error('The source audio range is unavailable.');
  }
  const bytes = await reply.arrayBuffer();
  if (bytes.byteLength > expected) throw new Error('Unexpected source audio range.');
  return bytes;
}

/** FFmpeg's qsin gain at a fractional position on a rising or falling ramp. */
export function quarterSine(fraction: number, rise: boolean) {
  const position = Math.min(1, Math.max(0, fraction));
  return Math.sin(((rise ? position : 1 - position) * Math.PI) / 2);
}

/** Cache headers and only the sub-second excerpts a nearby join actually uses. */
export class RoomSamples {
  readonly controller = new AbortController();
  private readonly formats = new Map<string, Promise<WaveFormat>>();
  private readonly excerpts = new Map<string, Promise<readonly Float32Array[]>>();

  constructor(
    private readonly plan: PreviewPlan,
    private readonly urls: ReadonlyMap<string, string>,
  ) {}

  get(index: number): Promise<readonly Float32Array[]> | undefined {
    const entry = this.plan.segments[index];
    const start = entry?.roomToneStartTicks;
    const proxy = this.plan.proxies.find(
      (item) => item.sourceFingerprint === entry?.sourceFingerprint,
    );
    if (!entry || start == null || !proxy || index < 1) return;
    const url = this.urls.get(entry.sourceFingerprint);
    if (!url) return Promise.reject(new Error('The room-tone rendition is unavailable.'));
    const rise = this.plan.segments[index - 1]?.fadeOutTicks ?? 0;
    const fall = entry.fadeInTicks ?? 0;
    if (rise <= 0 || fall <= 0 || rise + fall > TICKS / 2) return;
    const startTicks = start - (proxy.audioStartTicks ?? 0);
    if (startTicks < 0)
      return Promise.reject(new Error('The room-tone range is outside the rendition.'));
    const key = [url, startTicks, rise, fall].join('|');
    let excerpt = this.excerpts.get(key);
    if (!excerpt) {
      let format = this.formats.get(url);
      if (!format) {
        format = byteRange(url, 0, 4095, this.controller.signal).then(waveFormat);
        this.formats.set(url, format);
      }
      excerpt = format.then(async ({ channels, rate, offset }) => {
        const frames = Math.ceil(((rise + fall) * rate) / TICKS);
        const first = offset + Math.round((startTicks * rate) / TICKS) * channels * 2;
        const bytes = await byteRange(
          url,
          first,
          first + frames * channels * 2 - 1,
          this.controller.signal,
        );
        if (bytes.byteLength !== frames * channels * 2)
          throw new Error('The room-tone range is incomplete.');
        const view = new DataView(bytes);
        const data = Array.from({ length: channels }, () => new Float32Array(frames));
        for (let frame = 0; frame < frames; frame++) {
          const tick = (frame * TICKS) / rate;
          const gain =
            tick < rise ? quarterSine(tick / rise, true) : quarterSine((tick - rise) / fall, false);
          for (let channel = 0; channel < channels; channel++) {
            data[channel]![frame] =
              (gain * view.getInt16((frame * channels + channel) * 2, true)) / 32768;
          }
        }
        return data;
      });
      this.excerpts.set(key, excerpt);
    }
    return excerpt;
  }
}

/** Schedule from the current source time, including scrubs into the middle of a ramp. */
export function scheduleFades(
  param: AudioParam,
  now: number,
  speed: number,
  offset: number,
  length: number,
  fadeIn: number,
  fadeOut: number,
) {
  param.cancelScheduledValues(0);
  const outgoing = length - fadeOut;
  const gain =
    offset < fadeIn && fadeIn > 0
      ? quarterSine(offset / fadeIn, true)
      : offset >= outgoing && fadeOut > 0
        ? quarterSine((offset - outgoing) / fadeOut, false)
        : 1;
  param.setValueAtTime(gain, now);
  function ramp(at: number, duration: number, rise: boolean) {
    if (duration <= 0 || offset >= at + duration) return;
    const elapsed = Math.max(0, offset - at);
    const remaining = duration - elapsed;
    const values = Float32Array.from({ length: 65 }, (_, i) =>
      quarterSine((elapsed + (remaining * i) / 64) / duration, rise),
    );
    param.setValueCurveAtTime(values, now + Math.max(0, at - offset) / speed, remaining / speed);
  }
  ramp(0, fadeIn, true);
  ramp(outgoing, fadeOut, false);
}

export interface JoinAudition {
  readonly plan: PreviewPlan | null;
  readonly frame: number;
  readonly audioUrls?: ReadonlyMap<string, string> | undefined;
}
