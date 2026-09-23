import { ChevronDown, CircleAlert, Globe, PackagePlus, Search, ShieldCheck } from 'lucide-react';
import { type JSX, useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet';
import { Spinner } from '@/components/ui/spinner';

import type { ShellApi } from '../daemon/api.js';
import type { HubInspection, ModelLibrary } from '../daemon/models.js';
import { formatBytes } from '../deviceProfile.js';
import { reasonOf } from './useModelLibrary.js';

/** The jobs whose worker loads whatever model it is handed. */
const JOBS = [
  {
    capability: 'asr',
    title: 'Transcription',
    detail: 'A whisper.cpp model: one GGML .bin file.',
    example: 'ggerganov/whisper.cpp',
  },
  {
    capability: 'editorial',
    title: 'Editorial AI',
    detail: 'An MLX vision-language model folder. Runs on Apple silicon.',
    example: 'mlx-community/Qwen3.5-9B-4bit',
  },
] as const;

/** The registry's rule for a model name, checked here to say so early. */
export function validModelName(name: string): boolean {
  return (
    name.length > 0 &&
    name.length <= 64 &&
    /^[a-z0-9][a-z0-9.-]*$/.test(name) &&
    !name.includes('..') &&
    !/[-.]$/.test(name)
  );
}

/**
 * Pin a model from Hugging Face.
 *
 * Looking a repository up and downloading from it are network operations, so
 * both wait for a click. What is pinned is shown before it is: the commit the
 * name resolved to, the licence, and every file with its size.
 */
export function AddModelSheet({
  open,
  onOpenChange,
  api,
  onAdded,
}: {
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
  readonly api: ShellApi;
  readonly onAdded: (library: ModelLibrary) => void;
}): JSX.Element {
  const [capability, setCapability] = useState<string>('asr');
  const [repo, setRepo] = useState('');
  const [revision, setRevision] = useState('');
  const [inspection, setInspection] = useState<HubInspection | null>(null);
  const [weightsFile, setWeightsFile] = useState('');
  const [name, setName] = useState('');
  const [title, setTitle] = useState('');
  const [busy, setBusy] = useState<'inspect' | 'add' | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) {
      setInspection(null);
      setError(null);
      setBusy(null);
    }
  }, [open]);

  const job = JOBS.find((candidate) => candidate.capability === capability) ?? JOBS[0];
  const chosenWeights = inspection?.weightChoices.find((file) => file.path === weightsFile);
  const pinned =
    inspection === null
      ? []
      : capability === 'asr'
        ? chosenWeights
          ? [chosenWeights]
          : []
        : inspection.files;
  const bytes = pinned.reduce((total, file) => total + file.bytes, 0);
  const canAdd =
    inspection !== null &&
    inspection.problem === '' &&
    pinned.length > 0 &&
    validModelName(name) &&
    busy === null;

  const lookUp = async () => {
    setBusy('inspect');
    setError(null);
    setInspection(null);
    try {
      const answer = await api.inspectHubModel(repo.trim(), revision.trim(), capability);
      setInspection(answer);
      const first = answer.weightChoices[0];
      setWeightsFile(first?.path ?? '');
      setName(first?.suggestedName ?? answer.suggestedName);
      setTitle(first?.suggestedTitle ?? answer.suggestedTitle);
    } catch (cause) {
      setError(reasonOf(cause));
    } finally {
      setBusy(null);
    }
  };

  const add = async () => {
    if (inspection === null) return;
    setBusy('add');
    setError(null);
    try {
      const library = await api.addCustomModel({
        repo: inspection.repo,
        commit: inspection.commit,
        capability,
        weightsFile: capability === 'asr' ? weightsFile : '',
        name: name.trim(),
        title: title.trim(),
      });
      onAdded(library);
      onOpenChange(false);
    } catch (cause) {
      setError(reasonOf(cause));
    } finally {
      setBusy(null);
    }
  };

  return (
    <Sheet
      open={open}
      onOpenChange={(value) => {
        if (busy === null) onOpenChange(value);
      }}
    >
      <SheetContent
        className="w-full gap-0 overflow-hidden bg-[var(--cm-glass)] sm:max-w-[560px]"
        showCloseButton={busy === null}
      >
        <SheetHeader className="shrink-0 border-b border-[var(--cm-glass-border)] p-5 pr-10">
          <SheetTitle className="flex items-center gap-2">
            <PackagePlus className="size-4" />
            Add a model from Hugging Face
          </SheetTitle>
          <SheetDescription className="text-xs leading-relaxed">
            ClipMill pins the exact commit and every file&apos;s SHA-256, so the model you add is
            the model that runs.
          </SheetDescription>
        </SheetHeader>
        <div className="min-h-0 flex-1 space-y-5 overflow-y-auto p-5">
          <fieldset className="space-y-2">
            <legend className="mb-2 text-xs font-medium">Job</legend>
            <RadioGroup
              value={capability}
              onValueChange={(value) => {
                setCapability(value);
                setInspection(null);
              }}
              className="gap-2"
            >
              {JOBS.map((candidate) => (
                <label key={candidate.capability} className="add-model-choice">
                  <RadioGroupItem value={candidate.capability} aria-label={candidate.title} />
                  <span className="min-w-0">
                    <span className="block text-xs font-medium">{candidate.title}</span>
                    <span className="block text-[11px] text-[var(--cm-text-secondary)]">
                      {candidate.detail}
                    </span>
                  </span>
                </label>
              ))}
            </RadioGroup>
          </fieldset>

          <form
            className="space-y-3"
            onSubmit={(event) => {
              event.preventDefault();
              if (repo.trim() !== '' && busy === null) void lookUp();
            }}
          >
            <div>
              <Label htmlFor="hub-repo" className="mb-1 block text-xs">
                Repository
              </Label>
              <Input
                id="hub-repo"
                value={repo}
                spellCheck={false}
                autoComplete="off"
                placeholder={job.example}
                onChange={(event) => setRepo(event.target.value)}
              />
            </div>
            <div>
              <Label htmlFor="hub-revision" className="mb-1 block text-xs">
                Branch, tag or commit{' '}
                <span className="text-[var(--cm-text-muted)]">(optional)</span>
              </Label>
              <Input
                id="hub-revision"
                value={revision}
                spellCheck={false}
                autoComplete="off"
                placeholder="main"
                onChange={(event) => setRevision(event.target.value)}
              />
            </div>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <p className="model-network-note">
                <Globe aria-hidden="true" />
                Looking up and downloading contact huggingface.co.
              </p>
              <Button
                type="submit"
                size="sm"
                variant="outline"
                disabled={repo.trim() === '' || busy !== null}
              >
                {busy === 'inspect' ? <Spinner /> : <Search />}
                Look up
              </Button>
            </div>
          </form>

          {error !== null && (
            <p className="add-model-problem" role="alert">
              <CircleAlert aria-hidden="true" />
              {error}
            </p>
          )}

          {inspection !== null && (
            <section className="space-y-4" aria-label="What would be pinned">
              <dl className="add-model-facts">
                <div>
                  <dt>Commit</dt>
                  <dd className="mono">
                    {inspection.commit ? inspection.commit.slice(0, 12) : '—'}
                  </dd>
                </div>
                <div>
                  <dt>Licence</dt>
                  <dd>
                    {inspection.licenseAllowed ? (
                      <span className="inline-flex items-center gap-1 text-[var(--cm-success-ink)]">
                        <ShieldCheck className="size-3.5" aria-hidden="true" />
                        {inspection.licenseSpdx}
                      </span>
                    ) : (
                      inspection.license || 'Not stated'
                    )}
                  </dd>
                </div>
                {pinned.length > 0 && (
                  <div>
                    <dt>Download</dt>
                    <dd className="mono">{formatBytes(bytes)}</dd>
                  </div>
                )}
              </dl>

              {inspection.problem !== '' && (
                <p className="add-model-problem" role="alert">
                  <CircleAlert aria-hidden="true" />
                  {inspection.problem}
                </p>
              )}

              {inspection.problem === '' && capability === 'asr' && (
                <fieldset>
                  <legend className="mb-2 text-xs font-medium">Weights file</legend>
                  <RadioGroup
                    value={weightsFile}
                    onValueChange={(value) => {
                      setWeightsFile(value);
                      const file = inspection.weightChoices.find((item) => item.path === value);
                      if (file) {
                        setName(file.suggestedName);
                        setTitle(file.suggestedTitle);
                      }
                    }}
                    className="add-model-files"
                  >
                    {inspection.weightChoices.map((file) => (
                      <label key={file.path} className="add-model-choice">
                        <RadioGroupItem value={file.path} aria-label={file.path} />
                        <span className="mono min-w-0 flex-1 truncate text-[11px]">
                          {file.path}
                        </span>
                        <span className="mono shrink-0 text-[11px] text-[var(--cm-text-secondary)]">
                          {formatBytes(file.bytes)}
                        </span>
                      </label>
                    ))}
                  </RadioGroup>
                </fieldset>
              )}

              {inspection.problem === '' && capability === 'editorial' && (
                <>
                  <details className="preference-disclosure">
                    <summary>
                      <ChevronDown className="size-3.5" />
                      {inspection.files.length} files pinned together
                    </summary>
                    <ul className="add-model-file-list">
                      {inspection.files.map((file) => (
                        <li key={file.path}>
                          <span className="mono min-w-0 flex-1 truncate">{file.path}</span>
                          <span className="mono shrink-0">{formatBytes(file.bytes)}</span>
                        </li>
                      ))}
                    </ul>
                  </details>
                  <p className="add-model-caution">
                    ClipMill&apos;s editorial prompts are tuned for Qwen3.5 9B. Another model may
                    not follow the editorial format; an analysis says so if it fails. Choose it for
                    Editorial AI once it has downloaded.
                  </p>
                </>
              )}

              {inspection.problem === '' && (
                <div className="grid gap-3 sm:grid-cols-2">
                  <div>
                    <Label htmlFor="model-name" className="mb-1 block text-xs">
                      Name in ClipMill
                    </Label>
                    <Input
                      id="model-name"
                      value={name}
                      spellCheck={false}
                      aria-invalid={!validModelName(name)}
                      onChange={(event) => setName(event.target.value)}
                    />
                    {!validModelName(name) && (
                      <p className="mt-1 text-[11px] text-[var(--cm-danger-ink)]">
                        Lowercase letters, digits, hyphens and dots.
                      </p>
                    )}
                  </div>
                  <div>
                    <Label htmlFor="model-title" className="mb-1 block text-xs">
                      Title
                    </Label>
                    <Input
                      id="model-title"
                      value={title}
                      onChange={(event) => setTitle(event.target.value)}
                    />
                  </div>
                </div>
              )}
            </section>
          )}
        </div>
        <div className="flex shrink-0 flex-wrap items-center justify-end gap-2 border-t border-[var(--cm-glass-border)] p-4">
          <Button variant="ghost" disabled={busy !== null} onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button disabled={!canAdd} onClick={() => void add()}>
            {busy === 'add' ? <Spinner /> : <PackagePlus />}
            {pinned.length > 0 ? `Add and download · ${formatBytes(bytes)}` : 'Add and download'}
          </Button>
        </div>
      </SheetContent>
    </Sheet>
  );
}
