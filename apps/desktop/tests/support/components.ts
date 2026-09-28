/**
 * Components that answer from memory, recording what a screen asked for.
 * An install moves the parts it names to queued; how far one gets and whether
 * it fails are the daemon's, which a test states through `next`.
 */
import type { Component, Components, ComponentsApi } from '../../src/daemon/components.js';

const MIB = 1024 ** 2;

export function component(name: string, overrides: Partial<Component> = {}): Component {
  return {
    name,
    title: name,
    family: `family-${name}`,
    state: 'missing',
    detail: '',
    installedBytes: 0,
    downloadBytes: 60 * MIB,
    process: 'stopped',
    restarts: 0,
    logPath: `/data/logs/workers/${name}.log`,
    tool: false,
    ...overrides,
  };
}

export function packaged(parts: readonly Component[]): Components {
  return { managed: true, parts, pythonVersion: '3.12.13', unavailable: '' };
}

type Call = ['list'] | ['install', string[]] | ['cancel'];

export class FakeComponents {
  components: Components;
  readonly calls: Call[] = [];
  next: ((components: Components, call: Call) => Components) | null = null;

  constructor(components: Components) {
    this.components = components;
  }

  asked(kind: Call[0]): Call[] {
    return this.calls.filter((call) => call[0] === kind);
  }

  api(): ComponentsApi {
    return {
      listComponents: () => this.answer(['list'], (components) => components),
      installComponents: (names) =>
        this.answer(['install', [...names]], (components) => ({
          ...components,
          parts: components.parts.map((part) =>
            (
              names.length === 0
                ? ['missing', 'outdated', 'failed'].includes(part.state)
                : names.includes(part.name)
            )
              ? { ...part, state: 'queued' as const, detail: '' }
              : part,
          ),
        })),
      cancelComponentInstall: () =>
        this.answer(['cancel'], (components) => ({
          ...components,
          parts: components.parts.map((part) =>
            ['queued', 'installing'].includes(part.state)
              ? { ...part, state: 'missing' as const }
              : part,
          ),
        })),
    };
  }

  private answer(call: Call, change: (components: Components) => Components): Promise<Components> {
    this.calls.push(call);
    this.components = change(this.components);
    if (this.next !== null) this.components = this.next(this.components, call);
    return Promise.resolve(this.components);
  }
}
