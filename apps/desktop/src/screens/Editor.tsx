/**
 * Render the preview plan and its picture, crop, caption, and audio lanes.
 * All lanes share the plan's clock. Crop and caption decisions come from the
 * rendering code; the player maps media time to the corresponding program frame.
 */
import {
  ArrowLeft,
  Check,
  Film,
  StepBack,
  StepForward,
  Info,
  Maximize2,
  Minimize2,
  Pause,
  Play,
  Redo2,
  Undo2,
  Upload,
} from 'lucide-react';
import type {
  CSSProperties,
  PointerEvent as ReactPointerEvent,
  ReactNode,
  WheelEvent as ReactWheelEvent,
} from 'react';
import type { EditIr } from '@clipmill/contracts';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '../components/ui/empty.js';
import { EditorTranscript, type WordRange } from '../editor/EditorTranscript.js';
import { EditorProperties, type EditorSelection } from '../editor/EditorProperties.js';
import { StudioTimeline } from '../editor/StudioTimeline.js';
import { programWords, cutWords, rippleRange } from '../editor/transcript.js';
import type { Transcript } from '../results/transcript.js';
import type { Filmstrip, Peaks } from '../results/loader.js';
import { CompositionCanvas } from '../editor/CompositionCanvas.js';
import { useDraftAudio } from '../editor/useDraftAudio.js';
import '../editor/workspace.css';
import '../editor/studio.css';
import {
  batch,
  removeCropKeyframe,
  removeGainPoint,
  setCropKeyframe,
  setLayout,
  setCueRegion,
  splitSegment,
  ticksAt,
} from '../editor/commands.js';
import type { EditCommandJson } from '../daemon/client.js';
import type { PreviewPlan } from '../daemon/client.js';
import {
  cropAt,
  cueAt,
  cueLines,
  gainAt,
  highlightedWord,
  proxySecondsAt,
  resolvePlaybackFrame,
  segmentAt,
  sourceTicksAt,
  sourceOf,
  timecode,
} from '../editor/player.js';

const DEFAULT_KEYS = {
  play: 'space',
  reverse: 'j',
  pause: 'k',
  forward: 'l',
  markIn: 'i',
  markOut: 'o',
  split: 'mod+b',
  delete: 'backspace',
  marker: 'm',
  snap: 's',
  blade: 'b',
  select: 'v',
  find: 'mod+f',
  sheet: '?',
  zoomIn: 'mod+=',
  zoomOut: 'mod+-',
} as const;
type KeyAction = keyof typeof DEFAULT_KEYS;
type Keymap = Record<KeyAction, string>;
const KEY_LABELS: Record<KeyAction, string> = {
  play: 'Play / pause',
  reverse: 'Reverse',
  pause: 'Pause',
  forward: 'Forward',
  markIn: 'Mark in',
  markOut: 'Mark out',
  split: 'Split section',
  delete: 'Delete selection',
  marker: 'Add marker',
  snap: 'Toggle snapping',
  blade: 'Blade tool',
  select: 'Select tool',
  find: 'Find words',
  sheet: 'Shortcut sheet',
  zoomIn: 'Zoom in',
  zoomOut: 'Zoom out',
};
function chordOf(event: KeyboardEvent): string {
  const key = event.key === ' ' ? 'space' : event.key.toLowerCase();
  return [
    event.metaKey || event.ctrlKey ? 'mod' : '',
    event.altKey ? 'alt' : '',
    event.shiftKey && /^[a-z0-9]$/.test(key) ? 'shift' : '',
    key,
  ]
    .filter(Boolean)
    .join('+');
}

export interface EditorProps {
  /** Null until a clip has been approved and a document exists. */
  readonly plan: PreviewPlan | null;
  readonly document?: EditIr | null;
  readonly transcript?: Transcript | null;
  readonly filmstrip?: Filmstrip | null;
  readonly peaks?: Peaks | null;
  readonly filmstripUrl?: (file: string) => string;
  /**
   * The proxy for each source the plan names, by fingerprint, as a URL the
   * media protocol serves. A source with no entry has nothing to play.
   */
  readonly proxyUrls: ReadonlyMap<string, string>;
  readonly docId: string | null;
  /** What the clip is called — the project and the clip — when the route knew. */
  readonly labels: { readonly project?: string; readonly clip?: string } | null;
  readonly loading: boolean;
  readonly problem: string | null;
  readonly busy: boolean;
  readonly canUndo: boolean;
  readonly canRedo: boolean;
  readonly resolving: boolean;
  /** Why the solver cannot be asked, or `null` when it can. */
  readonly resolveRefusal: string | null;
  /**
   * What to show instead of a clip when none is open: the list of edits there
   * are. Null when a clip is named, so nothing is fetched for a list that
   * would not be shown.
   */
  readonly picker: ReactNode;
  readonly onOpenResults: () => void;
  /** Take this clip to the export screen. Null when no clip is open. */
  readonly onExport: (() => void) | null;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onUndo: () => void;
  readonly onRedo: () => void;
  readonly onResolve: (frame: number) => void;
}

export function Editor({
  plan,
  document = null,
  transcript = null,
  filmstrip = null,
  peaks = null,
  filmstripUrl = () => '',
  proxyUrls,
  docId,
  labels,
  loading,
  problem,
  busy,
  canUndo,
  canRedo,
  resolving,
  resolveRefusal,
  picker,
  onOpenResults,
  onExport,
  onApply,
  onUndo,
  onRedo,
  onResolve,
}: EditorProps) {
  const video = useRef<HTMLVideoElement>(null);
  const [playhead, setFrame] = useState(0);
  const clockFrame = useRef(0);
  const resumeAfterLoad = useRef(false);
  const waitingForReference = useRef(false);
  const resumeAfterReference = useRef(false);
  const playbackFailed = useRef(false);
  const activeDocument = useRef(docId);
  activeDocument.current = docId;
  const [preparingPreview, setPreparingPreview] = useState(false);
  const updateFrame = useCallback((next: number) => {
    clockFrame.current = next;
    setFrame(next);
  }, []);
  const [playing, setPlaying] = useState(false);
  const [selection, setSelection] = useState<EditorSelection>({ kind: 'clip' });
  const [zoom, setZoom] = useState(1);
  const [snapping, setSnapping] = useState(true);
  const [tool, setTool] = useState<'select' | 'blade'>('select');
  const [markers, setMarkers] = useState<readonly number[]>([]);
  const [inFrame, setInFrame] = useState<number | null>(null);
  const [outFrame, setOutFrame] = useState<number | null>(null);
  const [safePlatform, setSafePlatform] = useState<'off' | 'tiktok' | 'reels' | 'shorts'>('off');
  const [grid, setGrid] = useState(false);
  const [before, setBefore] = useState(false);
  const [loopRange, setLoopRange] = useState(false);
  const [speed, setSpeed] = useState(1);
  const reverseTimer = useRef<ReturnType<typeof setInterval> | null>(null);
  const [shortcutOpen, setShortcutOpen] = useState(false);
  const [recordingKey, setRecordingKey] = useState<KeyAction | null>(null);
  const [keymap, setKeymap] = useState<Keymap>(() => {
    try {
      return {
        ...DEFAULT_KEYS,
        ...JSON.parse(localStorage.getItem('clipmill.editor.keys') ?? '{}'),
      } as Keymap;
    } catch {
      return { ...DEFAULT_KEYS };
    }
  });
  const [findSignal, setFindSignal] = useState(0);
  const [panelWidths, setPanelWidths] = useState({ left: 300, right: 315 });
  const [focusedPreview, setFocusedPreview] = useState(false);
  const [playbackProblem, setPlaybackProblem] = useState<string | null>(null);
  // What is drawn is always a frame the program has. A trim can shorten the
  // program under a playhead that did not move, and between that render and
  // the effect below that moves it back, a frame past the end would find no
  // segment — and no segment would take the picture down with it.
  const frame = plan ? Math.max(0, Math.min(playhead, plan.frameCount - 1)) : playhead;
  const draftAudio = useDraftAudio(video, plan ? gainAt(plan, frame) : 0);

  useEffect(() => {
    if (!recordingKey) return;
    const record = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopImmediatePropagation();
      if (event.key === 'Escape') {
        setRecordingKey(null);
        return;
      }
      if (['Meta', 'Control', 'Alt', 'Shift'].includes(event.key)) return;
      const chord = chordOf(event);
      setKeymap((current) => {
        const next = { ...current };
        for (const action of Object.keys(next) as KeyAction[]) {
          if (action !== recordingKey && next[action] === chord)
            next[action] = current[recordingKey];
        }
        next[recordingKey] = chord;
        try {
          localStorage.setItem('clipmill.editor.keys', JSON.stringify(next));
        } catch {
          /* The current session still has the mapping. */
        }
        return next;
      });
      setRecordingKey(null);
    };
    window.addEventListener('keydown', record, true);
    return () => window.removeEventListener('keydown', record, true);
  }, [recordingKey]);

  // A different document is a different program; a playhead left where the
  // last one was would seek the new proxy to a frame it may not have.
  useEffect(() => {
    updateFrame(0);
    resumeAfterLoad.current = false;
    waitingForReference.current = false;
    resumeAfterReference.current = false;
    playbackFailed.current = false;
    setPreparingPreview(false);
    setPlaying(false);
    setSelection({ kind: 'clip' });
    setInFrame(null);
    setOutFrame(null);
    setZoom(1);
    setTool('select');
    setBefore(false);
    if (reverseTimer.current) clearInterval(reverseTimer.current);
    reverseTimer.current = null;
    try {
      setMarkers(
        JSON.parse(localStorage.getItem(`clipmill.editor.markers.${docId}`) ?? '[]') as number[],
      );
      setPanelWidths(
        JSON.parse(
          localStorage.getItem('clipmill.editor.panels') ?? '{"left":300,"right":315}',
        ) as { left: number; right: number },
      );
    } catch {
      setMarkers([]);
    }
    setFocusedPreview(false);
    setPlaybackProblem(null);
  }, [docId, updateFrame]);

  useEffect(() => {
    if (video.current) video.current.playbackRate = speed;
  }, [speed]);

  /**
   * Put the playhead on a program frame, and the media element where that
   * frame is in the proxy. Every seek goes through here — the transport, the
   * scrubber, the arrow keys — so there is one place the two clocks meet.
   */
  const seek = useCallback(
    (target: number) => {
      if (!plan) {
        return;
      }
      const next = Math.max(0, Math.min(plan.frameCount - 1, target));
      updateFrame(next);
      const element = video.current;
      const seconds = proxySecondsAt(plan, next);
      if (element && seconds !== null) {
        const source = segmentAt(plan, next);
        const url = source ? proxyUrls.get(source.sourceFingerprint) : null;
        if (url !== element.getAttribute('src')) {
          // React will change src; seeking the previous recording would show
          // unrelated footage. Resume only after the new proxy is positioned.
          resumeAfterLoad.current = !element.paused;
          return;
        }
        // Set whether or not the media has loaded: before metadata a browser
        // keeps it as the position to start from, which is what is wanted.
        element.currentTime = seconds;
      }
    },
    [plan, proxyUrls, updateFrame],
  );

  // A new plan is a new mapping. A trim moved the segment's window, so the
  // frame the playhead is on now shows different footage and may not exist
  // at all; the media element is told where that frame is now, and a playhead
  // past the new end is brought back onto the program. Keyed on the plan
  // rather than on the document because a trim changes neither the document
  // id nor the proxy.
  useEffect(() => {
    if (!plan) {
      return;
    }
    seek(Math.min(clockFrame.current, plan.frameCount - 1));
    // eslint-disable-next-line react-hooks/exhaustive-deps -- the plan is the signal
  }, [plan, docId]);

  const step = useCallback((by: number) => seek(frame + by), [frame, seek]);

  const stopWithProblem = useCallback(
    (message: string) => {
      if (activeDocument.current !== docId || playbackFailed.current) return;
      playbackFailed.current = true;
      waitingForReference.current = false;
      resumeAfterReference.current = false;
      resumeAfterLoad.current = false;
      setPreparingPreview(false);
      const element = video.current;
      if (element && !element.paused) element.pause();
      setPlaying(false);
      setPlaybackProblem(message);
    },
    [docId],
  );

  const onBuffering = useCallback(
    (waiting: boolean) => {
      if (activeDocument.current !== docId || waitingForReference.current === waiting) return;
      const element = video.current;
      if (waiting) {
        if (!element || playbackFailed.current) return;
        waitingForReference.current = true;
        resumeAfterReference.current = !element.paused || resumeAfterLoad.current;
        resumeAfterLoad.current = false;
        setPreparingPreview(true);
        element.pause();
        seek(clockFrame.current);
        setPlaying(resumeAfterReference.current);
        return;
      }
      const resume = resumeAfterReference.current;
      waitingForReference.current = false;
      resumeAfterReference.current = false;
      setPreparingPreview(false);
      setPlaying(false);
      if (resume && element && !playbackFailed.current) {
        draftAudio.connect();
        void element.play().catch(() => {
          stopWithProblem('Playback could not continue. Try playing the clip again.');
        });
      }
    },
    [docId, seek, draftAudio.connect, stopWithProblem],
  );

  const onMediaPlaying = useCallback(
    (next: boolean) => {
      if (activeDocument.current !== docId) return;
      setPlaying(waitingForReference.current ? resumeAfterReference.current : next);
    },
    [docId],
  );

  const togglePlayback = useCallback(() => {
    const element = video.current;
    if (!element || !plan) return;
    setPlaybackProblem(null);
    playbackFailed.current = false;
    if (waitingForReference.current) {
      resumeAfterReference.current = !resumeAfterReference.current;
      setPlaying(resumeAfterReference.current);
      return;
    }
    if (!element.paused) {
      element.pause();
    } else {
      if (frame >= plan.frameCount - 1 || element.ended) seek(0);
      draftAudio.connect();
      void element.play().catch(() => {
        stopWithProblem('Playback could not start. Try playing the clip again.');
      });
    }
  }, [frame, plan, seek, draftAudio.connect, stopWithProblem]);

  const addMarker = useCallback(
    (at: number) => {
      setMarkers((current) => {
        const next = [...new Set([...current, at])].toSorted((a, b) => a - b);
        try {
          localStorage.setItem(`clipmill.editor.markers.${docId}`, JSON.stringify(next));
        } catch {
          /* Markers remain available this session. */
        }
        return next;
      });
    },
    [docId],
  );

  const splitHere = useCallback(() => {
    if (!plan || busy) return;
    const part = segmentAt(plan, frame);
    const ticks = sourceTicksAt(plan, frame);
    if (!part || ticks === null || ticks <= part.inTicks || ticks >= part.outTicks) return;
    const existing = new Set(document?.video.segments?.map((item) => item.segment_id));
    let id = `${part.segmentId}_cut_${ticks}`;
    let suffix = 2;
    while (existing.has(id)) id = `${part.segmentId}_cut_${ticks}_${suffix++}`;
    onApply(splitSegment(part.segmentId, ticks, id));
    setSelection({ kind: 'section', segmentId: part.segmentId });
  }, [plan, busy, frame, document, onApply]);

  const deleteSelection = useCallback(() => {
    if (!plan || busy) return;
    let command: EditCommandJson | null = null;
    if (selection.kind === 'words') {
      const words = programWords(plan, transcript);
      command = cutWords(
        plan,
        words,
        Array.from(
          { length: selection.range.last - selection.range.first + 1 },
          (_, index) => selection.range.first + index,
        ),
      );
    } else if (selection.kind === 'keyframe') {
      command = removeCropKeyframe(selection.tTicks, selection.segmentId, selection.secondary);
    } else if (selection.kind === 'gain') {
      command = removeGainPoint(selection.tTicks);
    } else if (inFrame !== null && outFrame !== null) {
      command = rippleRange(
        plan,
        ticksAt(plan, Math.min(inFrame, outFrame)),
        ticksAt(plan, Math.max(inFrame, outFrame)),
      );
    }
    if (command) {
      onApply(command);
      setSelection({ kind: 'clip' });
    }
  }, [plan, busy, selection, transcript, inFrame, outFrame, onApply]);

  const resizePanel = (side: 'left' | 'right', event: ReactPointerEvent) => {
    event.preventDefault();
    const initial = event.clientX;
    const starting = panelWidths[side];
    const move = (next: PointerEvent) =>
      setPanelWidths((current) => ({
        ...current,
        [side]: Math.max(
          220,
          Math.min(480, starting + (next.clientX - initial) * (side === 'left' ? 1 : -1)),
        ),
      }));
    const done = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', done);
      setPanelWidths((current) => {
        try {
          localStorage.setItem('clipmill.editor.panels', JSON.stringify(current));
        } catch {
          /* This session still keeps the size. */
        }
        return current;
      });
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', done);
  };

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !event.defaultPrevented) {
        setFocusedPreview(false);
        setShortcutOpen(false);
        setSelection({ kind: 'clip' });
      }
      const target = event.target;
      // Text, range inputs and Radix controls own their keyboard interactions.
      if (
        target instanceof Element &&
        target.closest(
          'input, textarea, select, button, [role="tab"], [role="slider"], [contenteditable="true"]',
        )
      )
        return;
      if (event.defaultPrevented || !plan) return;
      const key = event.key.toLowerCase();
      const chord = chordOf(event);
      const is = (action: KeyAction) => chord === keymap[action];
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'z') {
        event.preventDefault();
        if (!busy && (event.shiftKey ? canRedo : canUndo)) (event.shiftKey ? onRedo : onUndo)();
      } else if (is('split')) {
        event.preventDefault();
        splitHere();
      } else if (is('find')) {
        event.preventDefault();
        setFindSignal((value) => value + 1);
      } else if ((event.metaKey || event.ctrlKey) && key === 'e' && onExport) {
        event.preventDefault();
        onExport();
      } else if (is('zoomIn') || chord === 'mod++') {
        event.preventDefault();
        setZoom((value) => Math.min(8, value * 1.5));
      } else if (is('zoomOut')) {
        event.preventDefault();
        setZoom((value) => Math.max(1, value / 1.5));
      } else if (is('play')) {
        event.preventDefault();
        if (reverseTimer.current) {
          clearInterval(reverseTimer.current);
          reverseTimer.current = null;
        }
        togglePlayback();
      } else if (is('pause')) {
        event.preventDefault();
        if (reverseTimer.current) {
          clearInterval(reverseTimer.current);
          reverseTimer.current = null;
        }
        video.current?.pause();
        setSpeed(1);
      } else if (is('reverse')) {
        event.preventDefault();
        const element = video.current;
        if (element && !element.paused) element.pause();
        if (reverseTimer.current) clearInterval(reverseTimer.current);
        const rate = speed >= 4 ? 1 : speed * 2;
        setSpeed(rate);
        reverseTimer.current = setInterval(
          () =>
            seek(
              clockFrame.current -
                Math.max(1, Math.round((rate * plan.rateNum) / plan.rateDen / 12)),
            ),
          80,
        );
      } else if (is('forward')) {
        event.preventDefault();
        if (reverseTimer.current) {
          clearInterval(reverseTimer.current);
          reverseTimer.current = null;
        }
        setSpeed((value) => (value >= 4 ? 1 : value * 2));
        if (video.current?.paused) togglePlayback();
      } else if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
        event.preventDefault();
        step((event.key === 'ArrowLeft' ? -1 : 1) * (event.shiftKey ? 10 : 1));
      } else if (key === 'home' || key === 'end') {
        event.preventDefault();
        seek(key === 'home' ? 0 : plan.frameCount - 1);
      } else if (is('markIn')) {
        event.preventDefault();
        setInFrame(frame);
      } else if (is('markOut')) {
        event.preventDefault();
        setOutFrame(frame);
      } else if (is('delete') || (keymap.delete === 'backspace' && chord === 'delete')) {
        event.preventDefault();
        deleteSelection();
      } else if (is('select')) {
        event.preventDefault();
        setTool('select');
      } else if (is('blade')) {
        event.preventDefault();
        setTool('blade');
      } else if (is('snap')) {
        event.preventDefault();
        setSnapping((value) => !value);
      } else if (is('marker')) {
        event.preventDefault();
        addMarker(frame);
      } else if (key === 'z' && event.shiftKey) {
        event.preventDefault();
        setZoom(1);
      } else if (is('sheet')) {
        event.preventDefault();
        setShortcutOpen((value) => !value);
      } else if (key === '[' || key === ']') {
        event.preventDefault();
        setInFrame(key === '[' ? frame : inFrame);
        setOutFrame(key === ']' ? frame : outFrame);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [
    step,
    plan,
    togglePlayback,
    busy,
    canRedo,
    canUndo,
    onRedo,
    onUndo,
    onExport,
    splitHere,
    deleteSelection,
    speed,
    seek,
    frame,
    addMarker,
    inFrame,
    outFrame,
    keymap,
  ]);

  useEffect(
    () => () => {
      if (reverseTimer.current) clearInterval(reverseTimer.current);
    },
    [],
  );

  const cue = useMemo(() => (plan ? cueAt(plan, frame) : null), [plan, frame]);
  const highlighted = useMemo(
    () => (plan && cue ? highlightedWord(plan, cue, frame) : -1),
    [plan, cue, frame],
  );
  const segment = useMemo(() => (plan ? segmentAt(plan, frame) : null), [plan, frame]);

  // This callback returns the frame synchronously: the canvas must use its
  // crop for these decoded pixels, without waiting for React to move the UI.
  const onProxyTime = useCallback(
    (seconds: number, atMediaEnd = false): number | null => {
      const element = video.current;
      if (!plan || !element || element.seeking || playbackFailed.current) return null;
      if (waitingForReference.current) return clockFrame.current;
      const active = segmentAt(plan, clockFrame.current);
      if (!active || proxyUrls.get(active.sourceFingerprint) !== element.getAttribute('src'))
        return null;
      // A paused seek requests a program frame even when the lower-rate proxy
      // decodes an earlier frame. Do not move the scrubber back to that frame.
      if (element.paused && !atMediaEnd) {
        // Pausing in the fractional boundary gap must not repaint the newly
        // decoded incoming bitmap under the held outgoing frame's crop.
        return resolvePlaybackFrame(plan, clockFrame.current, seconds).hold
          ? null
          : clockFrame.current;
      }
      const next = resolvePlaybackFrame(plan, clockFrame.current, seconds);
      if (
        loopRange &&
        inFrame !== null &&
        outFrame !== null &&
        next.frame >= Math.max(inFrame, outFrame)
      ) {
        seek(Math.min(inFrame, outFrame));
        return null;
      }
      if (next.seek || next.ended) {
        if (next.ended) {
          element.pause();
          setPlaying(false);
        }
        seek(next.frame);
        if (atMediaEnd && next.seek) {
          // Native EOF is paused already, but an edit can continue in another
          // recording (or repeat this one). Preserve that playback intent.
          const following = segmentAt(plan, next.frame);
          if (
            following &&
            proxyUrls.get(following.sourceFingerprint) !== element.getAttribute('src')
          ) {
            resumeAfterLoad.current = true;
          } else {
            void element.play().catch(() => {
              stopWithProblem('Playback could not continue. Try playing the clip again.');
            });
          }
        }
        return null;
      }
      if (atMediaEnd) {
        stopWithProblem('The preview ended before the clip. Reopen the clip to reload its proxy.');
        return null;
      }
      updateFrame(next.frame);
      return next.hold ? null : next.frame;
    },
    [plan, proxyUrls, seek, updateFrame, stopWithProblem, loopRange, inFrame, outFrame],
  );

  const onMetadata = useCallback(() => {
    const element = video.current;
    if (!element) return;
    seek(clockFrame.current);
    if (resumeAfterLoad.current) {
      resumeAfterLoad.current = false;
      if (waitingForReference.current) {
        resumeAfterReference.current = true;
        setPlaying(true);
        return;
      }
      if (playbackFailed.current) return;
      void element.play().catch(() => {
        stopWithProblem('Playback could not continue. Try playing the clip again.');
      });
    }
  }, [seek, stopWithProblem]);

  if (loading) {
    return (
      <div className="editor-loading" role="status">
        Opening your clip…
      </div>
    );
  }

  if (!plan || !docId) {
    return (
      <div className="p-8">
        <Empty>
          <EmptyHeader>
            <EmptyTitle>
              {problem && labels
                ? 'This clip could not be opened'
                : 'No clip is open in the editor'}
            </EmptyTitle>
            <EmptyDescription>
              {problem ??
                'Choose a saved edit below, or approve a clip in Results to start editing.'}
            </EmptyDescription>
          </EmptyHeader>
          {picker}
          <Button variant="outline" onClick={onOpenResults}>
            Go to Results
          </Button>
        </Empty>
      </div>
    );
  }

  const proxyUrl = segment ? (proxyUrls.get(segment.sourceFingerprint) ?? null) : null;

  return (
    <div className="editor-workspace" data-preview-focused={focusedPreview}>
      <header className="workspace-heading editor-heading">
        <div className="editor-identity">
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={onOpenResults}
            aria-label="Back to results"
            title="Back to results"
          >
            <ArrowLeft />
          </Button>
          <div className="min-w-0">
            <h1 className="truncate text-[15px] font-medium" data-testid="clip-name">
              {labels?.clip ?? 'Clip editor'}
            </h1>
            <p className="workspace-subtitle truncate">
              {labels?.project ?? 'Your edit'} <span aria-hidden>·</span>{' '}
              {timecode(plan, plan.frameCount)}
            </p>
          </div>
        </div>
        <div className="editor-header-actions">
          <span role="status" className="editor-save-state">
            {!busy && !problem && <Check className="size-3.5" />}{' '}
            {busy ? 'Saving…' : problem ? 'Check edit status' : 'Saved'}
          </span>
          <div className="editor-history">
            <Button
              size="icon-sm"
              variant="ghost"
              disabled={!canUndo || busy}
              onClick={onUndo}
              aria-label="Undo"
              title="Undo (⌘/Ctrl Z)"
            >
              <Undo2 />
            </Button>
            <Button
              size="icon-sm"
              variant="ghost"
              disabled={!canRedo || busy}
              onClick={onRedo}
              aria-label="Redo"
              title="Redo (⌘/Ctrl Shift Z)"
            >
              <Redo2 />
            </Button>
          </div>
          <Button
            size="icon-sm"
            variant="ghost"
            onClick={() => setFocusedPreview((focused) => !focused)}
            aria-label={focusedPreview ? 'Restore editing panels' : 'Focus preview'}
            aria-pressed={focusedPreview}
            title={focusedPreview ? 'Restore editing panels (Esc)' : 'Focus preview'}
          >
            {focusedPreview ? <Minimize2 /> : <Maximize2 />}
          </Button>
          {onExport && (
            <Button size="sm" onClick={onExport} disabled={busy} aria-label="Export this clip">
              <Upload className="size-4" />
              Export clip
            </Button>
          )}
        </div>
      </header>
      {(problem || playbackProblem || draftAudio.problem || segment?.framingWarning) && (
        <p role="alert" className="editor-alert">
          {problem || playbackProblem || draftAudio.problem || segment?.framingWarning}
        </p>
      )}
      <div
        className="editor-body"
        style={
          {
            '--editor-left': `${panelWidths.left}px`,
            '--editor-right': `${panelWidths.right}px`,
          } as CSSProperties
        }
      >
        <EditorTranscript
          plan={plan}
          transcript={transcript}
          frame={frame}
          busy={busy}
          selected={selection.kind === 'words' ? selection.range : null}
          findSignal={findSignal}
          onSelect={(range: WordRange) => setSelection({ kind: 'words', range })}
          onSeek={seek}
          onApply={onApply}
        />
        <div
          className="editor-panel-resizer"
          role="separator"
          aria-label="Resize transcript panel"
          aria-orientation="vertical"
          onPointerDown={(event) => resizePanel('left', event)}
        />
        <section className="editor-viewer" aria-label="Clip preview">
          <div className="editor-viewer-heading">
            <span className="editor-monitor-label">
              <Film aria-hidden="true" /> Program
            </span>
            <div className="editor-monitor-tools">
              <label>
                Safe zones{' '}
                <select
                  aria-label="Platform safe zones"
                  value={safePlatform}
                  onChange={(event) => setSafePlatform(event.target.value as typeof safePlatform)}
                >
                  <option value="off">Off</option>
                  <option value="tiktok">TikTok</option>
                  <option value="reels">Reels</option>
                  <option value="shorts">Shorts</option>
                </select>
              </label>
              <button type="button" aria-pressed={grid} onClick={() => setGrid((value) => !value)}>
                Grid
              </button>
              <button
                type="button"
                aria-pressed={before}
                onClick={() => setBefore((value) => !value)}
              >
                Before / after
              </button>
            </div>
            <span className="editor-monitor-metadata">
              {segment && plan.segments.length > 1 && (
                <span className="editor-monitor-position">
                  Section {String(plan.segments.indexOf(segment) + 1).padStart(2, '0')}
                  <span aria-hidden="true"> / </span>
                  <span className="sr-only"> of </span>
                  {String(plan.segments.length).padStart(2, '0')}
                </span>
              )}
              <span>
                {plan.width} × {plan.height}
              </span>
            </span>
          </div>
          <div className="editor-stage-wrap">
            <Stage
              key={docId}
              proxyUrl={proxyUrl}
              proxyUrls={proxyUrls}
              videoRef={video}
              frame={frame}
              plan={plan}
              lines={cue ? cueLines(cue) : []}
              cue={cue}
              highlighted={highlighted}
              startSeconds={proxySecondsAt(plan, frame)}
              onProxyTime={onProxyTime}
              onMetadata={onMetadata}
              onPlaying={onMediaPlaying}
              preparingPreview={preparingPreview}
              onBuffering={onBuffering}
              safePlatform={safePlatform}
              grid={grid}
              before={before}
              busy={busy}
              selectedCue={selection.kind === 'cue' ? selection.cueId : null}
              onSelectCue={(cueId) => setSelection({ kind: 'cue', cueId })}
              onApply={onApply}
              onError={() => {
                stopWithProblem('The preview could not be loaded. Reopen the clip to try again.');
              }}
            />
          </div>
          <Transport
            plan={plan}
            frame={frame}
            playing={playing}
            disabled={!proxyUrl}
            onStep={step}
            onToggle={togglePlayback}
          />
          <div className="editor-range-tools">
            <button type="button" onClick={() => setInFrame(frame)}>
              In <kbd>I</kbd>
            </button>
            <span>{inFrame === null ? '—' : timecode(plan, inFrame)}</span>
            <button type="button" onClick={() => setOutFrame(frame)}>
              Out <kbd>O</kbd>
            </button>
            <span>{outFrame === null ? '—' : timecode(plan, outFrame)}</span>
            <button
              type="button"
              aria-pressed={loopRange}
              onClick={() => setLoopRange((value) => !value)}
              disabled={inFrame === null || outFrame === null}
            >
              Loop range
            </button>
            <button
              type="button"
              disabled={busy || inFrame === null || outFrame === null}
              onClick={deleteSelection}
            >
              Delete range
            </button>
          </div>
        </section>
        <div
          className="editor-panel-resizer"
          role="separator"
          aria-label="Resize properties panel"
          aria-orientation="vertical"
          onPointerDown={(event) => resizePanel('right', event)}
        />
        <EditorProperties
          plan={plan}
          document={document}
          transcript={transcript}
          frame={frame}
          selection={selection}
          busy={busy}
          resolving={resolving}
          resolveRefusal={resolveRefusal}
          onApply={onApply}
          onResolve={() => onResolve(frame)}
        />
      </div>

      <StudioTimeline
        plan={plan}
        document={document}
        transcript={transcript}
        filmstrip={filmstrip}
        peaks={peaks}
        frame={frame}
        docId={docId}
        selection={selection}
        busy={busy}
        zoom={zoom}
        onZoom={setZoom}
        snap={snapping}
        onSnap={setSnapping}
        tool={tool}
        onTool={setTool}
        markers={markers}
        onMarker={addMarker}
        filmstripUrl={filmstripUrl}
        onSeek={seek}
        onSelect={setSelection}
        onApply={onApply}
      />
      <details className="editor-diagnostics">
        <summary>
          <Info aria-hidden="true" /> Diagnostics
        </summary>
        <p>
          Revision {plan.revision} · {(plan.rateNum / plan.rateDen).toFixed(2)} fps · {plan.width}×
          {plan.height} · preview from render plan
        </p>
      </details>
      {shortcutOpen && (
        <div className="editor-shortcut-sheet" role="dialog" aria-label="Editor shortcuts">
          <div>
            <header>
              <h2>Keyboard shortcuts</h2>
              <button
                type="button"
                onClick={() => {
                  setShortcutOpen(false);
                  setRecordingKey(null);
                }}
              >
                Close
              </button>
            </header>
            <p>Choose a shortcut, then press the keys you want to use.</p>
            <dl>
              {(Object.keys(DEFAULT_KEYS) as KeyAction[]).map((action) => (
                <div key={action}>
                  <dt>{KEY_LABELS[action]}</dt>
                  <dd>
                    <button
                      type="button"
                      aria-label={`Change ${KEY_LABELS[action]} shortcut`}
                      data-recording={recordingKey === action}
                      onClick={() => setRecordingKey(action)}
                    >
                      {recordingKey === action
                        ? 'Press keys…'
                        : keymap[action]
                            .replace('mod', '⌘/Ctrl')
                            .replace('space', 'Space')
                            .toUpperCase()}
                    </button>
                  </dd>
                </div>
              ))}
            </dl>
            <button
              type="button"
              className="editor-reset-shortcuts"
              onClick={() => {
                const next = { ...DEFAULT_KEYS };
                setKeymap(next);
                setRecordingKey(null);
                try {
                  localStorage.setItem('clipmill.editor.keys', JSON.stringify(next));
                } catch {
                  /* This session still resets. */
                }
              }}
            >
              Reset defaults
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

/** The 9:16 stage: the proxy, cropped by the plan, with the plan's captions. */
function Stage({
  proxyUrl,
  proxyUrls,
  videoRef,
  frame,
  plan,
  lines,
  cue,
  highlighted,
  startSeconds,
  onProxyTime,
  onMetadata,
  onPlaying,
  onError,
  preparingPreview,
  onBuffering,
  safePlatform,
  grid,
  before,
  busy,
  selectedCue,
  onSelectCue,
  onApply,
}: {
  readonly proxyUrl: string | null;
  readonly proxyUrls: ReadonlyMap<string, string>;
  readonly videoRef: React.RefObject<HTMLVideoElement | null>;
  readonly frame: number;
  readonly plan: PreviewPlan;
  readonly lines: readonly string[];
  readonly cue: ReturnType<typeof cueAt>;
  readonly highlighted: number;
  /** Where in the proxy the current frame is, to land on when the media loads. */
  readonly startSeconds: number | null;
  readonly onProxyTime: (seconds: number, atMediaEnd?: boolean) => number | null;
  readonly onMetadata: () => void;
  readonly onPlaying: (playing: boolean) => void;
  readonly onError: () => void;
  readonly preparingPreview: boolean;
  readonly onBuffering: (waiting: boolean) => void;
  readonly safePlatform: 'off' | 'tiktok' | 'reels' | 'shorts';
  readonly grid: boolean;
  readonly before: boolean;
  readonly busy: boolean;
  readonly selectedCue: string | null;
  readonly onSelectCue: (cueId: string) => void;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const style = plan.captionStyle;
  const part = segmentAt(plan, frame);
  const source = part ? sourceOf(plan, part) : null;
  const crop = cropAt(plan, frame);
  const localTicks = part ? ticksAt(plan, frame) - part.programStartTicks : 0;
  const comparisonPlan = useMemo(
    () =>
      before
        ? ({
            ...plan,
            crops: plan.crops.map(() => null),
            secondaryCrops: plan.secondaryCrops?.map(() => null),
          } as PreviewPlan)
        : plan,
    [before, plan],
  );
  const moveFrame = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (busy || before || !part || !source || !crop) return;
    const element = event.currentTarget;
    const startX = event.clientX;
    const startY = event.clientY;
    const move = (next: PointerEvent) => {
      const dx = Math.round(
        ((next.clientX - startX) / Math.max(1, element.clientWidth)) * crop.width,
      );
      const dy = Math.round(
        ((next.clientY - startY) / Math.max(1, element.clientHeight)) * crop.height,
      );
      element.style.setProperty('--drag-x', `${next.clientX - startX}px`);
      element.style.setProperty('--drag-y', `${next.clientY - startY}px`);
      element.dataset.dragging = String(dx !== 0 || dy !== 0);
    };
    const finish = (next: PointerEvent) => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', finish);
      element.style.removeProperty('--drag-x');
      element.style.removeProperty('--drag-y');
      delete element.dataset.dragging;
      const dx = Math.round(
        ((next.clientX - startX) / Math.max(1, element.clientWidth)) * crop.width,
      );
      const dy = Math.round(
        ((next.clientY - startY) / Math.max(1, element.clientHeight)) * crop.height,
      );
      if (dx || dy)
        onApply(
          setCropKeyframe(
            localTicks,
            {
              ...crop,
              x: Math.max(0, Math.min(source.displayWidth - crop.width, crop.x - dx)),
              y: Math.max(0, Math.min(source.displayHeight - crop.height, crop.y - dy)),
            },
            part.segmentId,
          ),
        );
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', finish);
  };
  const zoomFrame = (event: ReactWheelEvent<HTMLDivElement>) => {
    if (busy || before || !part || !source) return;
    event.preventDefault();
    const current = crop ?? {
      x: 0,
      y: 0,
      width: source.displayWidth,
      height: source.displayHeight,
    };
    const scale = event.deltaY < 0 ? 0.9 : 1.1;
    const height = Math.max(
      2,
      Math.min(source.displayHeight, Math.round((current.height * scale) / 2) * 2),
    );
    const width = Math.max(
      2,
      Math.min(source.displayWidth, Math.round((height * plan.width) / plan.height / 2) * 2),
    );
    if (width === current.width && height === current.height) return;
    const rect = {
      width,
      height,
      x: Math.max(
        0,
        Math.min(source.displayWidth - width, current.x + Math.round((current.width - width) / 2)),
      ),
      y: Math.max(
        0,
        Math.min(
          source.displayHeight - height,
          current.y + Math.round((current.height - height) / 2),
        ),
      ),
    };
    onApply(
      crop
        ? setCropKeyframe(localTicks, rect, part.segmentId)
        : batch([
            setLayout('speaker_fill', part.segmentId),
            setCropKeyframe(localTicks, rect, part.segmentId),
          ]),
    );
  };
  const captionGesture = (event: ReactPointerEvent<HTMLParagraphElement>) => {
    if (!cue) return;
    event.stopPropagation();
    onSelectCue(cue.cueId);
    const stage = event.currentTarget.closest<HTMLElement>('.video-stage');
    if (!stage || busy) return;
    const startY = event.clientY;
    const finish = (next: PointerEvent) => {
      window.removeEventListener('pointerup', finish);
      if (Math.abs(next.clientY - startY) < 8) return;
      const share =
        (next.clientY - stage.getBoundingClientRect().top) / Math.max(1, stage.clientHeight);
      const region = share < 0.37 ? 'upper_safe' : share > 0.65 ? 'lower_safe' : 'center';
      if (region !== cue.region) onApply(setCueRegion(cue.cueId, region, plan.presentation));
    };
    window.addEventListener('pointerup', finish);
  };
  const relative = (pixels: number) => `${(pixels / plan.width) * 100}cqw`;
  const verticalMargin = ((style?.marginVertical ?? 260) / plan.height) * 100;
  const position =
    cue?.region === 'upper_safe'
      ? { top: `${verticalMargin}%` }
      : cue?.region === 'center'
        ? { top: '50%', transform: 'translateY(-50%)' }
        : { bottom: `${verticalMargin}%` };
  let index = 0;
  return (
    <div className="video-stage" data-testid="stage" style={{ containerType: 'inline-size' }}>
      {proxyUrl ? (
        <div className="absolute inset-0 flex items-center justify-center">
          {/* eslint-disable-next-line jsx-a11y/media-has-caption -- the cues are
              drawn below from the plan rather than as a text track. */}
          <video
            ref={videoRef}
            src={proxyUrl}
            className="pointer-events-none absolute h-full w-full opacity-0"
            crossOrigin="anonymous"
            playsInline
            data-testid="proxy"
            data-start-seconds={startSeconds ?? undefined}
            onLoadedMetadata={onMetadata}
            onPlay={() => onPlaying(true)}
            onPause={() => onPlaying(false)}
            onEnded={(event) => {
              onPlaying(false);
              // The last decoded PTS can precede the last program frame.
              // EOF itself must advance/finish the edit, not restart the proxy.
              onProxyTime(event.currentTarget.currentTime, true);
            }}
            onError={onError}
          />
          <CompositionCanvas
            video={videoRef}
            mediaKey={proxyUrl}
            plan={comparisonPlan}
            frame={frame}
            onFrame={onProxyTime}
            proxyUrls={proxyUrls}
            onError={onError}
            onBuffering={onBuffering}
          />
        </div>
      ) : (
        <p className="grid h-full place-items-center p-6 text-center text-sm text-[var(--cm-viewer-ink)]">
          This recording has no proxy, so there is nothing to play.
        </p>
      )}
      {proxyUrl && (
        <div
          className="editor-frame-grab"
          aria-label="Drag picture to reframe; scroll to zoom"
          onPointerDown={moveFrame}
          onWheel={zoomFrame}
        />
      )}
      {grid && <div className="editor-monitor-grid" aria-hidden="true" />}
      {safePlatform !== 'off' && (
        <div className={`editor-safe-zone editor-safe-zone-${safePlatform}`} aria-hidden="true">
          <span>Safe for {safePlatform}</span>
        </div>
      )}
      {preparingPreview && (
        <div
          role="status"
          className="absolute inset-0 z-10 grid place-items-center bg-[var(--cm-glass)] backdrop-blur-[2px]"
        >
          <span className="rounded-md border border-[var(--cm-glass-border)] bg-[var(--cm-surface-1)] px-3 py-2 text-xs text-[var(--cm-ink-2)]">
            Preparing preview…
          </span>
        </div>
      )}
      {proxyUrl && !before && lines.length > 0 && (
        <p
          className="editor-grabbable-caption absolute text-center leading-tight"
          data-selected={selectedCue === cue?.cueId}
          onPointerDown={captionGesture}
          title="Click to edit; drag up or down to reposition"
          style={{
            ...position,
            left: `${((style?.marginHorizontal ?? 90) / plan.width) * 100}%`,
            right: `${((style?.marginHorizontal ?? 90) / plan.width) * 100}%`,
            fontFamily:
              !style || style.fontFamily === 'Inter' ? 'ClipMill Caption Inter' : style.fontFamily,
            fontSize: relative(style?.fontSize ?? 84),
            fontWeight: style?.bold === false ? 400 : 700,
            WebkitTextStroke: style?.boxed
              ? undefined
              : `${relative(style?.outlineWidth ?? 5)} ${style?.outline ?? '#000000'}`,
            paintOrder: 'stroke fill',
            textShadow: style?.boxed
              ? undefined
              : `${relative(style?.shadowDepth ?? 2)} ${relative(style?.shadowDepth ?? 2)} 0 ${style?.shadow ?? '#000000'}`,
          }}
          data-testid="caption"
        >
          {cue?.lines.map((line, lineIndex) => (
            // eslint-disable-next-line react/no-array-index-key -- lines have no
            // identity of their own; their position is what they are.
            <span key={lineIndex} className="block whitespace-nowrap">
              <span
                style={
                  style?.boxed
                    ? {
                        background: style.outline,
                        padding: `${relative(style.outlineWidth)} ${relative(style.outlineWidth * 2)}`,
                        boxDecorationBreak: 'clone',
                      }
                    : undefined
                }
              >
                {line.map((word) => {
                  const mine = index;
                  index += 1;
                  return (
                    <span
                      key={`${word.text}-${mine}`}
                      style={{
                        color:
                          !cue.karaoke || mine <= highlighted
                            ? (style?.spoken ?? '#ffd65c')
                            : (style?.unspoken ?? '#ffffff'),
                      }}
                    >
                      {word.text}{' '}
                    </span>
                  );
                })}
              </span>
            </span>
          ))}
        </p>
      )}
    </div>
  );
}

function Transport({
  plan,
  frame,
  playing,
  disabled,
  onStep,
  onToggle,
}: {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly playing: boolean;
  readonly disabled: boolean;
  readonly onStep: (by: number) => void;
  readonly onToggle: () => void;
}) {
  return (
    <div className="editor-transport" aria-label="Transport">
      <div className="editor-transport-row">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onStep(-1)}
          aria-label="Previous frame"
          disabled={disabled || frame === 0}
        >
          <StepBack className="size-4" />
        </Button>
        <Button
          size="icon-sm"
          variant="ghost"
          className="editor-play"
          disabled={disabled}
          onClick={onToggle}
          aria-label={playing ? 'Pause' : 'Play'}
        >
          {playing ? <Pause className="size-4" /> : <Play className="size-4" />}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onStep(1)}
          aria-label="Next frame"
          disabled={disabled || frame >= plan.frameCount - 1}
        >
          <StepForward className="size-4" />
        </Button>
        <span className="editor-timecode" data-testid="timecode">
          {timecode(plan, frame)}{' '}
          <span className="editor-duration">/ {timecode(plan, plan.frameCount)}</span>
          <span className="sr-only">
            {' '}
            · frame {frame} of {plan.frameCount}
          </span>
        </span>
      </div>
    </div>
  );
}
