/**
 * Editing and export preferences: how time reads, what an export starts from,
 * the caption styles kept on this machine, and the keys.
 *
 * Each is a choice the Editor or the Export screen also makes in place; this
 * is where it is made once, for every clip that follows.
 */
import { Bookmark, Keyboard, Trash2 } from 'lucide-react';
import { type JSX, useState } from 'react';

import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

import { forgetStyle, savedStyles } from '../editor/captionStyles.js';
import {
  type FormatChoice,
  HEIGHT_CHOICES,
  type HeightChoice,
  RATE_CHOICES,
  type RateChoice,
  heightLabel,
  rateLabel,
  recallFormat,
  rememberFormat,
} from '../export/format.js';
import { openShortcuts } from '../shell/ShortcutSheet.js';
import { type TimeFormat, setTimeFormat, useTimeFormat } from '../shell/timeFormat.js';

export function EditingPreferences(): JSX.Element {
  const format = useTimeFormat();
  const [exportFormat, setExportFormat] = useState<FormatChoice>(() => recallFormat());
  const [styles, setStyles] = useState(() => savedStyles());
  const changeExport = (next: FormatChoice) => {
    setExportFormat(next);
    rememberFormat(next);
  };
  return (
    <div className="grid gap-5">
      <div className="preference-row">
        <div>
          <h3 className="text-sm font-medium">Time</h3>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            How positions and lengths read in the Editor and in review.
          </p>
        </div>
        <div className="review-segmented" role="group" aria-label="Time format">
          {(
            [
              ['clock', '0:12.4'],
              ['frames', '0:12:10 frames'],
            ] as const satisfies readonly (readonly [TimeFormat, string])[]
          ).map(([value, label]) => (
            <button
              key={value}
              type="button"
              aria-pressed={format === value}
              onClick={() => setTimeFormat(value)}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      <div className="preference-row">
        <div>
          <h3 className="text-sm font-medium">Exports start at</h3>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            The frame rate and size a new export is set to. Each export can still change them.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Select
            value={exportFormat.rate}
            onValueChange={(rate) => changeExport({ ...exportFormat, rate: rate as RateChoice })}
          >
            <SelectTrigger aria-label="Default frame rate" className="w-[210px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {RATE_CHOICES.map((rate) => (
                <SelectItem key={rate} value={rate}>
                  {rateLabel(rate, null)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select
            value={String(exportFormat.height)}
            onValueChange={(height) =>
              changeExport({ ...exportFormat, height: Number(height) as HeightChoice })
            }
          >
            <SelectTrigger aria-label="Default resolution" className="w-[210px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {HEIGHT_CHOICES.map((height) => (
                <SelectItem key={height} value={String(height)}>
                  {heightLabel(height)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="preference-row">
        <div>
          <h3 className="text-sm font-medium">Caption styles</h3>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            Styles saved from the Editor’s Captions tab, ready for any clip.
          </p>
        </div>
        {styles.length === 0 ? (
          <p className="text-xs text-[var(--cm-text-muted)]">
            None yet. Save one from a clip with “Save this style”.
          </p>
        ) : (
          <ul className="grid gap-1.5">
            {styles.map((style) => (
              <li key={style.name} className="flex items-center justify-between gap-3 text-sm">
                <span className="flex min-w-0 items-center gap-2">
                  <Bookmark className="size-3.5 shrink-0" aria-hidden="true" />
                  <span className="truncate">{style.name}</span>
                </span>
                <Button
                  size="xs"
                  variant="ghost"
                  aria-label={`Forget ${style.name}`}
                  onClick={() => setStyles(forgetStyle(style.name))}
                >
                  <Trash2 className="size-3.5" aria-hidden="true" />
                </Button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="preference-row">
        <div>
          <h3 className="text-sm font-medium">Keyboard</h3>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            Every key the Editor and clip review answer.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={openShortcuts}>
          <Keyboard className="size-4" aria-hidden="true" />
          Keyboard shortcuts
        </Button>
      </div>
    </div>
  );
}
