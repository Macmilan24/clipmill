/**
 * Review preferences: what a decision does next, and the speed clips play
 * at. The Inspector changes both in place too; this is where they are set
 * for every clip that follows.
 */
import type { JSX } from 'react';

import { Switch } from '@/components/ui/switch';

import { SPEEDS } from '../inspector/playback.js';
import {
  setAutoAdvance,
  setReviewSpeed,
  useAutoAdvance,
  useReviewSpeed,
} from '../inspector/preferences.js';

export function ReviewPreferences(): JSX.Element {
  const advance = useAutoAdvance();
  const speed = useReviewSpeed();
  return (
    <div className="grid gap-5">
      <div className="preference-row">
        <div>
          <label htmlFor="review-advance-setting" className="text-sm font-medium">
            Next clip after deciding
          </label>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            Approving, holding or rejecting a clip opens the next one that has no decision yet.
          </p>
        </div>
        <Switch id="review-advance-setting" checked={advance} onCheckedChange={setAutoAdvance} />
      </div>

      <div className="preference-row">
        <div>
          <h3 className="text-sm font-medium">Clips play at</h3>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            The speed review starts every clip at. Changing it while reviewing keeps the new speed.
          </p>
        </div>
        <div className="review-segmented" role="group" aria-label="Review playback speed">
          {SPEEDS.map((value) => (
            <button
              key={value}
              type="button"
              aria-pressed={speed === value}
              onClick={() => setReviewSpeed(value)}
            >
              {`${value}×`}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
