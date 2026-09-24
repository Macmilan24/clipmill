/**
 * The review clock: where playing stops, how the shuttle speeds up, and that a
 * seek before the media is ready is not lost.
 */
import { describe, expect, it } from 'vitest';

import { PlaybackController } from '../src/inspector/playback.js';

const SECOND = 90_000;

/** Enough of a media element for the controller: a clock, events, play and pause. */
class FakeVideo extends EventTarget {
  currentTime = 0;
  readyState = 0;
  paused = true;
  playbackRate = 1;
  muted = false;
  seeking = false;
  play() {
    this.paused = false;
    this.dispatchEvent(new Event('play'));
    return Promise.resolve();
  }
  pause() {
    if (this.paused) return;
    this.paused = true;
    this.dispatchEvent(new Event('pause'));
  }
  ready() {
    this.readyState = 1;
    this.dispatchEvent(new Event('loadedmetadata'));
  }
}

function attached(ticks = 10 * SECOND) {
  const video = new FakeVideo();
  const controller = new PlaybackController(ticks);
  controller.attach(video as unknown as HTMLVideoElement);
  return { video, controller };
}

describe('seeking', () => {
  it('holds a seek made before the media is ready and lands it when it is', () => {
    const { video, controller } = attached();
    controller.seek(42 * SECOND);
    expect(video.currentTime).toBe(0);
    video.ready();
    expect(video.currentTime).toBe(42);
    expect(controller.getState().ticks).toBe(42 * SECOND);
  });

  it('opens at the position it was made with', () => {
    const { video } = attached(12 * SECOND);
    video.ready();
    expect(video.currentTime).toBe(12);
  });

  it('keeps a paused playhead where it was put, whatever frame the decoder shows', () => {
    const { video, controller } = attached();
    video.ready();
    controller.seek(20 * SECOND);
    controller.presented(19.98);
    expect(controller.getState().ticks).toBe(20 * SECOND);
  });
});

describe('playing through a cut', () => {
  it('stops on the out point when it plays through from inside', () => {
    const { video, controller } = attached();
    video.ready();
    controller.setCut({ startTicks: 10 * SECOND, endTicks: 30 * SECOND });
    controller.toggle();
    controller.presented(29.9);
    expect(controller.getState().playing).toBe(true);
    controller.presented(30.02);
    expect(controller.getState().playing).toBe(false);
    expect(controller.getState().ticks).toBe(30 * SECOND);
    expect(video.paused).toBe(true);
  });

  it('goes back to the in point instead when looping', () => {
    const { video, controller } = attached();
    video.ready();
    controller.setCut({ startTicks: 10 * SECOND, endTicks: 30 * SECOND });
    controller.setLoop(true);
    controller.toggle();
    controller.presented(29.9);
    controller.presented(30.02);
    expect(controller.getState().playing).toBe(true);
    expect(controller.getState().ticks).toBe(10 * SECOND);
    expect(video.currentTime).toBe(10);
  });

  it('plays on past the cut when it started after it, to hear what follows', () => {
    const { video, controller } = attached();
    video.ready();
    controller.setCut({ startTicks: 10 * SECOND, endTicks: 30 * SECOND });
    controller.seek(31 * SECOND);
    controller.toggle();
    controller.presented(31.5);
    controller.presented(40);
    expect(controller.getState().playing).toBe(true);
    expect(controller.getState().ticks).toBe(40 * SECOND);
  });

  it('starts over from the in point when play is pressed on the out point', () => {
    const { video, controller } = attached();
    video.ready();
    controller.setCut({ startTicks: 10 * SECOND, endTicks: 30 * SECOND });
    controller.seek(30 * SECOND);
    controller.toggle();
    expect(controller.getState().ticks).toBe(10 * SECOND);
    expect(controller.getState().playing).toBe(true);
  });
});

describe('playing around an edge', () => {
  it('plays two seconds either side and comes back to the edge', () => {
    const { video, controller } = attached();
    video.ready();
    controller.setCut({ startTicks: 10 * SECOND, endTicks: 30 * SECOND });
    controller.playAround(10 * SECOND);
    expect(video.currentTime).toBe(8);
    controller.presented(10.5);
    // Crossing into the cut is not the out point; playing carries on.
    expect(controller.getState().playing).toBe(true);
    controller.presented(12.01);
    expect(controller.getState().playing).toBe(false);
    expect(controller.getState().ticks).toBe(10 * SECOND);
  });

  it('plays across the out point without stopping there', () => {
    const { video, controller } = attached();
    video.ready();
    controller.setCut({ startTicks: 10 * SECOND, endTicks: 30 * SECOND });
    controller.playAround(30 * SECOND);
    controller.presented(29.9);
    controller.presented(30.5);
    expect(controller.getState().playing).toBe(true);
  });
});

describe('the shuttle', () => {
  it('doubles each press in the same direction, up to eight times', () => {
    const { video, controller } = attached();
    video.ready();
    controller.shuttle(1);
    expect(controller.getState().rate).toBe(1);
    controller.shuttle(1);
    controller.shuttle(1);
    controller.shuttle(1);
    controller.shuttle(1);
    expect(controller.getState().rate).toBe(8);
    expect(video.playbackRate).toBe(8);
  });

  it('plays a plain play at the chosen speed', () => {
    const { video, controller } = attached();
    video.ready();
    controller.setSpeed(1.5);
    controller.toggle();
    expect(video.playbackRate).toBe(1.5);
    controller.pause();
    expect(controller.getState().playing).toBe(false);
  });

  it('steps whole frames and pauses to do it', () => {
    const { video, controller } = attached(SECOND);
    video.ready();
    controller.toggle();
    controller.step(2);
    expect(controller.getState().playing).toBe(false);
    expect(controller.getState().ticks).toBe(SECOND + 2 * 3_003);
  });
});
