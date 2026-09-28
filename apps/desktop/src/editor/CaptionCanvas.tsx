/**
 * The burned-in captions, drawn by libass from the ASS the export writes.
 *
 * The editor used to draw captions with CSS: close to the export, never the
 * same — outline, box, line spacing and every highlight style were a second
 * implementation of what libass does in the render. This draws the render's
 * own subtitle script with libass compiled to WebAssembly (JASSUB), so the
 * preview is the export's pixels by construction rather than by comparison.
 *
 * JASSUB owns its canvas once it has it: it transfers the canvas to a worker
 * and removes it when destroyed. So the canvas is made here, outside React's
 * tree, inside a box React does own. Where WebAssembly workers or offscreen
 * canvases are missing the layer reports that it cannot draw, and the editor
 * keeps its CSS captions instead of showing nothing.
 */
import { type JSX, useEffect, useRef, useState } from 'react';

/** A caption face and where the player can load it from. */
export interface CaptionFace {
  readonly family: string;
  readonly url: string;
}

export interface CaptionCanvasProps {
  /** The subtitle script to draw: the plan's, or a look still being tried. */
  readonly ass: string;
  /** Every installed caption face; the one a style names is loaded first. */
  readonly faces: readonly CaptionFace[];
  /** The family the captions are set in now. */
  readonly family: string;
  /** Program time to draw, in seconds. */
  readonly seconds: number;
  /** The output frame the script's coordinates describe. */
  readonly frameWidth: number;
  readonly frameHeight: number;
  /** Told whether libass is drawing, so the CSS captions can step aside. */
  readonly onDrawing?: (drawing: boolean) => void;
}

type Instance = import('jassub').default;

/** Whether this page can run the player at all. */
export function canDrawExactCaptions(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof Worker === 'function' &&
    typeof WebAssembly === 'object' &&
    typeof OffscreenCanvas === 'function' &&
    typeof HTMLCanvasElement !== 'undefined' &&
    'transferControlToOffscreen' in HTMLCanvasElement.prototype
  );
}

export function CaptionCanvas({
  ass,
  faces,
  family,
  seconds,
  frameWidth,
  frameHeight,
  onDrawing,
}: CaptionCanvasProps): JSX.Element {
  const box = useRef<HTMLDivElement | null>(null);
  const instance = useRef<Instance | null>(null);
  const [ready, setReady] = useState(false);
  const shown = useRef('');
  const reporter = useRef(onDrawing);
  reporter.current = onDrawing;
  // The faces are read when the player starts; a new list means a new player.
  const faceKey = faces.map((face) => `${face.family}=${face.url}`).join('|');

  useEffect(() => {
    const host = box.current;
    if (!host || !canDrawExactCaptions()) {
      reporter.current?.(false);
      return undefined;
    }
    let live = true;
    const canvas = document.createElement('canvas');
    canvas.className = 'edit-caption-canvas';
    host.appendChild(canvas);
    void (async () => {
      try {
        const { default: JASSUB } = await import('jassub');
        if (!live) return;
        const availableFonts: Record<string, string> = {};
        for (const face of faces) availableFonts[face.family.toLowerCase()] = face.url;
        const current = faces.find((face) => face.family === family);
        const player = new JASSUB({
          canvas,
          subContent: ass || EMPTY_SCRIPT,
          fonts: current ? [current.url] : [],
          availableFonts,
          defaultFont: (current ?? faces[0])?.family.toLowerCase() ?? 'inter',
          // Never ask the system or the network for a face: the render sees
          // only the pinned ones, and so does the preview.
          queryFonts: false,
        });
        instance.current = player;
        await player.ready;
        if (!live) return;
        shown.current = ass;
        setReady(true);
        reporter.current?.(true);
      } catch {
        if (live) reporter.current?.(false);
      }
    })();
    return () => {
      live = false;
      setReady(false);
      const player = instance.current;
      instance.current = null;
      if (player) void player.destroy();
      else canvas.remove();
    };
    // The script, the family and the time are followed below without
    // restarting the player; only a different set of faces restarts it.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [faceKey]);

  // A new script — an edit, or a look being tried — replaces the track.
  useEffect(() => {
    const player = instance.current;
    if (!ready || !player || shown.current === ass) return;
    shown.current = ass;
    void (async () => {
      await player.renderer.setTrack(ass || EMPTY_SCRIPT);
      await player.manualRender(
        {
          mediaTime: seconds,
          expectedDisplayTime: performance.now(),
          width: frameWidth,
          height: frameHeight,
        },
        true,
      );
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [ass, ready]);

  // A face chosen after the player started is loaded before it is needed.
  useEffect(() => {
    const player = instance.current;
    const face = faces.find((candidate) => candidate.family === family);
    if (!ready || !player || !face) return;
    void player.renderer.addFonts([face.url]);
  }, [family, faces, ready]);

  useEffect(() => {
    const player = instance.current;
    if (!ready || !player) return;
    void player.manualRender({
      mediaTime: seconds,
      expectedDisplayTime: performance.now(),
      width: frameWidth,
      height: frameHeight,
    });
  }, [seconds, frameWidth, frameHeight, ready]);

  return <div ref={box} className="edit-caption-layer" aria-hidden="true" />;
}

/** A script with no events, for a clip with no captions. */
const EMPTY_SCRIPT = `[Script Info]
ScriptType: v4.00+
PlayResX: 1080
PlayResY: 1920

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
`;
