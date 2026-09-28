# Changelog

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
