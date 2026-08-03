/**
 * 13 Settings, Phase 1 subset: where things are, how long they stay, and
 * whether this installation is actually offline.
 *
 * Everything here is read. Not because settings are hard, but because a control
 * that changes nothing is worse than no control: retention is displayed because
 * the number is real and the collection policy that would let a user move it is
 * not written, and the delivery profile is stated on the Export screen for the
 * same reason.
 *
 * The Local Lock card is the one worth reading twice. It does not say "on"
 * because somebody typed true — it says how many stages the daemon will run,
 * how many of those may reach the network, and how many times a task declaring
 * network access has started since the daemon did. A claim with no way to come
 * out false is not evidence, so the card shows the counts that could.
 */
import { HardDrive, Lock, ShieldOff, TriangleAlert } from 'lucide-react';
import type { JSX } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { Spinner } from '@/components/ui/spinner';

import type { LocalLock, StorageStats } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';

/** The wording on screen is this file's; the daemon sends the key. */
const CATEGORY_LABELS: Readonly<Record<string, string>> = {
  artifacts: 'Artifacts',
  models: 'Model weights',
  state: 'Project state',
};

/** What each category means for a user deciding whether to touch it. */
const CATEGORY_NOTES: Readonly<Record<string, string>> = {
  artifacts: 'Derived from your recordings. Re-derivable, and safe to collect.',
  models: 'Downloaded once. Expensive to fetch again, and pinned by digest.',
  state: 'Your projects, edits, and decisions. Small, and not regenerable.',
};

export interface SettingsProps {
  readonly storage: StorageStats | null;
  readonly lock: LocalLock | null;
  readonly loading: boolean;
  readonly error: string | null;
}

export function Settings({ storage, lock, loading, error }: SettingsProps): JSX.Element {
  return (
    <div className="flex h-full flex-col gap-4 overflow-y-auto p-4">
      {error !== null && (
        <Alert variant="destructive">
          <TriangleAlert />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-sm">
            <HardDrive className="size-4" /> Storage
          </CardTitle>
        </CardHeader>
        <CardContent>
          {loading && storage === null ? (
            <p className="flex items-center gap-2 text-xs text-[var(--cm-ink-2)]">
              <Spinner className="size-3" /> Measuring…
            </p>
          ) : storage === null ? (
            <p className="text-xs text-[var(--cm-ink-2)]">This daemon measures no storage.</p>
          ) : (
            <>
              <ul className="space-y-3">
                {storage.categories.map((category) => (
                  <li key={category.key}>
                    <div className="flex items-baseline justify-between gap-4">
                      <span className="text-sm text-[var(--cm-ink-1)]">
                        {CATEGORY_LABELS[category.key] ?? category.key}
                      </span>
                      <span className="font-mono text-sm text-[var(--cm-ink-1)]">
                        {formatBytes(category.bytes)}
                      </span>
                    </div>
                    <p className="text-xs text-[var(--cm-ink-3)]">
                      {category.items} items · {CATEGORY_NOTES[category.key] ?? ''}
                    </p>
                    <p className="truncate font-mono text-[11px] text-[var(--cm-ink-3)]">
                      {category.path}
                    </p>
                  </li>
                ))}
              </ul>
              <Separator className="my-3" />
              <dl className="space-y-1 text-xs">
                <Row
                  label="Free on this volume"
                  value={
                    storage.availableBytes === undefined
                      ? 'not readable'
                      : formatBytes(storage.availableBytes)
                  }
                />
                <Row label="Retention" value={describeGrace(storage.retentionGraceSeconds)} />
              </dl>
              <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
                Retention is how long an artifact nothing refers to is kept before collection may
                take it. Shown rather than adjustable: the number is real, and a control that moved
                it would need a collection policy Phase 1 has not written.
              </p>
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-sm">
            {lock?.engaged === false ? (
              <ShieldOff className="size-4 text-[var(--cm-warning-ink)]" />
            ) : (
              <Lock className="size-4" />
            )}
            Local Lock
          </CardTitle>
        </CardHeader>
        <CardContent>
          {lock === null ? (
            <p className="text-xs text-[var(--cm-ink-2)]">
              {loading ? 'Asking…' : 'This daemon reports no policy.'}
            </p>
          ) : (
            <>
              <div className="mb-3 flex items-center gap-2">
                <Badge variant={lock.engaged ? 'default' : 'destructive'}>
                  {lock.engaged ? 'Engaged' : 'Not engaged'}
                </Badge>
                <span className="text-xs text-[var(--cm-ink-2)]">
                  {lock.engaged
                    ? 'Nothing this daemon runs may reach the network.'
                    : 'Something here can reach the network, or already has.'}
                </span>
              </div>
              <dl className="space-y-1 text-xs">
                <Row label="Stages registered" value={String(lock.stages)} />
                <Row
                  label="Stages allowed to use the network"
                  value={String(lock.networkAllowedStages)}
                />
                <Row label="Egress attempts this run" value={String(lock.egressAttempts)} />
              </dl>
              <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
                These are counts, not a switch. The first two come from the table of every stage the
                daemon will run, so a stage added with network access turns this card red without
                anyone remembering to change it; the third counts what has actually started. A claim
                with no way to come out false would not be worth showing.
              </p>
            </>
          )}
        </CardContent>
      </Card>

      <p className="text-xs text-[var(--cm-ink-3)]">
        <Badge variant="outline">Phase 2</Badge> Moving the storage location, editing the retention
        window, and per-project privacy rules are not built. What is here is what can be told
        truthfully today.
      </p>
    </div>
  );
}

function Row({ label, value }: { readonly label: string; readonly value: string }): JSX.Element {
  return (
    <div className="flex justify-between gap-4">
      <dt className="text-[var(--cm-ink-2)]">{label}</dt>
      <dd className="font-mono text-[var(--cm-ink-1)]">{value}</dd>
    </div>
  );
}

/** Seconds as something a person reads, without a date library. */
function describeGrace(seconds: number): string {
  if (seconds <= 0) {
    return 'collected immediately';
  }
  const days = Math.floor(seconds / 86_400);
  if (days >= 1) {
    return `${days} day${days === 1 ? '' : 's'}`;
  }
  const hours = Math.floor(seconds / 3_600);
  if (hours >= 1) {
    return `${hours} hour${hours === 1 ? '' : 's'}`;
  }
  const minutes = Math.max(1, Math.floor(seconds / 60));
  return `${minutes} minute${minutes === 1 ? '' : 's'}`;
}
