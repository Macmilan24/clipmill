import { RefreshCw } from 'lucide-react';

import { useReadiness } from '../analysis/readiness.js';
import { Button } from '../components/ui/button.js';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card.js';
import { type ShellApi, daemonApi } from '../daemon/api.js';

/** Model installation and runtime readiness are separate facts. */
export function ModelReadiness({ api = daemonApi }: { readonly api?: ShellApi }) {
  const { readiness, problem, refresh } = useReadiness(false, api);
  const stages =
    readiness?.stages.filter((stage) => stage.model && !stage.stage.endsWith('-cloud')) ?? [];
  const models = [...new Set(stages.map((stage) => stage.model))].map((name) => {
    const uses = stages.filter((stage) => stage.model === name);
    return {
      name,
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
    };
  });
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div className="space-y-1.5">
          <CardTitle className="text-section-title">Local models</CardTitle>
          <p className="text-[11px] text-[var(--cm-text-secondary)]">
            {readiness
              ? `${models.filter((model) => model.installed).length} installed`
              : problem
                ? 'Status unavailable'
                : 'Checking installation…'}
          </p>
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={refresh}
          aria-label="Refresh model readiness"
        >
          <RefreshCw />
        </Button>
      </CardHeader>
      <CardContent>
        {problem && (
          <p role="status" className="text-xs text-[var(--cm-text-secondary)]">
            Model readiness is unavailable. {problem}
          </p>
        )}
        <ul className="divide-y divide-[var(--cm-glass-border)]">
          {models.map((model) => (
            <li key={model.name} className="py-3 first:pt-0 last:pb-0">
              <div className="flex items-start justify-between gap-3">
                <span className="min-w-0 break-words text-[12px] font-medium">{model.name}</span>
                <span
                  className={`shrink-0 text-[11px] ${model.ready ? 'text-[var(--cm-success-ink)]' : 'text-[var(--cm-warning-ink)]'}`}
                >
                  {model.ready ? 'Ready' : model.installed ? 'Needs attention' : 'Not installed'}
                </span>
              </div>
              {model.remedies.length > 0 && (
                <details className="mt-2 text-[11px] text-[var(--cm-text-secondary)]">
                  <summary className="cursor-pointer">How to resolve</summary>
                  <ul className="mt-2 space-y-2 break-words">
                    {model.remedies.map((remedy) => (
                      <li key={remedy}>{remedy}</li>
                    ))}
                  </ul>
                </details>
              )}
            </li>
          ))}
        </ul>
        {readiness && models.length === 0 && (
          <p className="text-xs text-[var(--cm-text-secondary)]">No local models are registered.</p>
        )}
      </CardContent>
    </Card>
  );
}
