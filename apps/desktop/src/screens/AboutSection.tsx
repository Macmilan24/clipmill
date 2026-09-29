/**
 * About ClipMill: which build this is, the licence it is shared under, and the
 * details a bug report needs, copied in one press.
 */
import { Check, Copy, Info } from 'lucide-react';
import { type JSX, useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Switch } from '@/components/ui/switch';

import { isTauri } from '../daemon/client.js';
import { setUpdatesEnabled, useUpdatesEnabled } from '../shell/updates.js';

export const SOURCE_URL = 'https://github.com/Macmilan24/clipmill';

export interface AboutProps {
  /** The engine's version, when it is connected. */
  readonly engineVersion: string | null;
  /** The app's version; read from the shell when not given. */
  readonly appVersion?: string | null;
}

/** The app's own version, from the shell that bundled it. */
function useAppVersion(given: string | null | undefined): string | null {
  const [version, setVersion] = useState<string | null>(given ?? null);
  useEffect(() => {
    if (given !== undefined || !isTauri()) return;
    let live = true;
    void import('@tauri-apps/api/app')
      .then(({ getVersion }) => getVersion())
      .then((found) => {
        if (live) setVersion(found);
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [given]);
  return version;
}

/** What a bug report needs, as plain text. */
export function debugDetails(appVersion: string | null, engineVersion: string | null): string {
  return [
    `ClipMill ${appVersion ?? 'development build'}`,
    `Engine ${engineVersion ?? 'not connected'}`,
    `Platform ${typeof navigator === 'undefined' ? 'unknown' : navigator.userAgent}`,
    `Reported ${new Date().toISOString()}`,
  ].join('\n');
}

export function AboutSection({ engineVersion, appVersion }: AboutProps): JSX.Element {
  const version = useAppVersion(appVersion);
  const notifyUpdates = useUpdatesEnabled();
  const [copied, setCopied] = useState<'copied' | 'failed' | null>(null);
  const details = debugDetails(version, engineVersion);

  return (
    <Card className="preference-section gap-0 overflow-hidden py-0">
      <div className="flex items-center gap-2.5 border-b border-[var(--cm-glass-border)] px-5 py-4">
        <Info className="size-4 text-[var(--cm-text-secondary)]" aria-hidden="true" />
        <div>
          <h2 className="text-sm font-semibold">About ClipMill</h2>
          <p className="mt-0.5 text-xs text-[var(--cm-text-secondary)]">
            Free software that finds and finishes clips on this computer.
          </p>
        </div>
      </div>
      <CardContent className="grid gap-5 px-5 py-5">
        <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-2 text-xs">
          <dt className="text-[var(--cm-text-secondary)]">Version</dt>
          <dd className="font-mono">{version ?? 'Development build'}</dd>
          <dt className="text-[var(--cm-text-secondary)]">Engine</dt>
          <dd className="font-mono">{engineVersion ?? 'Not connected'}</dd>
          <dt className="text-[var(--cm-text-secondary)]">Source code</dt>
          <dd className="font-mono break-all select-all">{SOURCE_URL}</dd>
        </dl>
        <p className="text-xs leading-relaxed text-[var(--cm-text-secondary)]">
          ClipMill is licensed under the GNU Affero General Public License, version 3. You may use,
          study, change and share it under those terms. Bundled tools and model weights keep their
          own licences.
        </p>
        <div className="flex items-center gap-3 rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-3.5 py-3">
          <div className="min-w-0 flex-1">
            <label htmlFor="notify-updates" className="text-xs font-semibold">
              Say when a new version is out
            </label>
            <p className="mt-1 text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
              Once a day, ClipMill asks GitHub which release is newest and shows a link to it. The
              question counts as a network operation in Local Lock. Nothing is downloaded or
              installed.
            </p>
          </div>
          <Switch id="notify-updates" checked={notifyUpdates} onCheckedChange={setUpdatesEnabled} />
        </div>
        <div className="flex flex-wrap items-center gap-3 rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-3.5 py-3">
          <div className="min-w-0 flex-1">
            <h3 className="text-xs font-semibold">Reporting a problem</h3>
            <p className="mt-1 text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
              Copy the version and platform details, then paste them into your report.
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              navigator.clipboard
                .writeText(details)
                .then(() => setCopied('copied'))
                .catch(() => setCopied('failed'));
            }}
          >
            {copied === 'copied' ? <Check aria-hidden="true" /> : <Copy aria-hidden="true" />}
            {copied === 'copied' ? 'Copied' : 'Copy details'}
          </Button>
          {copied === 'failed' && (
            <pre className="w-full overflow-x-auto rounded-md bg-[var(--cm-glass)] p-2 text-[11px] select-all">
              {details}
            </pre>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
