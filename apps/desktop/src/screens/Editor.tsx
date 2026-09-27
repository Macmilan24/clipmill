/**
 * The Editor: transcript, program monitor, properties and timeline around one
 * approved clip. Every picture, crop and caption comes from the preview plan the
 * render code computed; the player maps media time onto the plan's program frames.
 */
import {
  ArrowLeft,
  Check,
  Keyboard,
  Link2,
  Maximize2,
  Minimize2,
  Redo2,
  Undo2,
  Upload,
} from 'lucide-react';
import {
  type CSSProperties,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import type { EditIr } from '@clipmill/contracts';

import { Button } from '../components/ui/button.js';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '../components/ui/empty.js';
import type { EditCommandJson, FaceSighting, PreviewPlan } from '../daemon/client.js';
import { type Cut, clockTenths } from '../inspector/review.js';
import { TipButton } from '../inspector/TipButton.js';
import type { Filmstrip, Peaks } from '../results/loader.js';
import { type Transcript, snapStart } from '../results/transcript.js';
import {
  batch,
  removeCaptionWord,
  removeCropKeyframe,
  removeGainPoint,
  splitSegment,
} from '../editor/commands.js';
import { removeOverlay, suggestedHook } from '../editor/overlays.js';
import type { AssetAccess } from '../editor/brand.js';
import { useMusicBed } from '../editor/music.js';
import { EditorMonitor, type MonitorPlayback } from '../editor/EditorMonitor.js';
import { EditorProperties } from '../editor/EditorProperties.js';
import { EditorTimeline, type Tool, deleteRange } from '../editor/EditorTimeline.js';
import { EditorTranscript } from '../editor/EditorTranscript.js';
import {
  type EditorSelection,
  type PropertiesTab,
  NOTHING,
  positions,
  tabFor,
} from '../editor/selection.js';
import {
  editPoints,
  extentOf,
  freshSegmentId,
  panSpan,
  programTicks,
  ticksOfFrame,
  zoomSpan,
} from '../editor/timeline.js';
import {
  cutWords,
  programSilences,
  programWords,
  rippleRange,
  shownCues,
} from '../editor/transcript.js';
import { useDraftAudio } from '../editor/useDraftAudio.js';
import { useEditorKeys } from '../editor/useEditorKeys.js';
import { ClipTitle, HistoryButton } from '../editor/EditorHistory.js';
import { exactCaptionsOf } from '../editor/exactCaptions.js';
import type { HistoryStep } from '../editor/history.js';
import type { EditorFocus } from '../shell/route.js';
import { openShortcuts } from '../shell/ShortcutSheet.js';
import { type CoachMark, CoachMarks } from '../onboarding/CoachMarks.js';
import {
  gainAt,
  proxySecondsAt,
  resolvePlaybackFrame,
  segmentAt,
  sourceTicksAt,
} from '../editor/player.js';
import '../inspector/review.css';
import '../editor/edit.css';

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
  readonly focus?: EditorFocus | null;
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
  readonly onRelink?: (() => void) | null;
  readonly relinking?: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onUndo: () => void;
  readonly onRedo: () => void;
  readonly onResolve: (frame: number) => void;
  /** Follow one face through the section at `frame`. Absent hides the faces. */
  readonly onFollow?: ((frame: number, trackId: number) => void) | null;
  /** The faces seen over a span of the source, for choosing whom to follow. */
  readonly loadFaces?:
    ((startTicks: number, endTicks: number) => Promise<readonly FaceSighting[]>) | null;
  /** The clip's whole edit history, newest last. Absent hides History. */
  readonly onLoadHistory?: (() => Promise<readonly HistoryStep[]>) | null;
  /** The person's pictures and sounds. Absent shows no logo and offers none. */
  readonly assets?: AssetAccess | null;
  /** Where a pinned caption font is served from. Absent keeps CSS captions. */
  readonly fontUrl?: ((file: string) => string) | null;
  /** Where a pinned emoji's picture is served from. Absent draws the character. */
  readonly emojiUrl?: ((code: string) => string) | null;
  /**
   * The captions under a look not chosen yet, as the export would write them.
   * Absent makes every look a saved choice, as before.
   */
  readonly previewCaptions?: ((draft: CaptionDraft) => Promise<string | null>) | null;
}

/** A caption look being tried: a preset, clip-wide options, or both. */
export interface CaptionDraft {
  readonly styleRef?: string;
  readonly options?: NonNullable<EditIr['captions']['options']>;
}

/** The Editor's tips, the first time it opens. */
const EDITOR_TIPS: readonly CoachMark[] = [
  {
    target: 'edit-transcript',
    title: 'Edit by word',
    body: 'Select words to cut them from the picture and the sound, hide them from the captions, or correct them. Fillers and long pauses are gathered above for review.',
  },
  {
    target: 'edit-preview',
    title: 'The picture is yours to frame',
    body: 'Drag it to reframe, pinch or ⌘-scroll to zoom, and drag a caption to move it. Original shows the whole frame, and a face there to follow.',
  },
  {
    target: 'edit-properties',
    title: 'Captions, framing and sound',
    body: 'Try a look before choosing it. Every change is one step: undo takes it back, and History lists them all.',
  },
  {
    target: 'edit-export',
    title: 'When it is ready',
    body: 'Export makes the 9:16 file and its subtitles, and shows you the clip first.',
  },
];

const PANELS_KEY = 'clipmill.editor.panels';
const DEFAULT_PANELS = { left: 300, right: 320, lanes: 1 };

function remembered<T>(key: string, fallback: T): T {
  try {
    const stored = localStorage.getItem(key);
    return stored ? (JSON.parse(stored) as T) : fallback;
  } catch {
    return fallback;
  }
}

function remember(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* This session keeps it. */
  }
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
  focus = null,
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
  onRelink = null,
  relinking = false,
  onApply,
  onUndo,
  onRedo,
  onResolve,
  onFollow = null,
  loadFaces = null,
  fontUrl = null,
  emojiUrl = null,
  previewCaptions = null,
  onLoadHistory = null,
  assets = null,
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
  const [speed, setSpeed] = useState(1);
  const [loop, setLoop] = useState(false);
  const [muted, setMuted] = useState(false);
  const reverseTimer = useRef<ReturnType<typeof setInterval> | null>(null);
  const [selection, setSelection] = useState<EditorSelection>(NOTHING);
  const [tab, setTab] = useState<PropertiesTab>('captions');
  // A caption look being tried: the script libass draws until it is chosen
  // or let go. Tagged with the revision it was drawn over, so an edit that
  // lands meanwhile shows the saved captions rather than a stale draft.
  const [draft, setDraft] = useState<{ ass: string; revision: number } | null>(null);
  const draftRequest = useRef(0);
  const tryLook = useCallback(
    (look: CaptionDraft | null) => {
      draftRequest.current += 1;
      const request = draftRequest.current;
      if (!look || !previewCaptions || !plan) {
        setDraft(null);
        return;
      }
      const revision = plan.revision;
      void previewCaptions(look).then((ass) => {
        if (request === draftRequest.current && ass !== null) setDraft({ ass, revision });
      });
    },
    [previewCaptions, plan],
  );
  const clipWords = useMemo(() => (plan ? programWords(plan, transcript) : []), [plan, transcript]);
  // A hook title starts as what the clip is called, or the first thing said.
  const hook = useMemo(() => {
    if (!plan) return 'Your hook here';
    const opening: string[] = [];
    for (const word of clipWords) {
      opening.push(word.text);
      if (/[.?!]$/.test(word.text) || opening.length >= 14) break;
    }
    return suggestedHook(document?.title ?? labels?.clip, opening.join(' '));
  }, [plan, clipWords, document?.title, labels?.clip]);
  const exactCaptions = useMemo(
    () =>
      exactCaptionsOf(
        plan,
        fontUrl,
        draft && plan && draft.revision === plan.revision ? draft.ass : plan?.ass,
      ),
    [plan, fontUrl, draft],
  );
  const [tool, setTool] = useState<Tool>('select');
  const [snap, setSnap] = useState(true);
  const [marks, setMarks] = useState<{ in: number | null; out: number | null }>({
    in: null,
    out: null,
  });
  const [markers, setMarkers] = useState<readonly number[]>([]);
  const [findSignal, setFindSignal] = useState(0);
  const [panels, setPanels] = useState(() => remembered(PANELS_KEY, DEFAULT_PANELS));
  const [focused, setFocused] = useState(false);
  const [playbackProblem, setPlaybackProblem] = useState<string | null>(null);
  const [view, setView] = useState<Cut>(() =>
    plan ? extentOf(plan) : { startTicks: 0, endTicks: 1 },
  );
  // What is drawn is always a frame the program has. A trim can shorten the
  // program under a playhead that did not move, and between that render and
  // the effect below that moves it back, a frame past the end would find no
  // segment — and no segment would take the picture down with it.
  const frame = plan ? Math.max(0, Math.min(playhead, plan.frameCount - 1)) : playhead;
  // The music under the voice, on the program's clock at the render's levels.
  const musicElement = useRef<HTMLAudioElement>(null);
  useMusicBed(musicElement, plan ?? null, frame, playing, muted);
  const draftAudio = useDraftAudio(
    video,
    plan ? gainAt(plan, frame) : 0,
    plan?.gain.some((point) => point.gainDb > 0) ?? false,
  );
  const range =
    marks.in !== null && marks.out !== null && marks.out > marks.in
      ? { first: marks.in, last: marks.out }
      : null;

  const select = useCallback((next: EditorSelection) => {
    setSelection(next);
    const wanted = tabFor(next);
    if (wanted) setTab(wanted);
  }, []);

  const stopShuttle = useCallback(() => {
    if (reverseTimer.current) clearInterval(reverseTimer.current);
    reverseTimer.current = null;
  }, []);

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
    setSelection(NOTHING);
    setMarks({ in: null, out: null });
    setTool('select');
    stopShuttle();
    setMarkers(remembered<number[]>(`clipmill.editor.markers.${docId}`, []));
    setFocused(false);
    setPlaybackProblem(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- a new document only
  }, [docId, updateFrame]);

  // A trim or an extension changes how far the timeline reaches; a view
  // outside the new reach is put back inside it.
  useEffect(() => {
    if (!plan) return;
    const extent = extentOf(plan);
    setView((current) => {
      const span = current.endTicks - current.startTicks;
      const whole = extent.endTicks - extent.startTicks;
      if (span >= whole || current.endTicks <= 1) return extent;
      return panSpan(current, 0, extent);
    });
  }, [plan, docId]);

  useEffect(() => {
    if (video.current) video.current.playbackRate = speed;
  }, [speed]);

  /**
   * Put the playhead on a program frame, and the media element where that
   * frame is in the proxy. Every seek goes through here — the transport, the
   * timeline, the transcript, the keys — so there is one place the two clocks meet.
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
        // Not when it is there already: a seek in place fires `seeking`, which
        // drops the decoded frame the composition redraws a paused edit from,
        // and a paused seek to the same frame never decodes a new one — so a
        // layout chosen while paused would not show until the next frame.
        if (Math.abs(element.currentTime - seconds) > 1 / 90_000) element.currentTime = seconds;
      }
    },
    [plan, proxyUrls, updateFrame],
  );

  const focusedCue = useRef<string | null>(null);
  useEffect(() => {
    if (!focus || !plan || !docId) return;
    const key = `${docId}:${focus.track}:${focus.cueId ?? ''}`;
    if (focusedCue.current === key) return;
    const cues = focus.track === 'reading' ? (plan.readingCues ?? plan.cues) : plan.cues;
    const target = cues.find((cue) => cue.cueId === focus.cueId) ?? cues[0];
    if (!target) return;
    focusedCue.current = key;
    setTab('captions');
    setSelection({ kind: 'cue', cueId: target.cueId });
    seek(target.firstFrame);
  }, [docId, focus, plan, seek]);

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

  const step = useCallback(
    (by: number) => {
      stopShuttle();
      seek(clockFrame.current + by);
    },
    [seek, stopShuttle],
  );

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

  const play = useCallback(() => {
    const element = video.current;
    if (!element || !plan) return;
    setPlaybackProblem(null);
    playbackFailed.current = false;
    if (waitingForReference.current) {
      resumeAfterReference.current = true;
      setPlaying(true);
      return;
    }
    if (!element.paused) return;
    if (clockFrame.current >= plan.frameCount - 1 || element.ended) seek(range?.first ?? 0);
    draftAudio.connect();
    void element.play().catch(() => {
      stopWithProblem('Playback could not start. Try playing the clip again.');
    });
  }, [plan, seek, range, draftAudio.connect, stopWithProblem]);

  const pause = useCallback(() => {
    stopShuttle();
    if (waitingForReference.current) {
      resumeAfterReference.current = false;
      setPlaying(false);
      return;
    }
    const element = video.current;
    if (element && !element.paused) element.pause();
  }, [stopShuttle]);

  const togglePlayback = useCallback(() => {
    const element = video.current;
    if (!element || !plan) return;
    if (reverseTimer.current) {
      stopShuttle();
      setSpeed(1);
      return;
    }
    const going = waitingForReference.current ? resumeAfterReference.current : !element.paused;
    if (going) pause();
    else play();
  }, [plan, pause, play, stopShuttle]);

  /** J and L: each press doubles the speed that way, up to four times. */
  const shuttle = useCallback(
    (direction: 1 | -1) => {
      if (!plan) return;
      const element = video.current;
      if (direction === 1) {
        stopShuttle();
        setSpeed((current) => (element && !element.paused ? Math.min(4, current * 2) : 1));
        play();
        return;
      }
      if (element && !element.paused) element.pause();
      const rate = reverseTimer.current ? Math.min(4, speed * 2) : 1;
      stopShuttle();
      setSpeed(rate);
      reverseTimer.current = setInterval(() => {
        if (clockFrame.current <= 0) {
          stopShuttle();
          return;
        }
        seek(
          clockFrame.current - Math.max(1, Math.round((rate * plan.rateNum) / plan.rateDen / 12)),
        );
      }, 1000 / 12);
    },
    [plan, play, seek, speed, stopShuttle],
  );

  useEffect(() => stopShuttle, [stopShuttle]);

  // This callback returns the frame synchronously: the canvas must use its
  // crop for these decoded pixels, without waiting for React to move the UI.
  const loopRef = useRef({ loop, range });
  loopRef.current = { loop, range };
  const onProxyTime = useCallback(
    (seconds: number, atMediaEnd = false): number | null => {
      const element = video.current;
      if (!plan || !element || element.seeking || playbackFailed.current) return null;
      if (waitingForReference.current) return clockFrame.current;
      const active = segmentAt(plan, clockFrame.current);
      if (!active || proxyUrls.get(active.sourceFingerprint) !== element.getAttribute('src'))
        return null;
      // A paused seek requests a program frame even when the lower-rate proxy
      // decodes an earlier frame. Do not move the playhead back to that frame.
      if (element.paused && !atMediaEnd) {
        // Pausing in the fractional boundary gap must not repaint the newly
        // decoded incoming bitmap under the held outgoing frame's crop.
        return resolvePlaybackFrame(plan, clockFrame.current, seconds).hold
          ? null
          : clockFrame.current;
      }
      const next = resolvePlaybackFrame(plan, clockFrame.current, seconds);
      const looping = loopRef.current;
      if (looping.loop && looping.range && next.frame >= looping.range.last) {
        seek(looping.range.first);
        return null;
      }
      if (next.seek || next.ended) {
        if (next.ended) {
          if (looping.loop) {
            seek(looping.range?.first ?? 0);
            void element.play().catch(() => {
              stopWithProblem('Playback could not continue. Try playing the clip again.');
            });
            return null;
          }
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
    [plan, proxyUrls, seek, updateFrame, stopWithProblem],
  );

  const onMetadata = useCallback(() => {
    const element = video.current;
    if (!element) return;
    element.playbackRate = speed;
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
  }, [seek, speed, stopWithProblem]);

  const addMarker = useCallback(() => {
    setMarkers((current) => {
      const next = [...new Set([...current, clockFrame.current])].toSorted((a, b) => a - b);
      remember(`clipmill.editor.markers.${docId}`, next);
      return next;
    });
  }, [docId]);

  /** Split the section under a frame, between two words when snapping. */
  const splitAt = useCallback(
    (at: number) => {
      if (!plan || busy) return;
      const part = segmentAt(plan, at);
      let ticks = sourceTicksAt(plan, at);
      if (!part || ticks === null) return;
      if (snap && transcript && transcript.words.length > 0) {
        const between = snapStart(transcript, ticks);
        if (Math.abs(between - ticks) <= 0.3 * 90_000) ticks = between;
      }
      if (ticks <= part.inTicks || ticks >= part.outTicks) return;
      const existing = document?.video.segments?.map((item) => item.segment_id) ?? [];
      onApply(splitSegment(part.segmentId, ticks, freshSegmentId(existing, part.segmentId, ticks)));
      select({ kind: 'section', segmentId: part.segmentId });
    },
    [plan, busy, snap, transcript, document, onApply, select],
  );

  const deleteMarked = useCallback(() => {
    if (!plan || busy || !range) return;
    const command = deleteRange(plan, range.first, range.last);
    if (!command) return;
    onApply(command);
    setMarks({ in: null, out: null });
    seek(range.first);
  }, [plan, busy, range, onApply, seek]);

  /** Delete what is selected: words, a caption, a keyframe, a point, a section or the range. */
  const remove = useCallback(() => {
    if (!plan || busy) return;
    let command: EditCommandJson | null = null;
    if (selection.kind === 'words') {
      command = cutWords(
        plan,
        programWords(plan, transcript),
        positions(selection.range),
        programSilences(plan, transcript),
      );
    } else if (selection.kind === 'keyframe') {
      command = removeCropKeyframe(selection.tTicks, selection.segmentId, selection.secondary);
    } else if (selection.kind === 'gain') {
      command = removeGainPoint(selection.tTicks);
    } else if (selection.kind === 'overlay') {
      command = removeOverlay(selection.overlayId);
    } else if (selection.kind === 'cue') {
      const cue = plan.cues.find((item) => item.cueId === selection.cueId);
      const count = cue?.lines.flat().length ?? 0;
      command =
        count > 0
          ? batch(
              Array.from({ length: count }, (_, index) =>
                removeCaptionWord(selection.cueId, count - 1 - index, plan.presentation),
              ),
            )
          : null;
    } else if (selection.kind === 'section' && plan.segments.length > 1) {
      const part = plan.segments.find((item) => item.segmentId === selection.segmentId);
      if (part) {
        command = rippleRange(
          plan,
          part.programStartTicks,
          part.programStartTicks + part.outTicks - part.inTicks,
        );
      }
    } else if (range) {
      deleteMarked();
      return;
    }
    if (command) {
      onApply(command);
      setSelection(NOTHING);
    }
  }, [plan, busy, selection, transcript, range, deleteMarked, onApply]);

  const zoomBy = useCallback(
    (factor: number) => {
      if (!plan) return;
      setView((current) =>
        zoomSpan(current, factor, ticksOfFrame(plan, clockFrame.current), extentOf(plan)),
      );
    },
    [plan],
  );

  const walkEdits = useCallback(
    (direction: 1 | -1) => {
      if (!plan) return;
      const points = editPoints(plan);
      const now = clockFrame.current;
      const target =
        direction === 1
          ? points.find((point) => point > now)
          : points.findLast((point) => point < now);
      if (target !== undefined) step(target - now);
    },
    [plan, step],
  );

  useEditorKeys(
    {
      toggle: togglePlayback,
      pause: () => {
        pause();
        setSpeed(1);
      },
      shuttle,
      step,
      goToStart: () => step(-clockFrame.current),
      goToEnd: () => plan && step(plan.frameCount - 1 - clockFrame.current),
      previousEdit: () => walkEdits(-1),
      nextEdit: () => walkEdits(1),
      markIn: () => setMarks((current) => ({ in: clockFrame.current, out: current.out })),
      markOut: () => setMarks((current) => ({ in: current.in, out: clockFrame.current })),
      clearMarks: () => setMarks({ in: null, out: null }),
      split: () => splitAt(clockFrame.current),
      remove,
      selectTool: () => setTool('select'),
      bladeTool: () => setTool('blade'),
      toggleSnap: () => setSnap((current) => !current),
      addMarker,
      find: () => setFindSignal((current) => current + 1),
      zoomIn: () => zoomBy(1 / 1.5),
      zoomOut: () => zoomBy(1.5),
      zoomFit: () => plan && setView(extentOf(plan)),
      undo: () => !busy && canUndo && onUndo(),
      redo: () => !busy && canRedo && onRedo(),
      exportClip: () => onExport?.(),
      escape: () => {
        if (focused) setFocused(false);
        else if (selection.kind !== 'clip') setSelection(NOTHING);
        else if (marks.in !== null || marks.out !== null) setMarks({ in: null, out: null });
        else if (tool === 'blade') setTool('select');
      },
    },
    plan !== null && docId !== null,
  );

  /**
   * Taller or shorter timeline lanes, by dragging the timeline's top edge:
   * more filmstrip and waveform for close work, more picture otherwise.
   */
  const resizeLanes = (event: ReactPointerEvent) => {
    event.preventDefault();
    const origin = event.clientY;
    const starting = panels.lanes ?? 1;
    const move = (next: PointerEvent) =>
      setPanels((current) => ({
        ...current,
        lanes: Math.max(0.75, Math.min(2.4, starting + (origin - next.clientY) / 130)),
      }));
    const done = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', done);
      setPanels((current) => {
        remember(PANELS_KEY, current);
        return current;
      });
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', done);
  };

  const resize = (side: 'left' | 'right', event: ReactPointerEvent) => {
    event.preventDefault();
    const origin = event.clientX;
    const starting = panels[side];
    const move = (next: PointerEvent) =>
      setPanels((current) => ({
        ...current,
        [side]: Math.max(
          248,
          Math.min(460, starting + (next.clientX - origin) * (side === 'left' ? 1 : -1)),
        ),
      }));
    const done = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', done);
      setPanels((current) => {
        remember(PANELS_KEY, current);
        return current;
      });
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', done);
  };

  const segment = useMemo(() => (plan ? segmentAt(plan, frame) : null), [plan, frame]);

  if (loading) {
    return (
      <div className="edit-loading" role="status">
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
          {onRelink && (
            <Button variant="outline" disabled={relinking} onClick={onRelink}>
              <Link2 className="size-4" />
              {relinking ? 'Checking recording…' : 'Locate recording…'}
            </Button>
          )}
          <Button variant="outline" onClick={onOpenResults}>
            Go to Results
          </Button>
        </Empty>
      </div>
    );
  }

  const proxyUrl = segment ? (proxyUrls.get(segment.sourceFingerprint) ?? null) : null;
  const alert = problem || playbackProblem || draftAudio.problem || segment?.framingWarning;
  const captions = shownCues(plan, document).length || plan.cues.length;
  const playback: MonitorPlayback = {
    frame,
    heardFrame:
      playing && draftAudio.lag > 0
        ? Math.max(
            0,
            frame - Math.round((draftAudio.lag * plan.rateNum) / Math.max(1, plan.rateDen)),
          )
        : frame,
    playing,
    speed,
    loop,
    muted,
    preparing: preparingPreview,
    onToggle: togglePlayback,
    onStep: step,
    onSeek: (target) => {
      stopShuttle();
      seek(target);
    },
    onSpeed: setSpeed,
    onLoop: setLoop,
    onMuted: setMuted,
    onProxyTime,
    onMetadata,
    onPlaying: onMediaPlaying,
    onBuffering,
    onError: () =>
      stopWithProblem('The preview could not be loaded. Reopen the clip to try again.'),
  };

  return (
    <div
      className="review-workspace edit-workspace"
      data-focused={focused ? 'true' : undefined}
      style={{ '--edit-lane-scale': String(panels.lanes ?? 1) } as CSSProperties}
    >
      <CoachMarks place="editor" marks={EDITOR_TIPS} />
      {plan.music && assets && (
        // The music the clip plays under the voice; the picture's own sound
        // plays from the proxy.
        <audio
          ref={musicElement}
          src={assets.url(plan.music.asset)}
          preload="auto"
          loop
          hidden
          data-testid="music"
        />
      )}
      <header className="review-heading">
        <div className="review-identity">
          <TipButton label="Back to results" onClick={onOpenResults}>
            <ArrowLeft className="size-4" />
          </TipButton>
          <div className="review-title">
            <ClipTitle
              title={document?.title ?? null}
              fallback={labels?.clip ?? 'Clip editor'}
              busy={busy}
              onApply={onApply}
            />
            <p>
              <span>{labels?.project ?? 'Your edit'}</span>
              <span aria-hidden="true">·</span>
              <span className="mono">{clockTenths(programTicks(plan))}</span>
              <span aria-hidden="true">·</span>
              <span>
                {captions} {captions === 1 ? 'caption' : 'captions'}
              </span>
            </p>
          </div>
        </div>
        <div className="review-heading-actions">
          <span
            role="status"
            className="edit-save-state"
            data-state={busy ? 'saving' : problem ? 'problem' : 'saved'}
          >
            {busy ? (
              'Saving…'
            ) : problem ? (
              'Not saved'
            ) : (
              <>
                <Check className="size-3.5" aria-hidden="true" /> Saved
              </>
            )}
          </span>
          <TipButton label="Undo" disabled={!canUndo || busy} onClick={onUndo}>
            <Undo2 className="size-4" />
          </TipButton>
          <TipButton label="Redo" disabled={!canRedo || busy} onClick={onRedo}>
            <Redo2 className="size-4" />
          </TipButton>
          <TipButton label="Keyboard shortcuts" onClick={openShortcuts}>
            <Keyboard className="size-4" />
          </TipButton>
          {onLoadHistory && (
            <HistoryButton
              revision={plan.revision}
              busy={busy}
              onLoad={onLoadHistory}
              onApply={onApply}
            />
          )}
          <span className="review-divider" aria-hidden="true" />
          <TipButton
            label={focused ? 'Restore editing panels' : 'Focus preview'}
            pressed={focused}
            onClick={() => setFocused(!focused)}
          >
            {focused ? <Minimize2 className="size-4" /> : <Maximize2 className="size-4" />}
          </TipButton>
          {onExport && (
            <Button
              size="sm"
              onClick={onExport}
              disabled={busy}
              aria-label="Export this clip"
              data-coach="edit-export"
            >
              <Upload className="size-4" aria-hidden="true" />
              Export
            </Button>
          )}
        </div>
      </header>
      {alert && (
        <p role="alert" className="edit-alert">
          {alert}
        </p>
      )}
      {onRelink && (
        <div role="status" className="edit-alert edit-alert-action">
          <span>
            The original recording is no longer where it was imported from. Editing continues from
            the preview copy; exporting needs the original.
          </span>
          <Button variant="outline" size="sm" disabled={relinking} onClick={onRelink}>
            <Link2 className="size-4" aria-hidden="true" />
            {relinking ? 'Checking recording…' : 'Locate recording…'}
          </Button>
        </div>
      )}
      <div
        className="edit-body"
        style={
          {
            '--edit-left': `${panels.left}px`,
            '--edit-right': `${panels.right}px`,
          } as CSSProperties
        }
      >
        <EditorTranscript
          plan={plan}
          document={document}
          transcript={transcript}
          frame={frame}
          playing={playing}
          busy={busy}
          selected={selection.kind === 'words' ? selection.range : null}
          findSignal={findSignal}
          onSelect={(words) => setSelection(words ? { kind: 'words', range: words } : NOTHING)}
          onSeek={seek}
          onApply={onApply}
        />
        <div
          className="edit-resizer"
          role="separator"
          aria-label="Resize the transcript"
          aria-orientation="vertical"
          onPointerDown={(event) => resize('left', event)}
        />
        <EditorMonitor
          plan={plan}
          docId={docId}
          videoRef={video}
          proxyUrl={proxyUrl}
          proxyUrls={proxyUrls}
          startSeconds={proxySecondsAt(plan, frame)}
          playback={playback}
          range={range}
          busy={busy}
          selection={selection}
          onSelect={select}
          onApply={onApply}
          captions={exactCaptions}
          captionOptions={document?.captions.options ?? {}}
          loadFaces={loadFaces}
          onFollow={onFollow ? (trackId) => onFollow(frame, trackId) : null}
          assetUrl={assets?.url ?? null}
          emojiUrl={emojiUrl}
        />
        <div
          className="edit-resizer"
          role="separator"
          aria-label="Resize the properties"
          aria-orientation="vertical"
          onPointerDown={(event) => resize('right', event)}
        />
        <EditorProperties
          plan={plan}
          focus={focus}
          document={document}
          frame={frame}
          selection={selection}
          tab={tab}
          onTab={setTab}
          busy={busy}
          resolving={resolving}
          resolveRefusal={resolveRefusal}
          onApply={onApply}
          onResolve={() => onResolve(frame)}
          onSelect={select}
          onSeek={seek}
          onTryLook={previewCaptions ? tryLook : null}
          fonts={plan.fonts ?? []}
          hook={hook}
          words={clipWords}
          assets={assets}
          emojiUrl={emojiUrl}
        />
      </div>
      <div
        className="edit-resizer-lanes"
        role="separator"
        aria-label="Resize the timeline"
        aria-orientation="horizontal"
        onPointerDown={resizeLanes}
        onDoubleClick={() => {
          const next = { ...panels, lanes: 1 };
          setPanels(next);
          remember(PANELS_KEY, next);
        }}
      />
      <EditorTimeline
        plan={plan}
        document={document}
        transcript={transcript}
        filmstrip={filmstrip}
        peaks={peaks}
        filmstripUrl={filmstripUrl}
        frame={frame}
        playing={playing}
        view={view}
        onView={setView}
        selection={selection}
        busy={busy}
        tool={tool}
        onTool={setTool}
        snap={snap}
        onSnap={setSnap}
        marks={marks}
        onClearMarks={() => setMarks({ in: null, out: null })}
        markers={markers}
        onMarker={addMarker}
        onSeek={seek}
        onSelect={select}
        onApply={onApply}
        onSplit={splitAt}
        onDeleteRange={deleteMarked}
        emojiUrl={emojiUrl}
      />
    </div>
  );
}
