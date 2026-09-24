/** One filled waveform silhouette, sampled from ingest's measured peaks. */
import type { Peaks } from './loader.js';

export function waveformPath(peaks: Peaks, from: number, to: number, columns = 320): string {
  const first = Math.max(0, Math.floor(from / peaks.bucketTicks));
  const last = Math.min(peaks.values.length - 1, Math.ceil(to / peaks.bucketTicks));
  if (last <= first) return '';
  const step = Math.max(1, Math.floor((last - first) / columns));
  const top: string[] = [];
  const bottom: string[] = [];
  for (let bucket = first; bucket <= last; bucket += step) {
    let low = 0;
    let high = 0;
    for (let inner = bucket; inner < Math.min(bucket + step, last + 1); inner += 1) {
      const [min, max] = peaks.values[inner]!;
      low = Math.min(low, min);
      high = Math.max(high, max);
    }
    const x = (((bucket * peaks.bucketTicks - from) / (to - from)) * 100).toFixed(2);
    top.push(`${x},${(50 - (high / 32_767) * 46).toFixed(1)}`);
    bottom.push(`${x},${(50 - (low / 32_767) * 46).toFixed(1)}`);
  }
  return `M${top.join(' L')} L${bottom.toReversed().join(' L')} Z`;
}
