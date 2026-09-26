/**
 * The Audio tab's music and voice: a sound under the clip that drops when
 * anyone speaks, and how much the voice is cleaned.
 */
import { Music, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import type { EditIr } from '@clipmill/contracts';

import { Button } from '../components/ui/button.js';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select.js';
import type { Asset, AssetLicense, EditCommandJson } from '../daemon/client.js';
import { clockTenths } from '../inspector/review.js';
import { type AssetAccess, LICENSES, withAsset } from './brand.js';
import { CommitSlider, Field } from './controls.js';
import { MUSIC_DUCKS, MUSIC_LEVELS, setCleanup, setMusic } from './music.js';

const decibels = (value: number) => `${value > 0 ? '+' : ''}${value} dB`;

export function MusicSection({
  document,
  busy,
  onApply,
  assets,
}: {
  readonly document: EditIr | null;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly assets: AssetAccess | null;
}) {
  const music = document?.audio.music;
  const cleanup = document?.audio.cleanup;
  const [sounds, setSounds] = useState<readonly Asset[]>([]);
  const [license, setLicense] = useState<AssetLicense>('royalty_free');
  const [bringing, setBringing] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  useEffect(() => {
    if (!assets) return undefined;
    let current = true;
    void assets
      .list('audio')
      .then((found) => {
        if (current) setSounds(found);
      })
      .catch(() => {});
    return () => {
      current = false;
    };
  }, [assets]);
  const choose = (sound: Asset) =>
    onApply(
      setMusic(
        music
          ? { ...music, asset: sound.hash }
          : {
              asset: sound.hash,
              level_db: MUSIC_LEVELS.default,
              duck_db: MUSIC_DUCKS.default,
            },
        withAsset(document?.assets, sound),
      ),
    );
  const bring = async () => {
    if (!assets) return;
    setBringing(true);
    setProblem(null);
    try {
      const sound = await assets.bring('audio', license);
      if (sound) {
        setSounds((current) => [sound, ...current.filter((item) => item.hash !== sound.hash)]);
        choose(sound);
      }
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
    } finally {
      setBringing(false);
    }
  };
  const current = music ? sounds.find((sound) => sound.hash === music.asset) : undefined;

  return (
    <>
      <section className="review-section">
        <h3 className="review-section-title">Music</h3>
        {!assets && <p className="review-footnote">Music can be added in the ClipMill app.</p>}
        {assets && (
          <>
            {sounds.length > 0 && (
              <ul className="edit-sounds" aria-label="Your sounds">
                {sounds.slice(0, 6).map((sound) => (
                  <li key={sound.hash}>
                    <button
                      type="button"
                      aria-pressed={music?.asset === sound.hash}
                      disabled={busy || !document}
                      onClick={() => choose(sound)}
                    >
                      <Music className="size-3.5" aria-hidden="true" />
                      <span className="edit-sound-name">{sound.name}</span>
                      <span className="edit-sound-length mono">
                        {clockTenths(sound.durationTicks)}
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
            <div className="flex flex-wrap items-center gap-2">
              <Select value={license} onValueChange={(value) => setLicense(value as AssetLicense)}>
                <SelectTrigger aria-label="Whose music it is" className="h-8 w-[190px] text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {LICENSES.map((item) => (
                    <SelectItem key={item.value} value={item.value}>
                      {item.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button
                size="sm"
                variant="outline"
                disabled={busy || bringing || !document}
                onClick={() => void bring()}
              >
                <Music className="size-4" aria-hidden="true" />
                {bringing ? 'Bringing it in…' : 'Choose music'}
              </Button>
            </div>
            {problem && <p className="review-footnote text-[var(--cm-warning-ink)]">{problem}</p>}
          </>
        )}
        {music && (
          <>
            <Field label="Level">
              <CommitSlider
                label="Music level"
                min={MUSIC_LEVELS.min}
                max={MUSIC_LEVELS.max}
                value={Math.round(music.level_db)}
                disabled={busy}
                format={decibels}
                onCommit={(level_db) => onApply(setMusic({ ...music, level_db }))}
              />
            </Field>
            <Field label="Under speech">
              <CommitSlider
                label="How far the music drops under speech"
                min={MUSIC_DUCKS.min}
                max={MUSIC_DUCKS.max}
                value={Math.round(music.duck_db)}
                disabled={busy}
                format={decibels}
                onCommit={(duck_db) => onApply(setMusic({ ...music, duck_db }))}
              />
            </Field>
            {current && current.durationTicks > 0 && (
              <Field label="Starts at">
                <CommitSlider
                  label="Where in the music the clip starts"
                  min={0}
                  max={Math.max(0, Math.floor(current.durationTicks / 90_000) - 1)}
                  value={Math.round((music.offset_ticks ?? 0) / 90_000)}
                  disabled={busy}
                  format={(seconds) => clockTenths(seconds * 90_000)}
                  onCommit={(seconds) => {
                    const { offset_ticks: _offset, ...rest } = music;
                    onApply(
                      setMusic(seconds > 0 ? { ...rest, offset_ticks: seconds * 90_000 } : rest),
                    );
                  }}
                />
              </Field>
            )}
            <p className="review-footnote">
              It fades in and out, drops wherever someone speaks, and loops if it is shorter than
              the clip.
            </p>
            <Button
              size="sm"
              variant="ghost"
              className="self-start"
              disabled={busy}
              onClick={() => onApply(setMusic(null))}
            >
              <Trash2 className="size-4" aria-hidden="true" />
              Remove the music{current ? ` (${current.name})` : ''}
            </Button>
          </>
        )}
      </section>

      <section className="review-section">
        <h3 className="review-section-title">Voice</h3>
        <Field label="Clean up">
          <div className="review-segmented" role="group" aria-label="Voice cleanup">
            {(
              [
                [null, 'Off'],
                ['light', 'Light'],
                ['strong', 'Strong'],
              ] as const
            ).map(([value, label]) => (
              <button
                key={label}
                type="button"
                aria-pressed={(cleanup ?? null) === value}
                disabled={busy || !document}
                onClick={() => onApply(setCleanup(value))}
              >
                {label}
              </button>
            ))}
          </div>
        </Field>
        <p className="review-footnote">
          Light cuts rumble and a little hiss and evens the level, for a quiet room; strong takes
          out more noise and softens hard esses, for a noisy one. Heard in the rendered preview and
          the export.
        </p>
      </section>
    </>
  );
}
