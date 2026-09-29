# Changelog

## Unreleased

- **Editorial AI on Windows and Linux.** The local editorial model now runs
  there too, in llama.cpp's server, which the editorial component carries: on
  the graphics card through Vulkan, or on the processor. Models offers Qwen3.5
  9B and a lighter 4B as GGUF builds, and your own GGUF model with its vision
  projector. Models lists only what this computer can run (decision R70).

## 0.2.0 — Windows (beta)

- **Windows (beta).** ClipMill runs on 64-bit Windows 10 and 11: the same
  daemon, components and workers as on macOS and Linux, installed with
  `ClipMill_0.2.0_x64-setup.exe`. The app talks to its daemon and workers over
  loopback, each end proving it holds a secret the daemon made at start
  (decision R68). Clips are chosen by the built-in picker, as on Linux. The
  installer installs Microsoft's Visual C++ runtime where a PC lacks it, and
  the daemon carries its own (decision R69). See
  [Installing ClipMill](docs/install.md#windows-beta).
- **Export names** never begin with a name Windows keeps for a device (`CON`,
  `NUL`, `COM1` and so on), so a folder of clips made anywhere copies onto
  Windows intact.

## 0.1.0 — first release

The first installable ClipMill, for macOS on Apple silicon (macOS 14 or later)
and 64-bit Linux. See [Installing ClipMill](docs/install.md).

**What it does**

- Turns a long recording into short clips on your own computer: import a local
  video, or a YouTube video you have permission to use, and analyse it.
- Transcribes speech with word timing, tells voices apart, finds shots and
  faces, and proposes moments. On Apple silicon a local Qwen model proposes and
  reviews the moments; on Linux a built-in picker does.
- Review suggestions in the Inspector, with the clip shown as the frame it
  will be, and keep, reject or make your own.
- Edit in the Editor: trims, word-level caption corrections, five highlight
  styles and seven caption faces drawn exactly as the export burns them in,
  framing that follows the speaker or the person you click, split and
  picture-in-picture layouts, punch-ins, shapes (9:16, 4:5, 1:1, 16:9), hook
  titles and text, colour emoji, a brand kit, a music bed that drops under the
  words, voice clean-up and B-roll cutaways. Undo and history survive restarts.
- Export MP4 with SRT and VTT sidecars, at the recording's frame rate or 30/60,
  up to 4K, individually or in batches.

**Setting up** — the app installs its components and the recommended models
from inside the app, in one step, with every download pinned by digest. After
that, analysis and export work offline. The Local Lock badge counts every
network operation the app starts.

**Known limitations**

- The local editorial model needs Apple silicon; Linux uses the built-in picker.
- The Mac app is not notarized yet: macOS asks you to allow it once.
- There is no automatic update: download a new release to update. The app
  offers to update its components when a new version brings new ones.
- Importing from YouTube needs Node.js 22 or newer installed on the computer.
- Windows is planned for a later release.
