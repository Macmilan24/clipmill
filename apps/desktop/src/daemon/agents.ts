/**
 * Connected agents: the tokens a person gives agents and the `clipmill`
 * command, what each may do, and the log of what they did (docs/engine-api.md).
 */
import { isTauri } from './client.js';

export type AgentScope = 'read' | 'edit' | 'render' | 'publish' | 'delete';

export const AGENT_SCOPES: readonly { scope: AgentScope; label: string; detail: string }[] = [
  { scope: 'read', label: 'Read', detail: 'See projects, transcripts and clips' },
  {
    scope: 'edit',
    label: 'Edit',
    detail: 'Add recordings, analyze them, decide on clips, make edits',
  },
  { scope: 'render', label: 'Render', detail: 'Export clips to files on this computer' },
];

export interface AgentClient {
  readonly clientId: string;
  readonly name: string;
  readonly scopes: readonly string[];
  readonly createdUnixMillis: number;
  /** Zero while the token works. */
  readonly revokedUnixMillis: number;
  readonly lastUsedUnixMillis: number;
}

export interface Agents {
  readonly clients: readonly AgentClient[];
  /** The `clipmill` command an agent host starts; null when this build has none. */
  readonly cliPath: string | null;
}

export interface AgentCreated {
  readonly client: AgentClient | null;
  /** Shown once. ClipMill keeps only its hash. */
  readonly token: string;
  readonly cliPath: string | null;
}

export interface EngineAuditEntry {
  readonly atUnixMillis: number;
  readonly clientId: string;
  readonly clientName: string;
  readonly operation: string;
  readonly target: string;
  readonly outcome: 'ok' | 'refused' | 'failed' | string;
  readonly detail: string;
}

export interface AgentsApi {
  listAgents(): Promise<Agents>;
  createAgent(name: string, scopes: readonly AgentScope[]): Promise<AgentCreated>;
  revokeAgent(clientId: string): Promise<Agents>;
  engineAudit(limit: number): Promise<readonly EngineAuditEntry[]>;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error('Connected agents are available in the desktop app.');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

export const agentsApi: AgentsApi = {
  listAgents: () => call('list_agents'),
  createAgent: (name, scopes) => call('create_agent', { name, scopes: [...scopes] }),
  revokeAgent: (clientId) => call('revoke_agent', { clientId }),
  engineAudit: (limit) => call('engine_audit', { limit }),
};

/**
 * What an MCP host such as Claude Desktop needs to start ClipMill for an agent:
 * the `clipmill` command, the `mcp` argument, and the agent's token.
 */
export function mcpConfig(cliPath: string, token: string): string {
  return JSON.stringify(
    {
      mcpServers: {
        clipmill: { command: cliPath, args: ['mcp'], env: { CLIPMILL_TOKEN: token } },
      },
    },
    null,
    2,
  );
}
