/**
 * Freeing space from Settings: each clean-up says what goes and what stays
 * before it runs, reports what the daemon freed, and nothing runs unasked.
 */
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { CleanAction, CleanResult } from '../src/daemon/models.js';
import { Settings } from '../src/screens/Settings.js';
import { SettingsScreen } from '../src/screens/SettingsScreen.js';
import { emptyWorld, fakeApi } from './support/library.js';
import { FakeModelLibrary, storageWithCleanUp } from './support/models.js';

const GIB = 1024 ** 3;

function category(name: string): HTMLElement {
  return screen.getByRole('listitem', { name });
}

function show(
  handlers: {
    onCleanStorage?: (action: CleanAction) => Promise<CleanResult>;
    onOpenStorage?: (key: string) => Promise<void>;
    onOpenModels?: () => void;
  } = {},
  storage = storageWithCleanUp(),
) {
  render(<Settings storage={storage} lock={null} loading={false} error={null} {...handlers} />);
}

describe('storage clean-up', () => {
  it('estimates what unused generated files would free and confirms before removing them', async () => {
    const onCleanStorage = vi.fn(async () => ({ freedBytes: 3 * GIB, removedItems: 41 }));
    show({ onCleanStorage });

    const media = category('Generated media');
    fireEvent.click(within(media).getByRole('button', { name: 'Free 3 GB in Generated media' }));
    expect(
      within(media).getByText(/Remove 41 generated files no project uses \(3 GB\)\?/),
    ).toBeTruthy();
    expect(onCleanStorage).not.toHaveBeenCalled();
    fireEvent.click(within(media).getByRole('button', { name: 'Delete' }));

    expect(await within(media).findByText('Freed 3 GB (41 items).')).toBeTruthy();
    expect(onCleanStorage).toHaveBeenCalledWith('unused_files');
    expect(screen.getByText(/About 3 GB can be freed now with Clean up/)).toBeTruthy();
  });

  it('says what it is doing while the engine checks every file a project uses', async () => {
    let finish: ((result: CleanResult) => void) | undefined;
    const onCleanStorage = vi.fn(
      () =>
        new Promise<CleanResult>((resolve) => {
          finish = resolve;
        }),
    );
    show({ onCleanStorage });

    const media = category('Generated media');
    fireEvent.click(within(media).getByRole('button', { name: 'Free 3 GB in Generated media' }));
    fireEvent.click(within(media).getByRole('button', { name: 'Delete' }));
    expect(
      await within(media).findByText(/Checking every file your projects use before removing/),
    ).toBeTruthy();
    expect(
      within(category('Database backups'))
        .getByRole('button', { name: 'Delete old backups in Database backups' })
        .hasAttribute('disabled'),
    ).toBe(true);

    finish?.({ freedBytes: 3 * GIB, removedItems: 41 });
    expect(await within(media).findByText('Freed 3 GB (41 items).')).toBeTruthy();
    expect(within(media).queryByText(/Checking every file/)).toBeNull();
  });

  it('keeps the newest database backup and says so', async () => {
    const onCleanStorage = vi.fn(async () => ({ freedBytes: 300 * 1024 ** 2, removedItems: 4 }));
    show({ onCleanStorage });
    const backups = category('Database backups');
    fireEvent.click(
      within(backups).getByRole('button', { name: 'Delete old backups in Database backups' }),
    );
    expect(
      within(backups).getByText('Delete 4 older database backups? The newest is kept.'),
    ).toBeTruthy();
    fireEvent.click(within(backups).getByRole('button', { name: 'Delete' }));
    await waitFor(() => {
      expect(onCleanStorage).toHaveBeenCalledWith('backups');
    });
  });

  it('offers nothing to clean where nothing can be freed', () => {
    const base = storageWithCleanUp();
    show(
      { onCleanStorage: vi.fn() },
      {
        ...base,
        reclaimableBytes: 0,
        reclaimableItems: 0,
        categories: base.categories.map((entry) =>
          entry.key === 'backups'
            ? { ...entry, items: 1 }
            : entry.key === 'temporary'
              ? { ...entry, bytes: 0, items: 0 }
              : entry,
        ),
      },
    );
    for (const [name, label] of [
      ['Generated media', 'Clean up unused in Generated media'],
      ['Database backups', 'Delete old backups in Database backups'],
      ['Temporary files', 'Clear in Temporary files'],
    ] as const) {
      expect(
        within(category(name)).getByRole('button', { name: label }).hasAttribute('disabled'),
      ).toBe(true);
    }
    expect(screen.getByText(/Nothing is waiting to be cleaned up/)).toBeTruthy();
  });

  it("reports the daemon's refusal where the clean-up was asked for", async () => {
    show({
      onCleanStorage: async () => {
        throw new Error(
          'The clean-up stopped to keep your projects safe: a reachable object is missing',
        );
      },
    });
    const temporary = category('Temporary files');
    fireEvent.click(within(temporary).getByRole('button', { name: 'Clear in Temporary files' }));
    fireEvent.click(within(temporary).getByRole('button', { name: 'Clear' }));
    expect(await within(temporary).findByText(/stopped to keep your projects safe/)).toBeTruthy();
  });

  it('opens a category folder by its key and sends weights to Models', async () => {
    const onOpenStorage = vi.fn(async () => undefined);
    const onOpenModels = vi.fn();
    show({ onOpenStorage, onOpenModels });
    fireEvent.click(
      within(category('Generated media')).getByRole('button', {
        name: 'Open the generated media folder',
      }),
    );
    await waitFor(() => {
      expect(onOpenStorage).toHaveBeenCalledWith('artifacts');
    });
    fireEvent.click(
      within(category('Model weights')).getByRole('button', { name: 'Manage in Models' }),
    );
    expect(onOpenModels).toHaveBeenCalledTimes(1);
  });

  it('reports only, with no clean-up offered, when nothing can act on it', () => {
    show();
    expect(screen.queryByRole('button', { name: /Free 3 GB/ })).toBeNull();
    expect(screen.queryByRole('button', { name: /Open the/ })).toBeNull();
    expect(screen.getByText('Database backups')).toBeTruthy();
    expect(screen.getByText('Temporary files')).toBeTruthy();
  });
});

describe('storage clean-up through the engine', () => {
  it('replaces the report with the one the daemon measured after cleaning', async () => {
    const models = new FakeModelLibrary();
    const after = storageWithCleanUp({ reclaimableBytes: 0, reclaimableItems: 0 });
    models.cleanResult = { freedBytes: 3 * GIB, removedItems: 41, storage: after };
    const api = fakeApi({ ...emptyWorld(), models, storage: storageWithCleanUp() });
    api.fetchLocalLock = async () => ({
      engaged: true,
      stages: 28,
      networkAllowedStages: 0,
      egressAttempts: 0,
    });
    render(<SettingsScreen api={api} />);

    const media = await screen.findByRole('listitem', { name: 'Generated media' });
    fireEvent.click(within(media).getByRole('button', { name: 'Free 3 GB in Generated media' }));
    fireEvent.click(within(media).getByRole('button', { name: 'Delete' }));

    expect(await screen.findByText(/Nothing is waiting to be cleaned up/)).toBeTruthy();
    expect(models.asked('clean')).toEqual([['clean', 'unused_files']]);
    fireEvent.click(
      within(screen.getByRole('listitem', { name: 'Imported originals' })).getByRole('button', {
        name: 'Open the imported originals folder',
      }),
    );
    await waitFor(() => {
      expect(models.asked('open')).toEqual([['open', 'imports']]);
    });
  });
});
