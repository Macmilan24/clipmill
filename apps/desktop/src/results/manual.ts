/** Source positions are integer 90 kHz ticks; only the input presentation uses seconds. */
export function parseSourceTime(value: string): number | null {
  const text = value.trim();
  if (!/^\d+(?::\d{1,2}){0,2}(?:\.\d{1,3})?$/.test(text)) return null;
  const parts = text.split(':').map(Number);
  if (parts.some((part) => !Number.isFinite(part))) return null;
  if (parts.length > 1 && parts.at(-1)! >= 60) return null;
  if (parts.length === 3 && parts[1]! >= 60) return null;
  const seconds = parts.reduce((sum, part) => sum * 60 + part, 0);
  const ticks = Math.round(seconds * 90_000);
  return Number.isSafeInteger(ticks) ? ticks : null;
}
export function sourceTime(ticks: number): string {
  const millis = Math.round(Math.max(0, ticks) / 90);
  const seconds = Math.floor(millis / 1000);
  const fraction = millis % 1000;
  const clock = `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
  return fraction ? `${clock}.${String(fraction).padStart(3, '0')}` : clock;
}
export function manualSpanProblem(
  start: number | null,
  end: number | null,
  duration: number | null,
): string | null {
  if (duration === null || !Number.isSafeInteger(duration) || duration <= 0)
    return 'Source duration is unavailable. Refresh this analysis before creating a manual clip.';
  if (start === null || end === null || !Number.isSafeInteger(start) || !Number.isSafeInteger(end))
    return 'Use seconds, m:ss, or h:mm:ss, with up to three decimal places.';
  if (start < 0 || end <= start) return 'The end must be after the start.';
  if (end > duration) return 'The selection extends beyond the recording.';
  return null;
}
