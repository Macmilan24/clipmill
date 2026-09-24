/**
 * The review clock shared by the monitor, strips and transcript.
 * Readers subscribe to the value they draw. The playhead may go anywhere in the
 * recording; playing through the cut stops at its out point, or loops.
 */
import { TICKS_PER_SECOND } from '../results/model.js';
import { FRAME_TICKS, type Cut, clamp } from './review.js';

export interface PlaybackState {
  /** Where the playhead is, in source ticks. */
  readonly ticks: number;
  readonly playing: boolean;
  /**
   * The shuttle rate while playing: positive forwards, negative backwards.
   * Zero when paused.
   */
  readonly rate: number;
  /** The speed a plain play runs at. */
  readonly speed: number;
  readonly loop: boolean;
  readonly muted: boolean;
  /** Whether the media can be positioned yet. */
  readonly ready: boolean;
  readonly problem: string | null;
}

/** Plain-play speeds, for a reviewer skimming a talk. */
export const SPEEDS = [1, 1.25, 1.5, 2] as const;

/** Pre-roll and post-roll around an edge, as an editor's "play around". */
const AROUND_TICKS = 2 * TICKS_PER_SECOND;
/** Fastest shuttle, either way. */
const MAX_SHUTTLE = 8;

type Listener = () => void;

export class PlaybackController {
  private video: HTMLVideoElement | null = null;
  private readonly listeners = new Set<Listener>();
  private state: PlaybackState;
  private cut: Cut | null = null;
  private around: { readonly until: number; readonly returnTo: number } | null = null;
  private reverse: number | null = null;
  private pending: number | null = null;
  private readonly unbind: (() => void)[] = [];

  /** Starts paused at `ticks`, so nothing reads a playhead at zero first. */
  constructor(ticks = 0) {
    this.state = {
      ticks: Math.max(0, Math.round(ticks)),
      playing: false,
      rate: 0,
      speed: 1,
      loop: false,
      muted: false,
      ready: false,
      problem: null,
    };
  }

  readonly subscribe = (listener: Listener): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  readonly getState = (): PlaybackState => this.state;

  /** Take over a media element, or let go of the current one with `null`. */
  attach(video: HTMLVideoElement | null): void {
    if (video === this.video) return;
    this.stopReverse();
    for (const release of this.unbind.splice(0)) release();
    this.video = video;
    this.update({ ready: false, playing: false, rate: 0, problem: null });
    if (!video) return;
    const on = (event: string, listener: () => void) => {
      video.addEventListener(event, listener);
      this.unbind.push(() => video.removeEventListener(event, listener));
    };
    on('loadedmetadata', () => {
      this.update({ ready: true });
      // The first seek that can land: before metadata a browser drops it.
      const wanted = this.pending ?? this.state.ticks;
      this.pending = null;
      video.currentTime = wanted / TICKS_PER_SECOND;
    });
    on('play', () => {
      if (this.reverse === null) {
        this.update({ playing: true, rate: this.state.rate || this.state.speed });
      }
    });
    on('pause', () => {
      if (this.reverse === null) this.update({ playing: false, rate: 0 });
    });
    on('ended', () => this.update({ playing: false, rate: 0 }));
    on('error', () =>
      this.update({
        playing: false,
        rate: 0,
        problem: 'The preview could not be loaded. Reopen this clip to try again.',
      }),
    );
    video.muted = this.state.muted;
    if (video.readyState >= 1) {
      this.update({ ready: true });
      video.currentTime = this.state.ticks / TICKS_PER_SECOND;
    }
  }

  /** The cut playing through stops at, or `null` to play freely. */
  setCut(cut: Cut | null): void {
    this.cut = cut;
  }

  /**
   * Put the playhead somewhere. Anywhere in the recording: the cut is where
   * playing stops, not where the playhead is allowed to be.
   */
  seek(ticks: number): void {
    this.around = null;
    const target = Math.max(0, Math.round(ticks));
    this.update({ ticks: target });
    const video = this.video;
    if (!video) return;
    if (video.readyState >= 1) {
      video.currentTime = target / TICKS_PER_SECOND;
    } else {
      this.pending = target;
    }
  }

  /**
   * A decoded frame reached the screen at `seconds` of media time.
   *
   * While paused the playhead stays where it was put: the proxy's frames need
   * not fall on the tick a person asked for, and a scrubber that crept back
   * to the decoded frame would disagree with the cut it was just set to.
   */
  presented(seconds: number): void {
    if (!this.state.playing || this.reverse !== null) return;
    const ticks = Math.round(seconds * TICKS_PER_SECOND);
    const previous = this.state.ticks;
    if (this.around && ticks >= this.around.until) {
      const back = this.around.returnTo;
      this.pause();
      this.seek(back);
      return;
    }
    const cut = this.cut;
    if (cut && !this.around && previous < cut.endTicks && ticks >= cut.endTicks) {
      if (this.state.loop) {
        this.seek(cut.startTicks);
        return;
      }
      this.pause();
      this.seek(cut.endTicks);
      return;
    }
    this.update({ ticks });
  }

  /** Play at the plain speed, or pause. */
  toggle(): void {
    if (this.state.playing) {
      this.pause();
      return;
    }
    const cut = this.cut;
    // Pressing play on the out point means "again", not "past the end".
    if (cut && Math.abs(this.state.ticks - cut.endTicks) < FRAME_TICKS) {
      this.seek(cut.startTicks);
    }
    this.play(this.state.speed);
  }

  play(rate = this.state.speed): void {
    const video = this.video;
    if (!video) return;
    this.stopReverse();
    video.playbackRate = rate;
    this.update({ problem: null, rate, playing: true });
    // A promise in every current webview; older media elements return nothing.
    const started = video.play() as Promise<void> | undefined;
    started?.catch(() => {
      this.update({ playing: false, rate: 0, problem: 'Playback could not start. Try again.' });
    });
  }

  pause(): void {
    this.stopReverse();
    if (this.video && !this.video.paused) this.video.pause();
    this.update({ playing: false, rate: 0 });
  }

  /**
   * J and L: each press in the same direction doubles the speed, and the
   * other direction starts over at normal speed.
   */
  shuttle(direction: 1 | -1): void {
    const current = this.state.playing ? this.state.rate : 0;
    const next =
      Math.sign(current) === direction ? clamp(current * 2, -MAX_SHUTTLE, MAX_SHUTTLE) : direction;
    this.around = null;
    if (next > 0) {
      this.play(next);
      return;
    }
    this.startReverse(-next);
  }

  /** Move by whole frames, paused, the way a jog wheel does. */
  step(frames: number): void {
    this.pause();
    this.seek(this.state.ticks + frames * FRAME_TICKS);
  }

  /**
   * Play a little either side of an edge and come back to it — how an editor
   * hears whether a cut lands on the breath or in the middle of a thought.
   */
  playAround(edgeTicks: number): void {
    this.pause();
    this.seek(Math.max(0, edgeTicks - AROUND_TICKS));
    this.around = { until: edgeTicks + AROUND_TICKS, returnTo: edgeTicks };
    this.play(1);
  }

  setSpeed(speed: number): void {
    this.update({ speed });
    if (this.state.playing && this.state.rate > 0 && this.video) {
      this.video.playbackRate = speed;
      this.update({ rate: speed });
    }
  }

  setLoop(loop: boolean): void {
    this.update({ loop });
  }

  setMuted(muted: boolean): void {
    if (this.video) this.video.muted = muted;
    this.update({ muted });
  }

  dispose(): void {
    this.attach(null);
    this.listeners.clear();
  }

  /**
   * Backwards is seeking, not playing: no webview plays video in reverse. A
   * new seek is asked for only once the last one has landed, so a slow
   * decoder shows fewer frames rather than falling ever further behind.
   */
  private startReverse(rate: number): void {
    const video = this.video;
    if (!video) return;
    this.stopReverse();
    video.pause();
    this.update({ playing: true, rate: -rate, problem: null });
    let last = performance.now();
    const tick = (now: number) => {
      const elapsed = (now - last) / 1000;
      last = now;
      if (!video.seeking) {
        const target = this.state.ticks - elapsed * rate * TICKS_PER_SECOND;
        if (target <= 0) {
          this.stopReverse();
          this.update({ ticks: 0, playing: false, rate: 0 });
          video.currentTime = 0;
          return;
        }
        this.update({ ticks: Math.round(target) });
        video.currentTime = target / TICKS_PER_SECOND;
      }
      this.reverse = requestAnimationFrame(tick);
    };
    this.reverse = requestAnimationFrame(tick);
  }

  private stopReverse(): void {
    if (this.reverse !== null) {
      cancelAnimationFrame(this.reverse);
      this.reverse = null;
      this.update({ playing: false, rate: 0 });
    }
  }

  private update(partial: Partial<PlaybackState>): void {
    let changed = false;
    for (const [key, value] of Object.entries(partial)) {
      if (this.state[key as keyof PlaybackState] !== value) {
        changed = true;
        break;
      }
    }
    if (!changed) return;
    this.state = { ...this.state, ...partial };
    for (const listener of this.listeners) listener();
  }
}
