/**
 * The half of a preview plan that maps the program onto a recording.
 *
 * Every plan test used to carry only the program: frames, crops, cues. The
 * contract now also says where those frames come from, and a fixture that
 * left that out would test a player against a plan the daemon never sends.
 * One segment, several minutes into a landscape source, is the shape the
 * "which footage" bugs hide in — a plan at the recording's opening cannot
 * tell a correct seek from a seek to zero.
 */
import type { PreviewPlan } from '../../src/daemon/client.js';

export const FINGERPRINT = `sha256:${'aa'.repeat(32)}`;
export const PROXY_ID = 'sha256:proxy-aa';
export const TICKS = 90_000;

/** Ticks a plan's frame count spans at its rate. */
export function programTicks(plan: Pick<PreviewPlan, 'frameCount' | 'rateNum' | 'rateDen'>) {
  return Math.round((plan.frameCount * plan.rateDen * TICKS) / plan.rateNum);
}

/**
 * One segment starting `inSeconds` into the source, covering the whole plan,
 * on a 1920×1080 source whose proxy starts at the recording's opening.
 */
export function mapping(
  plan: Pick<PreviewPlan, 'frameCount' | 'rateNum' | 'rateDen'>,
  inSeconds = 600,
  overrides: Partial<Pick<PreviewPlan, 'segments' | 'sources' | 'proxies'>> = {},
): Pick<PreviewPlan, 'segments' | 'sources' | 'proxies'> {
  const inTicks = inSeconds * TICKS;
  return {
    segments: [
      {
        segmentId: 'seg_1',
        sourceFingerprint: FINGERPRINT,
        inTicks,
        outTicks: inTicks + programTicks(plan),
        programStartTicks: 0,
        firstFrame: 0,
        endFrame: plan.frameCount,
      },
    ],
    sources: [
      {
        sourceFingerprint: FINGERPRINT,
        sourceId: 'src_p_old',
        displayWidth: 1920,
        displayHeight: 1080,
      },
    ],
    proxies: [
      {
        sourceFingerprint: FINGERPRINT,
        artifactId: PROXY_ID,
        file: 'proxy.mp4',
        coverageStartTicks: 0,
        coverageEndTicks: 3600 * TICKS,
        width: 1280,
        height: 720,
        rateNum: plan.rateNum,
        rateDen: plan.rateDen,
      },
    ],
    ...overrides,
  };
}
