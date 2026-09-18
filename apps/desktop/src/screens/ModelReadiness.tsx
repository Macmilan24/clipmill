import { Check, ChevronDown, Cpu, Eye, MessageSquareText, RefreshCw, Workflow } from 'lucide-react';

import { useReadiness } from '../analysis/readiness.js';
import { StatusBadge } from '../components/StatusBadge.js';
import { Button } from '../components/ui/button.js';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card.js';
import { Spinner } from '../components/ui/spinner.js';
import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { StageReadiness } from '../daemon/client.js';

const STAGES: Readonly<Record<string, string>> = {
  'speech-vad': 'Detect speech',
  'speech-asr': 'Transcribe speech',
  'speech-align': 'Align word timing',
  'editorial-propose': 'Find complete moments',
  'editorial-review': 'Review meaning and context',
  'editorial-look': 'Check visual references',
  'shots-detect': 'Detect scene changes',
};

function roleOf(stages: readonly StageReadiness[]) {
  if (stages.some((stage) => stage.stage.startsWith('editorial-'))) {
    return {
      icon: <MessageSquareText className="size-5" />,
      role: 'Editorial intelligence',
      description:
        'Finds moments with a hook, context and payoff. Reviews their meaning before ranking.',
    };
  }
  if (stages.some((stage) => stage.stage.startsWith('speech-'))) {
    return {
      icon: <Workflow className="size-5" />,
      role: 'Speech & captions',
      description: stages.map((stage) => STAGES[stage.stage] ?? stage.stage).join(' · '),
    };
  }
  return {
    icon: <Eye className="size-5" />,
    role: 'Visual analysis',
    description: stages.map((stage) => STAGES[stage.stage] ?? stage.stage).join(' · '),
  };
}

/** Installation, worker presence and readiness remain separate, inspectable facts. */
export function ModelReadiness({ api = daemonApi }: { readonly api?: ShellApi }) {
  const { readiness, problem, refresh } = useReadiness(false, api);
  const stages =
    readiness?.stages.filter((stage) => stage.model && !stage.stage.endsWith('-cloud')) ?? [];
  const models = [...new Set(stages.map((stage) => stage.model))].map((name) => {
    const uses = stages.filter((stage) => stage.model === name);
    const role = roleOf(uses);
    return {
      name,
      uses,
      installed: uses.every((stage) => stage.modelPresent),
      ready: uses.every((stage) => stage.ready),
      remedies: [
        ...new Set(
          uses
            .filter((stage) => !stage.ready)
            .map((stage) => stage.remedy)
            .filter(Boolean),
        ),
      ],
      icon: role.icon,
      role: role.role,
      description: role.description,
    };
  });
  const installed = models.filter((model) => model.installed).length;
  return (
    <Card className="gap-0 overflow-hidden py-0">
      <CardHeader className="flex flex-row flex-wrap items-center justify-between gap-4 border-b border-[var(--cm-glass-border)] px-5 py-5">
        <div>
          <div className="mb-1.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-[var(--cm-text-muted)]">
            Analysis engine
          </div>
          <CardTitle className="text-base">Local models</CardTitle>
          <p className="mt-1 text-xs text-[var(--cm-text-secondary)]">
            {readiness
              ? `${installed} installed`
              : problem
                ? 'Status unavailable'
                : 'Checking installation…'}
            {readiness && (
              <span>
                {' '}
                · {readiness.workers.length} connected{' '}
                {readiness.workers.length === 1 ? 'worker' : 'workers'}
              </span>
            )}
          </p>
        </div>
        <div className="flex items-center gap-3">
          {readiness && !problem && (
            <StatusBadge tone={readiness.ready ? 'success' : 'warning'}>
              {readiness.ready ? 'Ready for analysis' : 'Setup needs attention'}
            </StatusBadge>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={refresh}
            aria-label="Refresh model readiness"
          >
            <RefreshCw />
            Refresh status
          </Button>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        {problem && (
          <p
            role="status"
            className="border-b border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-5 py-3 text-xs leading-relaxed text-[var(--cm-warning-ink)]"
          >
            Model readiness is unavailable. {problem}
            {readiness ? ' The details below are from the last successful check.' : ''}
          </p>
        )}
        {!readiness && !problem && (
          <div className="flex items-center gap-3 px-5 py-8 text-xs text-[var(--cm-text-secondary)]">
            <Spinner />
            Reading installed weights and connected workers…
          </div>
        )}
        <ul className="divide-y divide-[var(--cm-glass-border)]">
          {models.map((model) => (
            <li key={model.name} className="px-5 py-5">
              <div className="flex items-start gap-3.5">
                <div className="flex size-10 shrink-0 items-center justify-center rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] text-[var(--cm-text-secondary)]">
                  {model.icon}
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <div className="min-w-0">
                      <p className="mb-1 text-[10px] font-medium uppercase tracking-wider text-[var(--cm-text-muted)]">
                        {model.role}
                      </p>
                      <h3 className="break-words text-sm font-semibold leading-relaxed">
                        {model.name}
                      </h3>
                    </div>
                    <StatusBadge tone={problem ? 'neutral' : model.ready ? 'success' : 'warning'}>
                      {problem
                        ? 'Last known status'
                        : model.ready
                          ? 'Ready'
                          : model.installed
                            ? 'Needs attention'
                            : 'Not installed'}
                    </StatusBadge>
                  </div>
                  <p className="mt-1.5 max-w-[660px] text-xs leading-relaxed text-[var(--cm-text-secondary)]">
                    {model.description}
                  </p>
                  <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1.5 text-[11px] text-[var(--cm-text-secondary)]">
                    <span className="inline-flex items-center gap-1.5">
                      <Check
                        className={`size-3 ${model.installed ? 'text-[var(--cm-success-ink)]' : 'invisible'}`}
                      />
                      {model.installed ? 'Weights on device' : 'Weights missing'}
                    </span>
                    <span className="inline-flex items-center gap-1.5">
                      <Cpu className="size-3" />
                      {[...new Set(model.uses.map((stage) => stage.backend))].join(' / ') ||
                        'Backend not reported'}
                    </span>
                    <span>
                      {model.uses.length} {model.uses.length === 1 ? 'stage' : 'stages'}
                    </span>
                  </div>
                  {model.remedies.length > 0 && (
                    <div className="mt-3 rounded-lg border border-[color-mix(in_srgb,var(--color-warning)_30%,transparent)] bg-[var(--cm-recessed)] px-3 py-2.5">
                      <p className="text-[11px] font-semibold text-[var(--cm-warning-ink)]">
                        How to resolve
                      </p>
                      <ul className="mt-1 space-y-1.5 break-words text-xs leading-relaxed text-[var(--cm-text-secondary)]">
                        {model.remedies.map((remedy) => (
                          <li key={remedy}>{remedy}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                  <details className="group mt-3 text-xs">
                    <summary className="flex w-fit cursor-pointer list-none items-center gap-1.5 rounded text-[var(--cm-text-secondary)] transition-colors hover:text-[var(--cm-text-primary)] [&::-webkit-details-marker]:hidden">
                      <ChevronDown className="size-3 transition-transform group-open:rotate-180" />
                      Runtime details<span className="sr-only"> for {model.name}</span>
                    </summary>
                    <div className="mt-3 overflow-hidden rounded-lg border border-[var(--cm-glass-border)]">
                      {model.uses.map((stage) => (
                        <div
                          key={stage.stage}
                          className="border-b border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-3 py-3 last:border-b-0"
                        >
                          <div className="flex flex-wrap justify-between gap-2">
                            <span className="font-medium">
                              {STAGES[stage.stage] ?? stage.stage}
                            </span>
                            <span className="text-[11px] text-[var(--cm-text-secondary)]">
                              {stage.workerPresent ? 'Worker connected' : 'Worker disconnected'}
                            </span>
                          </div>
                          <p className="mt-1 break-all font-mono text-[10px] leading-relaxed text-[var(--cm-text-muted)]">
                            {stage.implementation} · {stage.capability}
                          </p>
                          {stage.missingFiles.length > 0 && (
                            <div className="mt-2">
                              <p className="text-[11px] font-medium text-[var(--cm-warning-ink)]">
                                Missing files
                              </p>
                              <ul className="mt-1 space-y-1 break-all font-mono text-[10px] text-[var(--cm-text-secondary)]">
                                {stage.missingFiles.map((path) => (
                                  <li key={path}>{path}</li>
                                ))}
                              </ul>
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  </details>
                </div>
              </div>
            </li>
          ))}
        </ul>
        {readiness && models.length === 0 && (
          <p className="px-5 py-6 text-xs text-[var(--cm-text-secondary)]">
            No local models are registered.
          </p>
        )}
        {readiness && (
          <div className="flex flex-wrap items-start justify-between gap-3 border-t border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-5 py-3">
            <p className="max-w-[640px] text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
              Model files are pinned by the engine. Installation and a connected worker are both
              required; readiness does not guarantee enough free memory for every job.
            </p>
            <StatusBadge tone={readiness.decoderPresent ? 'neutral' : 'warning'}>
              {readiness.decoderPresent ? 'Decoder installed' : 'Decoder missing'}
            </StatusBadge>
            {!readiness.decoderPresent && (
              <p className="w-full break-all text-xs text-[var(--cm-warning-ink)]">
                Run <code>just setup</code> to install the pinned decoder at{' '}
                {readiness.decoderPath || 'its configured location'}.
              </p>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
