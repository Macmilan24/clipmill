# Releasing ClipMill

A release is a tag. Pushing `v<version>` runs
[`.github/workflows/release.yml`](../.github/workflows/release.yml), which
builds the installers on each platform's own runner and collects them into a
**draft** GitHub release. Nothing is public until a person publishes the draft.

## What a packaged app is

A development checkout builds its workers with `just setup` and starts them
with `just workers`. A packaged app carries instead:

- beside its executable: the release `clipmilld`, FFmpeg, FFprobe and uv, each
  pinned in [`bom.toml`](../bom.toml);
- in its resources: the caption faces, the emoji, the model manifests,
  licence notices, and the components — each worker as a wheel, its
  third-party requirements pinned by hash, and `engine.json`.

The shell recognises a bundle by `resources/engine/engine.json` and starts the
daemon with `--resources`, `--ffprobe` and `--uv`. The daemon then installs the
components when the person asks (Set up ClipMill), enrols a key for each
worker, keeps one process per installed component running, and on a Mac
proves the editorial model runs before admitting its Metal worker. See
decision R65 and [the threat model](threat-model.md).

## Cutting a release

1. Update the version in three places, keeping them equal: `version` in the
   root `Cargo.toml` (every crate inherits it, and the shell compares it with
   the daemon's), `apps/desktop/src-tauri/tauri.conf.json` and
   `apps/desktop/package.json`. Run `cargo check` so `Cargo.lock` follows.
2. Add the release to [`CHANGELOG.md`](../CHANGELOG.md).
3. Merge to `main` with CI green.
4. Tag the merge and push the tag:

    ```sh
    git tag v0.1.0 && git push origin v0.1.0
    ```

    The workflow refuses a tag that does not name the app's version.

5. When both build jobs pass, open the draft release, check the installers
   (install each on a clean machine or account if you can), edit the notes and
   publish.

A failed build can be re-run from the Actions page; a changed commit needs a
new tag (delete the old tag and the draft first).

## Trial builds

Run the release workflow by hand (**Actions → release → Run workflow**) on any
branch to build every installer without releasing anything: the installers are
kept as the run's artifacts for 14 days, and no draft is made. This is how a
Windows installer reaches a test machine before a release includes it.

## Signing

The workflow signs only with secrets that exist; without them it still builds.

**macOS.** Without a Developer ID the app is signed ad hoc, and people allow it
once in System Settings (see [Installing](install.md)). With an Apple
Developer account, add these repository secrets and the next release is
signed and notarized:

| Secret                                        | Value                                                                       |
| --------------------------------------------- | --------------------------------------------------------------------------- |
| `APPLE_CERTIFICATE`                           | the Developer ID Application certificate exported as `.p12`, base64-encoded |
| `APPLE_CERTIFICATE_PASSWORD`                  | the password the `.p12` was exported with                                   |
| `APPLE_SIGNING_IDENTITY`                      | e.g. `Developer ID Application: Name (TEAMID)`                              |
| `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` | for notarization: the Apple ID, an app-specific password, and the team ID   |

**Windows** (from v0.2): free signing for open-source projects is available
from the SignPath Foundation once the project has a public release.

**Linux** packages are not signed; `SHA256SUMS.txt` lists every file's digest.

## Building a release on your own machine

```sh
uv run --no-project --with packaging==25.0 python tools/release/stage.py --verify-engine
pnpm --filter @clipmill/desktop tauri build --config src-tauri/tauri.release.conf.json --bundles app,dmg
```

`stage.py` builds the release daemon and writes `apps/desktop/src-tauri/binaries`
and `resources` (both ignored by Git); `--verify-engine` installs every
component once, exactly as the app will, and imports each. To try the result
without touching your own projects, open it on a separate data folder — keep
the path short, since a Unix socket path must stay under 104 bytes:

```sh
open -n target/release/bundle/macos/ClipMill.app --env CLIPMILL_DATA_DIR=/tmp/cm-try
```

## Changing what ships

- **A worker's dependencies:** update its `uv.lock` as usual; the next staging
  exports the new requirements. The component's digest changes, so an
  installed app offers the update, and only that component is reinstalled.
  Worker wheels are built reproducibly, so a release that changes nothing in a
  component leaves it installed.
- **A new component:** add the worker to `tools/release/engine.toml` with its
  title, family, command, module and platforms.
- **FFmpeg, uv or Python:** change the pin in `bom.toml`; `stage.py` and
  `tools/fetch-ffmpeg.sh` verify every download against it, and
  `tools/security/check-bom.py` refuses a build that is not redistributable.
  An FFmpeg pin also names its `source`, where the provider publishes the
  build's scripts and library versions; the app's FFmpeg notice prints it.
- **The Visual C++ runtime** the Windows installer may run: follow
  <https://aka.ms/vc14/vc_redist.x64.exe> to Microsoft's versioned download
  and pin that URL; its path names the file's SHA-256, which `check-bom.py`
  requires the pin to repeat. The oldest runtime the installer accepts is in
  `apps/desktop/src-tauri/windows/installer-hooks.nsh`; raise it when a
  component's libraries are built with a newer toolset.

## Licences in the app

`stage.py` writes every licence and notice for what the app carries into
`resources/licenses`, indexed by its `README.txt`. `tools/release/notices.py`
lists the crates compiled into the two programs and the npm packages bundled
into the interface, with each one's own licence files and those of code it
vendors. A package that ships no licence text is given the standard text of a
licence it offers, and one that offers none the script knows stops the
release, so add its text to `tools/release/licenses` or `notices.py`.

`tools/release/licenses/JASSUB-LIBRARIES.txt` names the libraries compiled
into the caption player's WebAssembly, at the commits the JASSUB release was
built from, and the fallback font it ships (Liberation Sans), with each one's
own licence files. Staging refuses a JASSUB
version the notice does not name, so updating JASSUB means rebuilding that
notice for the new release.
