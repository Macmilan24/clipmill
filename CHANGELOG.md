# Changelog

## 0.4.0 — Studio tour, activity and update notices

- **A studio tour.** Two minutes through where everything is, from a long
  recording to a posted clip: each step lights the control it is about and
  dims the rest, and the screens that fill up only once there is a project
  (Results, review, the Editor, Export) are shown as small drawings instead
  of empty pages. It moves between screens and starts nothing. The welcome
  offers it at its end, and Settings → Getting started runs it again.

- **Says when a new version is out.** Once a day, ClipMill asks GitHub which
  release is newest and, when it is newer, says so at the top of the window
  with a link to its page. Nothing is downloaded or installed. The first
  question waits a day after installing. It counts as a network operation in
  Local Lock, and Settings → About turns it off.
- **An activity tray.** A button at the top of the window counts what is
  running (analyses, exports, model downloads, component installs, YouTube
  imports and uploads) and lists each with how far it has come; a row opens
  where that work is shown. When an analysis or export finishes while the
  window is in the background, the Dock icon bounces and the title says so,
  whichever screen is open.
- **Models is shorter.** Each job shows the model it uses, and any download
  under way; the other models fold under "Other models", which says how many
  are already on this computer.
- **More in Settings.** A Review section: whether a decision moves on to the
  next clip, and the speed clips play at (the player remembers a change too).
  Editing & export gains where exports go, how files are named, and the
  caption look new projects start with.
- **How fast Editorial AI runs here.** On Windows and Linux, Models shows the
  speed the editorial model measured on this computer, and warns when an
  analysis would be slow, suggesting a smaller model. It never refuses one.
- **Windows: the editorial check completes.** The runtime check could not
  write its receipt on Windows (a POSIX-only flag), so Editorial AI was never
  admitted there; failure traces had the same problem.
- **Transcripts leave out `[BLANK_AUDIO]`.** whisper.cpp writes silence and
  sound markers (`[BLANK_AUDIO]`, `[music]`, `(speaking in foreign
language)`) as words. Captions already hid them; now the transcript itself
  leaves them out, so discovery and the editorial model never read them.
  Each recording's transcript is assembled once more; nothing is
  transcribed again.
- **One recording in two projects.** Cancelling one project's analysis could
  fail the other's with "artifact key is already in flight": the cancelled
  step kept its unfinished output reserved, and the other gave up after three
  quick retries. A cancelled step now lets go of what it started, and a step
  whose output another run is already making waits for it, without using up
  a retry, then reuses it.

## 0.3.0 — Editorial AI on Windows and Linux

- **Editorial AI on Windows and Linux.** A local Qwen3.5 model can now propose
  and review clips on both platforms. The editorial component carries a pinned
  llama.cpp server and runs through Vulkan when a suitable graphics card is
  available, or on the processor. Apple silicon continues to use MLX.
- **More model choices.** Models offers Qwen3.5 9B and the lighter 4B GGUF
  builds, plus the option to use your own GGUF model with its vision projector.
  It shows only models the current computer can run; Windows and Linux do not
  show MLX models.
- **Setup and first run.** Editorial AI is available as a component in Setup
  and Models. After downloading a model, ClipMill checks that it runs before
  using it for analysis. Processor-only machines may take several minutes to
  validate a model and analyse a video. The built-in picker remains available
  without an editorial model.

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
