/**
 * Settings → Connected agents: give an agent such as Claude Desktop its own
 * token, choose what it may do, see what it did, and take the token back.
 * The token is shown once; ClipMill keeps only its hash.
 */
import { Bot, Check, Copy, History, Unplug } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';

import '../screens/preferences.css';
import { Button } from '../components/ui/button.js';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card.js';
import { Input } from '../components/ui/input.js';
import {
  AGENT_SCOPES,
  type AgentCreated,
  type AgentScope,
  type Agents,
  type AgentsApi,
  type EngineAuditEntry,
  agentsApi,
  mcpConfig,
} from '../daemon/agents.js';

const AUDIT_SHOWN = 50;

function when(millis: number): string {
  if (!millis) return 'never';
  return new Date(millis).toLocaleString(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  });
}

function CopyButton({ text, label }: { readonly text: string; readonly label: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <Button
      variant="outline"
      size="sm"
      aria-label={label}
      onClick={() => {
        void navigator.clipboard
          .writeText(text)
          .then(() => setCopied(true))
          .catch(() => setCopied(false));
      }}
    >
      {copied ? <Check aria-hidden /> : <Copy aria-hidden />}
      {copied ? 'Copied' : 'Copy'}
    </Button>
  );
}

/** The token and the configuration to paste, shown once after connecting. */
function Created({ created }: { readonly created: AgentCreated }) {
  const config = created.cliPath ? mcpConfig(created.cliPath, created.token) : null;
  return (
    <div
      role="status"
      className="space-y-3 rounded-[var(--cm-radius-card)] border border-[var(--cm-glass-border)] p-4"
    >
      <p className="text-xs font-medium">
        {created.client?.name ?? 'The agent'} is connected. Copy its token now: ClipMill keeps only
        a hash of it, so it cannot be shown again.
      </p>
      <div className="flex items-center gap-2">
        <code className="min-w-0 flex-1 truncate rounded bg-[var(--cm-recessed)] px-2 py-1 font-mono text-[11px]">
          {created.token}
        </code>
        <CopyButton text={created.token} label="Copy the token" />
      </div>
      {config ? (
        <div className="space-y-2">
          <p className="text-[11px] text-[var(--cm-text-secondary)]">
            For Claude Desktop and other MCP clients, add this to the client's MCP servers
            configuration, then restart it:
          </p>
          <pre className="max-h-48 overflow-auto rounded bg-[var(--cm-recessed)] p-2 font-mono text-[10px]">
            {config}
          </pre>
          <CopyButton text={config} label="Copy the MCP configuration" />
        </div>
      ) : (
        <p className="text-[11px] text-[var(--cm-text-secondary)]">
          Run <code>clipmill mcp</code> with this token in <code>CLIPMILL_TOKEN</code>.
        </p>
      )}
    </div>
  );
}

export function ConnectedAgents({ api = agentsApi }: { readonly api?: AgentsApi }) {
  const [agents, setAgents] = useState<Agents | null>(null);
  const [audit, setAudit] = useState<readonly EngineAuditEntry[]>([]);
  const [name, setName] = useState('Claude Desktop');
  const [scopes, setScopes] = useState<readonly AgentScope[]>(['read', 'edit']);
  const [created, setCreated] = useState<AgentCreated | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const live = useRef(true);

  const refresh = useCallback(async () => {
    try {
      const [listed, log] = await Promise.all([api.listAgents(), api.engineAudit(AUDIT_SHOWN)]);
      if (!live.current) return;
      setAgents(listed);
      setAudit(log);
      setError(null);
    } catch (cause) {
      if (live.current) setError(cause instanceof Error ? cause.message : String(cause));
    }
  }, [api]);

  useEffect(() => {
    live.current = true;
    void refresh();
    return () => {
      live.current = false;
    };
  }, [refresh]);

  const act = async (action: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await action();
    } catch (cause) {
      if (live.current) setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (live.current) setBusy(false);
      void refresh();
    }
  };

  const active = agents?.clients.filter((client) => client.revokedUnixMillis === 0) ?? [];

  return (
    <Card className="preference-section gap-0 overflow-hidden py-0">
      <CardHeader className="border-b border-[var(--cm-glass-border)] px-5 py-4">
        <CardTitle className="flex items-center gap-2 text-sm">
          <Bot className="size-4" aria-hidden />
          Connected agents
        </CardTitle>
        <p className="text-xs text-[var(--cm-text-secondary)]">
          Let an AI agent such as Claude Desktop use ClipMill on this computer, through its own
          token and only for what you allow. Everything it does is listed below.
        </p>
      </CardHeader>
      <CardContent className="space-y-5 px-5 py-4">
        <form
          className="space-y-3"
          onSubmit={(event) => {
            event.preventDefault();
            void act(async () => {
              const made = await api.createAgent(name.trim(), scopes);
              if (live.current) setCreated(made);
            });
          }}
        >
          <label className="block space-y-1 text-xs">
            <span className="font-medium">Agent name</span>
            <Input
              value={name}
              maxLength={80}
              onChange={(event) => setName(event.target.value)}
              aria-label="Agent name"
            />
          </label>
          <fieldset className="space-y-2">
            <legend className="text-xs font-medium">It may</legend>
            {AGENT_SCOPES.map(({ scope, label, detail }) => (
              <label key={scope} className="flex items-start gap-2 text-xs">
                <input
                  type="checkbox"
                  className="mt-0.5 size-3.5 accent-[var(--cm-accent)]"
                  checked={scopes.includes(scope)}
                  aria-label={label}
                  onChange={(event) => {
                    const checked = event.target.checked;
                    setScopes((current) =>
                      checked ? [...current, scope] : current.filter((held) => held !== scope),
                    );
                  }}
                />
                <span>
                  <span className="font-medium">{label}</span>{' '}
                  <span className="text-[var(--cm-text-secondary)]">{detail}</span>
                </span>
              </label>
            ))}
          </fieldset>
          <Button type="submit" size="sm" disabled={busy || !name.trim() || scopes.length === 0}>
            Connect an agent
          </Button>
        </form>

        {created && <Created created={created} />}
        {error && (
          <p role="alert" className="text-xs text-[var(--cm-danger-ink)]">
            {error}
          </p>
        )}

        <div className="space-y-2">
          <h3 className="text-xs font-semibold">Connected</h3>
          {active.length === 0 ? (
            <p className="text-xs text-[var(--cm-text-secondary)]">No agent is connected.</p>
          ) : (
            <ul className="divide-y divide-[var(--cm-glass-border)]" aria-label="Connected agents">
              {active.map((client) => (
                <li key={client.clientId} className="flex items-center justify-between gap-3 py-2">
                  <div className="min-w-0">
                    <p className="truncate text-xs font-medium">{client.name}</p>
                    <p className="text-[11px] text-[var(--cm-text-secondary)]">
                      {client.scopes.join(', ')} · last used {when(client.lastUsedUnixMillis)}
                    </p>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={busy}
                    aria-label={`Disconnect ${client.name}`}
                    onClick={() =>
                      void act(async () => {
                        await api.revokeAgent(client.clientId);
                        if (live.current && created?.client?.clientId === client.clientId) {
                          setCreated(null);
                        }
                      })
                    }
                  >
                    <Unplug aria-hidden />
                    Disconnect
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </div>

        <details className="group">
          <summary className="flex cursor-pointer list-none items-center gap-2 text-xs font-semibold">
            <History className="size-3.5" aria-hidden />
            Activity <span className="font-mono text-[var(--cm-text-muted)]">{audit.length}</span>
          </summary>
          {audit.length === 0 ? (
            <p className="mt-2 text-xs text-[var(--cm-text-secondary)]">Nothing yet.</p>
          ) : (
            <ul
              className="mt-2 max-h-60 space-y-1 overflow-y-auto font-mono text-[10px]"
              aria-label="Agent activity"
            >
              {audit.map((entry, index) => (
                <li
                  key={`${entry.atUnixMillis}-${index}`}
                  className={
                    entry.outcome === 'ok'
                      ? 'text-[var(--cm-text-secondary)]'
                      : 'text-[var(--cm-warning-ink)]'
                  }
                >
                  {when(entry.atUnixMillis)} · {entry.clientName} · {entry.operation}
                  {entry.outcome === 'ok' ? '' : ` · ${entry.outcome}: ${entry.detail}`}
                </li>
              ))}
            </ul>
          )}
        </details>
      </CardContent>
    </Card>
  );
}
