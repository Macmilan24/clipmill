/**
 * Editing and export preferences: how time reads, what new projects and
 * exports start from, the caption styles kept on this machine, and the keys.
 *
 * Each is a choice New Project, the Editor or the Export screen also makes in
 * place; this is where it is made once, for every clip that follows.
 */
import { Bookmark, Compass, FolderOpen, Keyboard, Sparkles, Trash2 } from 'lucide-react';
import { type JSX, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

import { daemonApi } from '../daemon/api.js';
import { forgetStyle, savedStyles } from '../editor/captionStyles.js';
import {
  DEFAULT_PATTERN,
  type FormatChoice,
  HEIGHT_CHOICES,
  type HeightChoice,
  RATE_CHOICES,
  type RateChoice,
  forgetFolder,
  heightLabel,
  rateLabel,
  recallFolder,
  recallFormat,
  recallPattern,
  rememberFolder,
  rememberFormat,
  rememberPattern,
} from '../export/format.js';
import { CAPTION_LOOKS, setStartingLook, startingLook } from '../results/captionLook.js';
import { forgetOnboarding, openTour, openWelcome } from '../onboarding/state.js';
import { openShortcuts } from '../shell/ShortcutSheet.js';
import { type TimeFormat, setTimeFormat, useTimeFormat } from '../shell/timeFormat.js';

export function EditingPreferences(): JSX.Element {
  const format = useTimeFormat();
  const [exportFormat, setExportFormat] = useState<FormatChoice>(() => recallFormat());
  const [styles, setStyles] = useState(() => savedStyles());
  const [look, setLook] = useState(() => startingLook());
  const [folder, setFolder] = useState(() => recallFolder());
  const [pattern, setPattern] = useState(() => recallPattern());
  const changeExport = (next: FormatChoice) => {
    setExportFormat(next);
    rememberFormat(next);
  };
  const chooseFolder = () => {
    daemonApi
      .chooseExportFolder()
      .then((chosen) => {
        if (chosen) {
          rememberFolder(chosen);
          setFolder(chosen);
        }
      })
      .catch(() => {});
  };
  const keepPattern = () => {
    rememberPattern(pattern);
    setPattern(recallPattern());
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
          <h3 className="text-sm font-medium">Exports go to</h3>
          <p className="text-xs text-[var(--cm-text-secondary)] break-all">
            {folder === ''
              ? 'The first export asks for a folder; later ones start in the last one used.'
              : folder}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" size="sm" onClick={chooseFolder}>
            <FolderOpen className="size-4" aria-hidden="true" />
            Choose…
          </Button>
          {folder !== '' && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                forgetFolder();
                setFolder('');
              }}
            >
              Forget
            </Button>
          )}
        </div>
      </div>

      <div className="preference-row">
        <div>
          <label htmlFor="default-name-pattern" className="text-sm font-medium">
            Files are named
          </label>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            Use {'{index}'}, {'{clip}'}, {'{project}'}, {'{duration}'}, {'{date}'} or {'{address}'}.
            A plain name gets a clip number. Each export can still change it.
          </p>
        </div>
        <Input
          id="default-name-pattern"
          className="w-[210px] font-mono text-xs"
          value={pattern}
          placeholder={DEFAULT_PATTERN}
          onChange={(event) => setPattern(event.target.value)}
          onBlur={keepPattern}
          onKeyDown={(event) => {
            if (event.key === 'Enter') keepPattern();
          }}
        />
      </div>

      <div className="preference-row">
        <div>
          <h3 className="text-sm font-medium">New projects’ captions</h3>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            The look a new project’s clips start with. New Project can still choose another.
          </p>
        </div>
        <div className="review-segmented" role="group" aria-label="Caption look for new projects">
          {CAPTION_LOOKS.map((choice) => (
            <button
              key={choice.ref}
              type="button"
              aria-pressed={look === choice.ref}
              onClick={() => {
                setStartingLook(choice.ref);
                setLook(choice.ref);
              }}
            >
              {choice.label}
            </button>
          ))}
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

      <div className="preference-row">
        <div>
          <h3 className="text-sm font-medium">Getting started</h3>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            A tour of where everything is; the welcome, and the tips the Inspector and the Editor
            show the first time.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" size="sm" onClick={openTour}>
            <Compass className="size-4" aria-hidden="true" />
            Take the tour
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              forgetOnboarding();
              openWelcome();
            }}
          >
            <Sparkles className="size-4" aria-hidden="true" />
            Show them again
          </Button>
        </div>
      </div>
    </div>
  );
}
