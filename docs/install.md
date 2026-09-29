# Installing ClipMill

ClipMill runs on **macOS 14 or later on Apple silicon** and on **64-bit Linux
(x86_64)**, and from version 0.2 on **64-bit Windows 10 and 11** as a beta.

Download the installer for your computer from the
[latest release](https://github.com/Macmilan24/clipmill/releases/latest).
`SHA256SUMS.txt` in the same release lists the digest of every file.

## macOS

1. Open `ClipMill_<version>_aarch64.dmg` and drag ClipMill to Applications.
2. Open ClipMill from Applications. This build is not notarized by Apple, so
   macOS says it cannot verify the app and offers only to move it to the Bin.
   Choose **Done**, then open **System Settings → Privacy & Security**, scroll
   to the message about ClipMill and choose **Open Anyway**. macOS asks once
   more; choose **Open**. You only do this once.

An Intel Mac is not supported: the local editorial model needs Apple silicon,
and the pinned FFmpeg is built for it.

## Linux

Either package works on distributions with at least the glibc of Ubuntu
22.04:

- **AppImage** — make it executable and run it:
  `chmod +x ClipMill_<version>_amd64.AppImage && ./ClipMill_<version>_amd64.AppImage`.
  It carries its own WebView libraries.
- **Debian and Ubuntu** — `sudo apt install ./ClipMill_<version>_amd64.deb`,
  which also installs the WebView, OpenGL and GLib libraries it needs.

On Linux, clips are chosen by ClipMill's built-in picker. The local editorial
model (Qwen) currently runs only on Apple silicon.

## Windows (beta)

Windows support is new: try it, and report what happens.

1. Download `ClipMill_<version>_x64-setup.exe` and run it. The installer is not
   signed yet, so Windows may say **Windows protected your PC**: choose **More
   info**, then **Run anyway**.
2. ClipMill installs for your user account and adds itself to the Start menu.
   It draws its window with Microsoft Edge WebView2, which Windows 11 includes;
   the installer fetches it where it is missing.
3. ClipMill's components need Microsoft's Visual C++ runtime, version 14.44 or
   newer, which many PCs already have. Where it is missing or older, the
   installer runs Microsoft's own installer for it, and Windows asks for
   permission first. If you decline, ClipMill still installs; install the
   runtime [from Microsoft](https://aka.ms/vc14/vc_redist.x64.exe) before you
   set ClipMill up.

As on Linux, clips are chosen by ClipMill's built-in picker.

## First run: Set up ClipMill

The app arrives without its components (the programs that run the models) and
without model weights. New Project opens with **Set up ClipMill**, which lists
what it installs and roughly how much it downloads, and one button does all
of it:

|                                                   | macOS (Apple silicon)                        | Linux        |
| ------------------------------------------------- | -------------------------------------------- | ------------ |
| Components: Python 3.12 and the workers' packages | about 530 MB                                 | about 300 MB |
| Recommended models                                | about 6.5 GB (with the Qwen editorial model) | about 560 MB |

Components come from GitHub (the pinned Python build) and PyPI, every package
pinned by its SHA-256; models come from Hugging Face at a pinned commit and are
checked against their pinned digests. Nothing about your projects is sent.
Each download counts as a network operation in the Local Lock badge. After
setup, analysis and export work without a connection.

On a Mac, once the editorial component and the Qwen model are both installed,
ClipMill checks once that the model runs on your Mac (about a minute) before
it gives it work. **Models** shows every component and model, whether each is
running, and lets you update, retry or remove them.

## Importing from YouTube

YouTube import is one of the components Set up installs. It also needs
[Node.js](https://nodejs.org) 22 or newer on your computer, which yt-dlp uses
to read YouTube's player; ClipMill finds it wherever Homebrew, nvm, Volta, fnm
or the official installer put it. Import only videos you have permission to
use.

## Where ClipMill keeps things

|                                       | macOS                                                 | Linux                     | Windows                                 |
| ------------------------------------- | ----------------------------------------------------- | ------------------------- | --------------------------------------- |
| Projects, components, models and logs | `~/Library/Application Support/dev.clipmill.ClipMill` | `~/.local/share/clipmill` | `%LOCALAPPDATA%\clipmill\ClipMill\data` |

Inside it, `engine/` holds the components, `models/` the weights, and `logs/`
what each component's process says, which is the first place to look when
something stops. **Settings → Storage** shows what uses the space and frees
what is safe to remove.

Quitting ClipMill stops its background processes. Work in progress, such as
an export, resumes the next time you open it.

## Uninstalling

Delete the app (macOS: drag it from Applications to the Bin; Debian/Ubuntu:
remove the ClipMill package with your package manager; AppImage: delete the
file; Windows: **Settings → Apps → Installed apps → ClipMill → Uninstall**),
then delete the folder above to remove projects, components and models.

## Troubleshooting

- **A component shows "Restarting"** — its process stopped and ClipMill starts
  it again with a growing pause. Its log is in `logs/workers/`; include it when
  you report the problem.
- **Set up stopped with a reason** — choose **Try again**. The full installer
  output is in `logs/engine-install.log`.
- **The window stays blank on Linux with an NVIDIA GPU** — start ClipMill with
  `WEBKIT_DISABLE_DMABUF_RENDERER=1`.

Report problems at
[github.com/Macmilan24/clipmill/issues](https://github.com/Macmilan24/clipmill/issues).

## Licences

ClipMill is free software under the GNU Affero General Public License,
version 3. The licences and notices of everything the app carries (FFmpeg,
uv, the libraries and packages it is built from, and its fonts) are in the
app's `resources/licenses` folder, starting with `README.txt`. The Windows
installer also carries Microsoft's Visual C++ Redistributable, under
Microsoft's licence terms, and runs it only where a PC lacks the runtime. On a Mac,
Control-click ClipMill in Applications, choose **Show Package Contents**, and
open `Contents/Resources/resources/licenses`.
