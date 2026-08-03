# Export, archive, and the Local Lock

What W25 built, and the four claims it is held to.

## The strip

An export is the last thing a user does and the first thing they would blame,
so nothing leaves without four questions being answered. Each answer is a
finding carrying its own reason, because "export failed" is not something
anybody can act on.

| Check                   | Blocks when                                                | Why it is not a warning                                                                                                          |
| ----------------------- | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Rights present          | The attestation is blank                                   | An export carries a claim about the footage. A blank one is still a claim.                                                       |
| Rights gate             | The clip runs past 60 s and the confirmation was not given | Sixty seconds is where platforms stop treating a clip as a short, which is where a rights position starts being worth something. |
| Word-snapped boundaries | A cut lands strictly inside a caption word                 | The same rule the boundary optimizer follows upstream. A cut inside a word is a cut a viewer hears.                              |
| Reading speed           | An accessibility cue exceeds the profile                   | These become the SRT and VTT that leave the building.                                                                            |
| Disk headroom           | The estimate does not fit                                  | An export that starts and stops halfway is a folder of partial files.                                                            |

Two severities, and the line between them is a design decision rather than a
scale. **Blocking** means the delivered file would be wrong or would not fit;
**advisory** means a person might have meant it.

The burn-in caption track is what makes that line real. It runs deliberately
hot — a few words at a time, held briefly, is what the kinetic intent is _for_ —
so a reading-rate finding against it is advice. The same finding against the
accessibility cues blocks. One document, two intents, two answers.

Two limits are worth stating rather than discovering:

- **Shot-cut avoidance is not re-checked here.** That violation needs the shot
  index, which the caption engine has and the strip does not. Passing an empty
  list of cuts would silently pass the check, so the strip does not claim to run
  it.
- **The estimate is a ceiling, not a measurement.** 1080×1920 at CRF 18 is
  content-dependent by a factor of several, so the pre-flight figure is drawn
  from the noisiest material. A check that under-estimated would let an export
  start and fail halfway, which is the failure it exists to prevent. The real
  byte count replaces it once the render exists.

## One implementation of the names

The naming pattern is resolved in exactly one place: `clipmill-export::naming`,
in the daemon. The Export screen asks for a preview over the socket on every
keystroke and draws the answer.

This is deliberate churn. The alternative — resolving the pattern in the
renderer — is two implementations of the same rules, and the failure mode is a
preview a user approved that is not the name they got. It is the same rule the
editor's player follows for crops and cue windows; see
[preview-parity.md](preview-parity.md).

Tokens: `{project}`, `{clip}`, `{index}`, `{duration}`, `{date}`, `{address}`.
Two patterns are refused rather than accepted-and-mangled: one naming a token
nobody implements, and one that would give every clip in an export the same
stem — the second delivery would overwrite the first, and de-duplicating
silently would be worse than saying so.

Nothing in the naming code reads a clock. `{date}` arrives as an argument,
because a pure function that reads the day is one whose output nobody can
reproduce.

## What an export writes

Seven files, from one list read by both the delivery and the preview:

```
<stem>.mp4                     the clip
<stem>.srt                     accessibility sidecar
<stem>.vtt                     accessibility sidecar
<stem>.render-manifest.json    what the renderer measured
<stem>.jpg                     one frame, taken from the delivered clip
<stem>.metadata.json           clipmill.export.package.v1
<stem>.sha256                  sha256sum-compatible
```

The checksum file is the format `sha256sum -c` reads, so verifying an export
needs nothing from this project. That is the point of writing it.

Three rules govern the writing:

1. **Files are written every time**, cache hit or not. The package artifact
   describes a delivery; it is not the delivery. A user who deleted their export
   and asked again gets it back.
2. **Nothing lands under its final name until it is complete.** Each file is
   written under a temporary suffix and renamed into place, because a partial
   file under the delivered name is indistinguishable from a finished one.
3. **Every byte is re-verified on the way out.** The clip and sidecars are read
   back through the render's artifact lease, so an artifact that rotted since it
   was written is refused rather than delivered.

The thumbnail is taken from the delivered clip rather than the source, a tenth
of the way in and clamped to three seconds. From the source it would be a
thumbnail of the wrong picture: the crop, the captions, and the colour are all
the render's.

### Where an export may go

Local directories only. A mounted network share looks like an ordinary
directory, so the path is not asked — the kernel is, via `statfs`. On macOS the
refused filesystems are `nfs`, `smbfs`, `afpfs`, `webdav`, and `ftp`; on Linux
the NFS, SMB/CIFS, and Coda magic numbers.

A destination that could not be classified is _allowed_, not refused: refusing
every unrecognised filesystem would refuse working local disks. Windows is a
Phase 2 target and has no implementation, so it returns "not classified" rather
than claiming to have checked.

## The archive

`ExportArchive` packs a project into a zip described by
`clipmill.archive_index.v1`: the project state, every edit document, the command
log behind each one, the clip decisions, and the render manifests.

**Media is named, not carried.** A project's recordings are three orders of
magnitude larger than its documents and already exist on the user's disk. Every
source travels as `(source_id, fingerprint, display_name)`, so a re-import can
say which file it is looking for rather than failing on a path that stopped
being true.

Each command log carries the document it started from as well as the commands,
because a list of commands without the thing they were applied to replays to
nothing. Inverses travel too, so the history can be walked backwards.

### Why the ZIP writer is hand-written

Determinism. Two archives of an unchanged project must be the same bytes, or the
round-trip gate cannot compare them. Every general-purpose writer stamps the
current time into each entry, and several record the host platform and
permission bits as well — all of which make identical content produce different
files, and fixing it from outside means overriding most of what the library
does.

So: entries are **stored** rather than deflated, timestamps are pinned to the
format's own epoch (1980-01-01), the host byte is zero, external attributes are
zero, and the caller controls the order. The one field that legitimately varies
is `created_unix_millis` in the index, which is project state and is what lets a
person tell two archives apart.

Zip64 is not implemented. The two limits that would need it — 65 535 entries and
4 GiB — are refused with a reason rather than written as a file that some tools
open and others do not.

## The Local Lock, read rather than asserted

Health used to answer `local_lock: true` with a literal. A claim with no way to
come out false is not evidence, so it is now derived from two things that change
when the daemon changes:

- **The stage registry.** Every kind the daemon will run declares a network
  policy. A stage added with network access turns the answer false without
  anybody editing a boolean.
- **A counter of what started.** Every task either lease path begins is offered
  to the policy, and one declaring anything but `local-lock` is counted.

Both halves must hold. The Settings card shows all three numbers — stages
registered, stages allowed the network, egress attempts this run — because
"engaged" on its own is a word, while "twenty-eight stages, none network-allowed,
nothing attempted" is a count of the table that decides.

The counter reads zero and is expected to forever. It exists so that a non-zero
reading is _possible_, which is what makes a zero one worth showing. It counts
since the process started, because a durable total would be a number nobody
could attribute to a run.

## What Settings does not do

Everything on the screen is read. Retention is displayed because the number is
real and the collection policy that would let a user move it is not written;
moving the storage location, editing the retention window, and per-project
privacy rules are Phase 2 and say so. The delivery profile is stated on the
Export screen for the same reason: Phase 1 delivers one, and a control that let
you change it would be a control that changed nothing.

## The gate

`just gate-export` runs `tools/drills/export-drill.sh`:

- the strip's refusals, and the burn-in track advising rather than blocking
- destination classification and naming resolution
- the Local Lock derived from the registry
- the published documents parsed by the types generated from their schemas
- **an archive opened by Python's `zipfile`** — CRCs, central directory, and
  all — with every entry verified against the digest the index claims, every
  path checked for escapes, and the absence of media confirmed
- two archives of one project compared byte for byte
- the screens: a blocking finding stops an export, and the names shown are the
  daemon's rather than ones the renderer composed

The Python leg is the one that matters most. A zip this project verifies with
its own code is a zip nobody else has agreed to; opening it with an unrelated
reader is the only check that can fail when the writer is wrong about its own
format.
