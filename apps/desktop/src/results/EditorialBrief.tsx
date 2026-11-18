import { useState } from 'react';

import { COUNT_BOUNDS, DURATION_BOUNDS } from '../import/model.js';
import { Input } from '../components/ui/input.js';
import { Button } from '../components/ui/button.js';
import { type MindApi, type Moment, type RecordingRun } from '../daemon/mind.js';
import { clock, TICKS_PER_SECOND } from './model.js';

export function EditorialBrief({
  api,
  run,
  prompt,
  defaults,
  onStarted,
}: {
  readonly api: MindApi;
  readonly run: RecordingRun;
  readonly prompt: string;
  readonly defaults: {
    readonly minSeconds: number;
    readonly maxSeconds: number;
    readonly count: number;
  };
  readonly onStarted: (jobId: string) => void;
}) {
  const [draft, setDraft] = useState(prompt);
  const [limits, setLimits] = useState(defaults);
  const valid =
    Number.isFinite(limits.minSeconds) &&
    Number.isFinite(limits.maxSeconds) &&
    limits.minSeconds >= DURATION_BOUNDS.min &&
    limits.maxSeconds <= DURATION_BOUNDS.max &&
    limits.minSeconds < limits.maxSeconds &&
    Number.isInteger(limits.count) &&
    limits.count >= COUNT_BOUNDS.min &&
    limits.count <= COUNT_BOUNDS.max;
  const [matches, setMatches] = useState<readonly Moment[] | null>(null);
  const [busy, setBusy] = useState<'find' | 'cut' | null>(null);
  const [error, setError] = useState<string | null>(null);
  const text = draft.trim();
  async function submit(action: 'find' | 'cut') {
    if (!text || busy) return;
    setBusy(action);
    setError(null);
    try {
      if (action === 'find' && api.findMoments) {
        setMatches(await api.findMoments(run, text));
      } else if (action === 'cut' && api.reprompt) {
        onStarted(
          await api.reprompt(run, text, {
            ...(limits.minSeconds !== defaults.minSeconds ||
            limits.maxSeconds !== defaults.maxSeconds
              ? { minSeconds: limits.minSeconds, maxSeconds: limits.maxSeconds }
              : {}),
            ...(limits.count !== defaults.count ? { count: limits.count } : {}),
          }),
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(null);
    }
  }
  return (
    <section className="workspace-panel shrink-0 space-y-2 p-4" aria-label="Editorial brief">
      <label htmlFor="results-brief" className="text-xs font-semibold">
        Editorial brief
      </label>
      <textarea
        id="results-brief"
        rows={2}
        maxLength={1000}
        disabled={busy !== null}
        className="block w-full rounded-md border border-[var(--cm-glass-border)] bg-transparent p-2 text-sm"
        placeholder="Which moments should the edit include?"
        value={draft}
        onChange={(event) => {
          setDraft(event.target.value);
          setMatches(null);
          setError(null);
        }}
      />
      {draft.trim().length > 500 && api.findMoments && (
        <p className="text-xs text-[var(--cm-text-secondary)]">
          For moment search, shorten the brief to 500 characters.
        </p>
      )}
      {api.reprompt && (
        <details>
          <summary className="cursor-pointer text-xs text-[var(--cm-text-secondary)]">
            Clip length and count · {limits.minSeconds || '…'}–{limits.maxSeconds || '…'} s · up to{' '}
            {limits.count || '…'} clips
          </summary>
          <div className="my-2 flex flex-wrap gap-3 text-xs">
            {(
              [
                ['minSeconds', 'Minimum seconds', DURATION_BOUNDS],
                ['maxSeconds', 'Maximum seconds', DURATION_BOUNDS],
                ['count', 'Clip count', COUNT_BOUNDS],
              ] as const
            ).map(([key, label, bounds]) => (
              <label key={key}>
                {label}
                <Input
                  className="mt-1 w-24"
                  type="number"
                  min={bounds.min}
                  max={bounds.max}
                  step={1}
                  value={Number.isFinite(limits[key]) ? limits[key] : ''}
                  disabled={busy !== null}
                  onChange={(event) => setLimits({ ...limits, [key]: event.target.valueAsNumber })}
                />
              </label>
            ))}
          </div>
          {!valid && <p className="text-xs">Choose a valid length range and clip count.</p>}
        </details>
      )}
      <div className="flex flex-wrap items-center gap-2">
        {api.findMoments && (
          <Button
            size="sm"
            variant="outline"
            disabled={!text || busy !== null || text.length > 500}
            onClick={() => void submit('find')}
          >
            {busy === 'find' ? 'Searching…' : 'Find moments'}
          </Button>
        )}
        {api.reprompt && (
          <Button
            size="sm"
            disabled={!text || busy !== null || !valid}
            onClick={() => void submit('cut')}
          >
            {busy === 'cut' ? 'Starting…' : 'Make clips from brief'}
          </Button>
        )}
      </div>
      {matches !== null && (
        <div aria-live="polite" className="text-xs">
          {matches.length === 0 ? (
            <p>No matching moments in this recording.</p>
          ) : (
            <ol className="max-h-48 space-y-2 overflow-y-auto">
              {matches.map((moment) => (
                <li key={moment.units}>
                  <span className="mr-2 font-mono text-[var(--cm-text-muted)]">
                    {clock(moment.start * TICKS_PER_SECOND)}–{clock(moment.end * TICKS_PER_SECOND)}
                  </span>
                  {moment.label && <strong>{moment.label} · </strong>}
                  {moment.text}
                </li>
              ))}
            </ol>
          )}
        </div>
      )}
      {error && (
        <p role="alert" className="text-xs text-[var(--cm-text-secondary)]">
          {error}
        </p>
      )}
    </section>
  );
}
