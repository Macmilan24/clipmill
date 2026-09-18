# Desktop workspace refresh

The interface follows the original Stitch direction's compact type, restrained
indigo accent, and focus on footage. The working application now determines the
layout: opaque neutral surfaces, fewer nested cards, and one owner for page
padding. Shared tokens cover light and dark modes, including readable accent
text independently of button fills. Standard borders have a defined color rather
than falling back to text color.

The live sidebar groups Library, New Project, Results, Editor and Export above
Models and Settings. Unimplemented Discovery and Brand destinations are hidden
from normal navigation. Old routes still resolve. New sessions begin in Library.
A compact icon rail leaves room for editing in narrow desktop windows.

Results has aligned columns, a persistent action area, real review explanations,
and explicit project selection. Inspector gives the video most of the space;
previous/next controls keep every candidate reachable when the candidate rail
is hidden. Editor has a fixed header, a bounded preview, independently scrollable
properties, and a timeline whose tracks use one coordinate system. Export keeps
file naming, validation and the delivery format together, with a return to the
same edit.

Behavior corrected with the refresh:

- Returning to Results preserves project, recording and analysis run, including
  after relaunch. An unavailable project does not open a different one.
- Late project loads and crop calculations cannot overwrite newer selections.
  Approval responses cannot redirect a screen the user already left or reload
  an older project over the one now selected.
- Caption text fields, sliders and tabs own their keyboard events. Playback
  shortcuts respect those controls, with Space for playback and arrow keys for
  stepping. Playback failures and edit errors are visible outside property tabs.
- Inspector preview includes audio and a mute control. Normalized crops transform
  the source video, not the narrower output wrapper.
- Timeline tracks seek by pointer and keyboard; framing runs are compacted rather
  than producing one DOM element for every video frame.
- Speaker-follow calculates a path before switching a fitted clip's layout.
- Export remains disabled while a revised destination or filename is being
  checked. Models reads actual installation and worker readiness instead of a
  hardcoded zero. Privacy indicators do not invent transferred-byte counts.

## Browser development preview

Run `pnpm --filter @clipmill/desktop dev` and open
`http://127.0.0.1:5173/preview.html`. This separate development entry uses clearly
labeled synthetic data and never writes to the daemon. It is not an entry in the
production build. `?screen=editor&theme=light` selects a screen and theme; `media`
may name a video URL served by that same development server. Fixture caption
corrections are temporary. Actual command persistence, validation and export
remain covered by the renderer's tests over the host API.

## Verification

Renderer tests cover identity through Results → Inspector → Editor → Export,
caption/trim commands, export revisions and delivery recovery. New regressions
cover delayed result/crop answers, text-field keyboard isolation, playback
rejection, errors across property tabs, end-of-program timecodes, reframe setup,
model-readiness reporting and export while validation is pending.

The desktop suite passes 327 tests. Type checking, token-generation drift checks,
the eight token tests and the production renderer build pass. Device charts load
with Models & Device, reducing the main JavaScript chunk from 952 kB to 579 kB
(minified); Vite still reports its 500 kB advisory for that chunk.

Browser checks use the real React components with synthetic data and a local
video, in both themes at 1440×900 and 960×720. They exercise playback, caption
correction, search/filter/grid views, and review/edit/export navigation, and
check viewport and panel bounds. Native Tauri interaction and a new render
through the live worker fleet have not been rerun for this UI-only refresh.
