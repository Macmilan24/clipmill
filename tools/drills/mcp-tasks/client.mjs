// Drive a real MCP client against a new or restarted local ClipMill daemon.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { Client, StreamableHTTPClientTransport } from '@modelcontextprotocol/client';
import { StdioClientTransport } from '@modelcontextprotocol/client/stdio';
import { createTaskSessionFromClient } from '@modelcontextprotocol/ext-tasks/client';

const fixture = JSON.parse(readFileSync(0, 'utf8'));
const { phase, socket, cli, token, url, project, source, taskId } = fixture;
const extension = 'io.modelcontextprotocol/tasks';
const version = '2026-07-28';
const capabilities = { extensions: { [extension]: {} } };
const clientInfo = { name: 'clipmill-conformance', version: '1.0' };

async function connect(mode, transport, caps = capabilities) {
    const client = new Client(clientInfo, { capabilities: caps, versionNegotiation: { mode } });
    await client.connect(transport);
    return client;
}

function stdio() {
    return new StdioClientTransport({
        command: cli,
        args: ['--socket', socket, 'mcp'],
        env: { CLIPMILL_TOKEN: token },
        stderr: 'inherit',
    });
}

function http() {
    return new StreamableHTTPClientTransport(new URL(url), {
        requestInit: { headers: { Authorization: `Bearer ${token}` } },
        // Extension methods define a routing header beyond the core tools/call mapping.
        fetch: (input, init) => {
            const headers = new Headers(init.headers);
            const body = typeof init.body === 'string' && JSON.parse(init.body);
            if (body?.method?.startsWith('tasks/')) headers.set('mcp-name', body.params.taskId);
            return fetch(input, { ...init, headers });
        },
    });
}

// The v2 extension adapter delegates dispatch to its host. Use a second
// official transport and correlate the wire responses by JSON-RPC ID.
async function taskSession(client, transport) {
    const pending = new Map();
    let sequence = 0;
    let disconnected = false;
    transport.onmessage = (message) => {
        const reply = pending.get(message.id);
        if (!reply) return;
        pending.delete(message.id);
        reply(
            message.error
                ? { kind: 'error', error: message.error }
                : { kind: 'result', result: message.result },
        );
    };
    await transport.start();
    transport.setProtocolVersion?.(version);
    const session = createTaskSessionFromClient(client, {
        endpointId: 'clipmill-conformance',
        v2RequestFraming: {
            protocolVersion: version,
            clientInfo,
            clientCapabilities: capabilities,
        },
        rawDispatch: async (request) => {
            if (disconnected) throw new Error('Transport disconnected');
            const id = ++sequence;
            const result = new Promise((resolve) => pending.set(id, resolve));
            try {
                await transport.send({ jsonrpc: '2.0', id, ...request });
                return await result;
            } finally {
                pending.delete(id);
            }
        },
    });
    return {
        session,
        close: async () => {
            // Closing a session can cancel in-flight handles. Disconnect the
            // transport first to exercise a lost client without cancelling work.
            disconnected = true;
            await transport.close();
            await session.close();
        },
    };
}

if (phase === 'start') {
    const legacy = await connect('legacy', stdio(), {});
    const modern = await connect('auto', stdio());
    const tasks = await taskSession(modern, stdio());
    try {
        assert.equal(legacy.getNegotiatedProtocolVersion(), '2025-11-25');
        assert((await legacy.listTools()).tools.some((tool) => tool.name === 'clipmill_analyze'));
        assert.equal(modern.getNegotiatedProtocolVersion(), version);
        assert((await modern.discover()).capabilities.extensions[extension]);
        assert(
            (await modern.listTools()).tools.some((tool) => tool.name === 'clipmill_export_clip'),
        );
        const args = { project_id: project, source_id: source, editorial: 'none' };
        const options = { metadata: { 'io.clipmill/requestId': fixture.key ?? 'first-analysis' } };
        const started = await tasks.session.callTool('clipmill_analyze', args, options);
        assert.equal(started.kind, 'task');
        const id = started.handle.taskId;
        const progress = await tasks.session.task(id).snapshot();
        await started.detach();
        const replay = await tasks.session.callTool('clipmill_analyze', args, options);
        assert.equal(replay.handle.taskId, id);
        await replay.detach();
        process.stdout.write(JSON.stringify({ taskId: id, createdAt: progress.createdAt }));
    } finally {
        await tasks.close();
        await modern.close();
        await legacy.close();
    }
} else if (phase === 'export-forms') {
    for (const action of ['decline', 'accept']) {
        const client = await connect('auto', http(), { elicitation: { form: {} } });
        let presented = 0;
        client.setRequestHandler('elicitation/create', async () => {
            presented++;
            return action === 'accept'
                ? { action, content: { rights: 'I own this synthetic recording.' } }
                : { action };
        });
        try {
            const uri = 'clipmill://edits/' + fixture.docId;
            const resource = await client.readResource({ uri });
            assert.equal(JSON.parse(resource.contents[0].text).doc_id, fixture.docId);
            const result = await client.callTool({
                name: 'clipmill_export_clip',
                arguments: { doc_id: fixture.docId, destination_dir: fixture.destination },
            });
            assert.equal(presented, 1);
            if (action === 'decline') {
                assert.equal(result.isError, true);
                assert.match(result.content[0].text, /cancelled/);
            } else {
                assert.equal(result.isError, false, JSON.stringify(result.content));
                assert(result.structuredContent.job_id);
            }
        } finally {
            await client.close();
        }
    }
} else if (phase === 'revoked') {
    await assert.rejects(connect('auto', http()), /401|unauthorized|revoked/i);
} else {
    const client = await connect('auto', http());
    const tasks = await taskSession(client, http());
    try {
        const task = tasks.session.task(taskId);
        if (phase === 'other') {
            for (const invoke of [
                () => task.snapshot(),
                () => task.update({}),
                () => task.cancel(),
            ]) {
                await assert.rejects(invoke(), /unknown task|-32602/i);
            }
        } else {
            const poll = await task.snapshot();
            assert.equal(poll.createdAt, fixture.createdAt);
            await task.update({});
            if (phase === 'cancel') {
                await task.cancel();
                let current = await task.snapshot();
                for (let i = 0; i < 40 && current.status === 'working'; ++i) {
                    await new Promise((done) => setTimeout(done, 150));
                    current = await task.snapshot();
                }
                assert.equal(current.status, 'cancelled');
            } else if (phase === 'terminal') {
                assert.equal(poll.status, 'completed');
                assert.equal(poll.raw.result.resultType, 'complete');
                assert.equal(poll.raw.result.isError, false);
                assert.equal(poll.raw.result.structuredContent.job_id, taskId);
            }
        }
    } finally {
        await tasks.close();
        await client.close();
    }
}
