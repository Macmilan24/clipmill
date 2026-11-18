import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ConnectedAgents } from '../src/agents/ConnectedAgents.js';
import type { AgentClient, AgentsApi, EngineAuditEntry } from '../src/daemon/agents.js';

beforeEach(() => {
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function fakeApi() {
  let clients: AgentClient[] = [];
  const audit: EngineAuditEntry[] = [
    {
      atUnixMillis: 1_700_000_000_000,
      clientId: 'local',
      clientName: 'local',
      operation: 'list_projects',
      target: '',
      outcome: 'ok',
      detail: '',
    },
  ];
  const api: AgentsApi = {
    listAgents: async () => ({
      clients,
      cliPath: '/Applications/ClipMill.app/Contents/MacOS/clipmill',
    }),
    createAgent: async (name, scopes) => {
      const client = {
        clientId: 'agt_1',
        name,
        scopes: [...scopes],
        createdUnixMillis: 1,
        revokedUnixMillis: 0,
        lastUsedUnixMillis: 0,
      };
      clients = [client];
      return {
        client,
        token: 'cmt_secret',
        cliPath: '/Applications/ClipMill.app/Contents/MacOS/clipmill',
      };
    },
    revokeAgent: async () => {
      clients = clients.map((c) => ({ ...c, revokedUnixMillis: 2 }));
      return { clients, cliPath: null };
    },
    engineAudit: async () => audit,
  };
  return api;
}

describe('Connected agents', () => {
  it('connects an agent, shows its token once with the MCP configuration, and disconnects it', async () => {
    render(<ConnectedAgents api={fakeApi()} />);
    await screen.findByText('No agent is connected.');
    fireEvent.click(screen.getByRole('button', { name: 'Connect an agent' }));
    await screen.findByText('cmt_secret');
    expect(screen.getByText(/"CLIPMILL_TOKEN": "cmt_secret"/)).toBeTruthy();
    expect(screen.getByText(/"args": \[\s*"mcp"\s*\]/)).toBeTruthy();
    await screen.findByText(/read, edit · last used never/);
    fireEvent.click(screen.getByRole('button', { name: 'Disconnect Claude Desktop' }));
    await waitFor(() => expect(screen.queryByText('cmt_secret')).toBeNull());
    await screen.findByText('No agent is connected.');
  });

  it('lists what agents did', async () => {
    render(<ConnectedAgents api={fakeApi()} />);
    expect(await screen.findByText(/local · list_projects/)).toBeTruthy();
  });
});
