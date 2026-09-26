import { useCallback, useEffect, useRef, useState } from 'react';
import type { RefObject } from 'react';

interface Graph {
  readonly media: HTMLVideoElement;
  readonly context: AudioContext;
  readonly gain: GainNode;
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
) {
  const graph = useRef<Graph | null>(null);
  const currentGain = useRef(gainDb);
  currentGain.current = gainDb;
  const [problem, setProblem] = useState<string | null>(null);
  const [lag, setLag] = useState(0);
  const connect = useCallback(() => {
    const media = video.current;
    if (!media) return;
    const routed = graph.current?.media === media;
    if (!routed && !boosts) {
      media.volume = Math.min(1, level(currentGain.current));
      setLag(0);
      setProblem(null);
      return;
    }
    if (typeof AudioContext === 'undefined') {
      media.volume = Math.min(1, level(currentGain.current));
      setProblem(
        'This player cannot play a boost louder than the recording. Use the rendered preview to check audio.',
      );
      return;
    }
    try {
      if (!routed) {
        if (graph.current) void graph.current.context.close();
        const context = new AudioContext({ latencyHint: 'interactive' });
        const gain = context.createGain();
        context.createMediaElementSource(media).connect(gain).connect(context.destination);
        graph.current = { media, context, gain };
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
  }, [video, boosts]);
  useEffect(() => {
    const active = graph.current;
    const media = video.current;
    if (active && active.media === media) {
      active.gain.gain.setValueAtTime(level(gainDb), active.context.currentTime);
    } else if (media) {
      media.volume = Math.min(1, level(gainDb));
    }
  }, [video, gainDb]);
  useEffect(
    () => () => {
      if (graph.current) void graph.current.context.close();
    },
    [],
  );
  return { connect, problem, lag };
}
