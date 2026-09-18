import { useCallback, useEffect, useRef, useState } from 'react';
import type { RefObject } from 'react';

/** Gain is auditioned over the proxy. Two-pass loudness remains the rendered file's authority. */
export function useDraftAudio(video: RefObject<HTMLVideoElement | null>, gainDb: number) {
  const graph = useRef<{ media: HTMLVideoElement; context: AudioContext; gain: GainNode } | null>(
    null,
  );
  const currentGain = useRef(gainDb);
  currentGain.current = gainDb;
  const [problem, setProblem] = useState<string | null>(null);
  const connect = useCallback(() => {
    const media = video.current;
    if (!media) return;
    if (typeof AudioContext === 'undefined') {
      if (currentGain.current !== 0)
        setProblem(
          'This player cannot audition gain changes. Use the rendered preview to check audio.',
        );
      return;
    }
    try {
      if (graph.current?.media !== media) {
        if (graph.current) void graph.current.context.close();
        const context = new AudioContext();
        const gain = context.createGain();
        context.createMediaElementSource(media).connect(gain).connect(context.destination);
        graph.current = { media, context, gain };
      }
      graph.current.gain.gain.setValueAtTime(
        10 ** (currentGain.current / 20),
        graph.current.context.currentTime,
      );
      void graph.current.context
        .resume()
        .catch(() =>
          setProblem('Audio could not resume. Try Play again, or listen to the rendered preview.'),
        );
      setProblem(null);
    } catch {
      setProblem('Gain audition is unavailable. Check audio in the rendered preview.');
    }
  }, [video]);
  useEffect(() => {
    const active = graph.current;
    if (active) active.gain.gain.setValueAtTime(10 ** (gainDb / 20), active.context.currentTime);
  }, [gainDb]);
  useEffect(
    () => () => {
      if (graph.current) void graph.current.context.close();
    },
    [],
  );
  return { connect, problem };
}
