import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { RefObject } from 'react';

import { RoomSamples, scheduleFades, type JoinAudition } from './joinAudio.js';
import { segmentAt } from './player.js';

interface Graph {
  readonly media: HTMLVideoElement;
  readonly context: AudioContext;
  readonly gain: GainNode;
  readonly edges: GainNode;
}

const level = (gainDb: number) => 10 ** (gainDb / 20);

/**
 * How far behind the picture the routed sound is heard: what the context
 * buffers plus what the output device adds, as far as the webview reports.
 */
function latencyOf(context: AudioContext): number {
  const buffered = Number.isFinite(context.baseLatency) ? context.baseLatency : 0;
  const output = Number.isFinite(context.outputLatency) ? context.outputLatency : 0;
  return Math.max(0, buffered) + Math.max(0, output);
}

/**
 * Gain is auditioned over the proxy. Two-pass loudness remains the rendered
 * file's authority.
 *
 * A cut in level plays through the element's own volume, which leaves sound
 * and picture in the player's own sync. Only a boost — more than the element's
 * full volume — needs Web Audio, and routing the element there puts the sound
 * behind the picture by the context's output latency: `lag` reports it, in
 * seconds, so the captions can follow what is heard. An element once routed
 * stays routed, so `lag` holds for the rest of its life.
 *
 * `boosts` says whether the edit raises the level anywhere; it is decided
 * before playing starts, since routing mid-play would click.
 */
export function useDraftAudio(
  video: RefObject<HTMLVideoElement | null>,
  gainDb: number,
  boosts = gainDb > 0,
  audition?: JoinAudition,
) {
  const graph = useRef<Graph | null>(null);
  const [connected, setConnected] = useState<Graph | null>(null);
  const plan = audition?.plan;
  const audioUrls = audition?.audioUrls;
  const hasFades = useMemo(
    () => plan?.segments.some((segment) => segment.fadeInTicks || segment.fadeOutTicks) ?? false,
    [plan],
  );
  const [samples, setSamples] = useState<RoomSamples | null>(null);
  const [roomProblem, setRoomProblem] = useState<string | null>(null);
  useEffect(() => {
    setRoomProblem(null);
    if (
      !plan ||
      !audioUrls ||
      !plan.segments.some((segment) => segment.roomToneStartTicks != null)
    ) {
      setSamples(null);
      return;
    }
    const reader = new RoomSamples(plan, audioUrls);
    setSamples(reader);
    return () => reader.controller.abort();
  }, [plan, audioUrls]);
  const currentGain = useRef(gainDb);
  currentGain.current = gainDb;
  const [problem, setProblem] = useState<string | null>(null);
  const [lag, setLag] = useState(0);
  const connect = useCallback(() => {
    const media = video.current;
    if (!media) return;
    const routed = graph.current?.media === media;
    if (!routed && !boosts && !hasFades) {
      media.volume = Math.min(1, level(currentGain.current));
      setLag(0);
      setProblem(null);
      return;
    }
    if (typeof AudioContext === 'undefined') {
      media.volume = Math.min(1, level(currentGain.current));
      setProblem(
        hasFades
          ? 'This player cannot audition join audio. Use the rendered preview to hear it.'
          : 'This player cannot play a boost louder than the recording. Use the rendered preview to check audio.',
      );
      return;
    }
    try {
      if (!routed) {
        if (graph.current) void graph.current.context.close();
        const context = new AudioContext({ latencyHint: 'interactive' });
        const gain = context.createGain();
        const edges = context.createGain();
        context.createMediaElementSource(media).connect(edges);
        edges.connect(gain);
        gain.connect(context.destination);
        graph.current = { media, context, gain, edges };
        setConnected(graph.current);
        media.volume = 1;
      }
      const active = graph.current!;
      active.gain.gain.setValueAtTime(level(currentGain.current), active.context.currentTime);
      void active.context
        .resume()
        .then(() => setLag(latencyOf(active.context)))
        .catch(() =>
          setProblem('Audio could not resume. Try Play again, or listen to the rendered preview.'),
        );
      setLag(latencyOf(active.context));
      setProblem(null);
    } catch {
      setProblem('Gain audition is unavailable. Check audio in the rendered preview.');
    }
  }, [video, boosts, hasFades]);
  useEffect(() => {
    const active = graph.current;
    const media = video.current;
    if (active && active.media === media) {
      active.gain.gain.setValueAtTime(level(gainDb), active.context.currentTime);
    } else if (media) {
      media.volume = Math.min(1, level(gainDb));
    }
  }, [video, gainDb]);
  const segment = plan && audition ? segmentAt(plan, audition.frame) : null;
  useEffect(() => {
    if (!plan || !segment || !hasFades) return;
    const index = plan.segments.indexOf(segment);
    const first = samples?.get(index);
    const next = samples?.get(index + 1);
    let live = true;
    const unavailable = () => {
      if (live && !samples?.controller.signal.aborted)
        setRoomProblem('Room tone could not be loaded. Check this join in the rendered preview.');
    };
    void first?.catch(unavailable);
    void next?.catch(unavailable);
    const proxy = plan.proxies.find((item) => item.sourceFingerprint === segment.sourceFingerprint);
    if (!connected || !proxy)
      return () => {
        live = false;
      };
    const { context, media, edges, gain } = connected;
    const fadeIn = (segment.fadeInTicks ?? 0) / 90_000;
    const fadeOut = (segment.fadeOutTicks ?? 0) / 90_000;
    const length = (segment.outTicks - segment.inTicks) / 90_000;
    const sourceIn = (segment.inTicks - proxy.coverageStartTicks) / 90_000;
    const previousFade = (plan.segments[index - 1]?.fadeOutTicks ?? 0) / 90_000;
    let generation = 0;
    const playing = new Set<AudioBufferSourceNode>();
    function stop() {
      generation++;
      for (const tone of playing) {
        tone.stop();
        tone.disconnect();
      }
      playing.clear();
      edges.gain.cancelScheduledValues(0);
      edges.gain.setValueAtTime(1, context.currentTime);
    }
    function schedule() {
      stop();
      if (media.paused || media.seeking || media.playbackRate <= 0) return;
      const offset = media.currentTime - sourceIn;
      scheduleFades(
        edges.gain,
        context.currentTime,
        media.playbackRate,
        offset,
        length,
        fadeIn,
        fadeOut,
      );
      if (media.muted || media.volume === 0) return;
      const current = generation;
      function portion(
        excerpt: Promise<readonly Float32Array[]> | undefined,
        at: number,
        start: number,
        duration: number,
      ) {
        if (!excerpt || duration <= 0 || offset >= at + duration) return;
        void excerpt
          .then((channels) => {
            if (!live || generation !== current || media.paused || media.seeking) return;
            const position = media.currentTime - sourceIn;
            const elapsed = Math.max(0, position - at);
            if (elapsed >= duration) return;
            const buffer = context.createBuffer(channels.length, channels[0]!.length, 48_000);
            channels.forEach((channel, i) => buffer.getChannelData(i).set(channel));
            const tone = context.createBufferSource();
            tone.buffer = buffer;
            tone.playbackRate.value = media.playbackRate;
            tone.connect(gain);
            tone.addEventListener(
              'ended',
              () => {
                playing.delete(tone);
                tone.disconnect();
              },
              { once: true },
            );
            playing.add(tone);
            tone.start(
              context.currentTime + Math.max(0, at - position) / media.playbackRate,
              start + elapsed,
              duration - elapsed,
            );
          })
          .catch(unavailable);
      }
      portion(first, 0, previousFade, fadeIn);
      portion(next, length - fadeOut, 0, fadeOut);
    }
    const starts = ['playing', 'seeked', 'ratechange', 'volumechange'];
    const stops = ['pause', 'seeking', 'waiting', 'ended'];
    starts.forEach((name) => media.addEventListener(name, schedule));
    stops.forEach((name) => media.addEventListener(name, stop));
    schedule();
    return () => {
      live = false;
      starts.forEach((name) => media.removeEventListener(name, schedule));
      stops.forEach((name) => media.removeEventListener(name, stop));
      stop();
    };
  }, [plan, segment, connected, samples, hasFades]);
  useEffect(
    () => () => {
      if (graph.current) void graph.current.context.close();
    },
    [],
  );
  return { connect, problem: problem ?? roomProblem, lag };
}
