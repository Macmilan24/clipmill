# Results, the Inspector, and the edit director (W23)

Three things that are really one: a board of what the ranking believes in, one
clip opened, and the step that turns a decision into an edit.

## Nothing here decides anything

The board and the Inspector display measurements. Every number was published by
a stage, and the interface's whole job is to show it without becoming a second
opinion on it.

That has consequences worth naming. The cohort is shown in the ranking's own
order rather than re-sorted, because the order is a decision the ranking made
and an interface that re-made it would be a ranking nobody can see. An axis
nothing measured shows **why** nothing measured it rather than a zero, because a
zero reads as a measurement of badness. Uncertainty is three words — strong,
promising, needs review — rather than a shading of the score, which is what the
book asks for and the contract already carries. And the shortfall is stated
rather than padded away: a recording holding three good moments returns three
and says so, because the fourth would be a clip the system does not believe in.

Filtering is client-side. Every row was already fetched to draw the summary, so
asking the daemon again could only produce what is on screen.

## The preview is real, and it is read-only

The Inspector shows the project's own proxy, served over the media protocol,
seeked to the clip's window and cropped to 9:16 along the path the reframe
solver proposes. The solve **writes nothing**, which is what makes it safe to
ask again every time the selection moves.

Two details matter. The crop is a transform over the real frame rather than a
rectangle drawn beside it: a guide shows where the camera is pointing, and this
shows what the camera sees, which is the question an editor is asking. And the
caption overlay is the **burned-in grouping of the document the director
produced**, so what a viewer sees over the proxy is what the encoder will draw —
which is only possible because the Edit IR carries both groupings.

The editor's player is W24's. This is deliberately not it.

## The director

`clipmill-director`. Assembly, not judgement: discovery proposed the span, the
boundary optimizer chose the cut, the caption engine decided the line breaks,
the reframe gate decided whether a face earned the frame. The director reads
those and writes a document.

The gate's headline is therefore a **golden**. The same candidate, boundary and
evidence must produce the same bytes, because an editor who approves the same
clip twice and gets two different edits has been told the tool is guessing.

- A boundary is **snapped to the lattice** rather than taken as given. No amount
  of care with a mouse gets a person within a frame of a sentence edge, and a
  boundary a few frames off one is the mid-word cut the optimizer exists to
  avoid. The edge that was dragged is honoured; the other moves only as far as
  it must, because somebody dragging the start is answering "where should this
  begin" and silently moving the start would answer a question they did not ask.
- A boundary that did **not** come from the snap is checked against the lattice
  anyway and refused rather than rounded — one arriving over IPC has been
  through a process the director does not control.
- A camera move is proposed only when the reframe gate says a face earned the
  frame. Otherwise the layout is `Fit` and the rationale carries the gate's own
  reason rather than replacing it with silence.
- Both caption groupings are written, and the golden checks they hold the same
  words. That property is the one the caption engine's whole shape exists to
  guarantee, so it is the one a bug here would quietly undo.

Captions are derived for the **span** rather than read from a published caption
artifact. A cue may not cross the edge of its window, so cues segmented over the
whole recording are not the cues this clip should carry — and publishing an
artifact per boundary would mint a cache entry every time somebody nudged a
handle.

## Decisions are durable

Rejecting a clip is small work done a dozen times a session, which is exactly
why losing it is worse than losing something large: nobody remembers what they
rejected, so nobody can redo it. Decisions live in the daemon's store, one row
per candidate, and the gate kills the daemon between the write and the read.

Keyed by candidate, not by clip. The candidate id survives a re-run of ranking
over the same evidence, so re-analyzing a recording does not quietly reset
somebody's rejections; it does not survive re-analysis that changed the
evidence, and that is correct, because those are different candidates whatever
they are called.

Approving is the only decision that directs. Keeping and rejecting record an
opinion and nothing else.

## What this phase does not do

- **The boundary strip shows the lattice; it does not drag yet.** The snapping
  arithmetic is done and tested and the RPC accepts a hand-set pair, but the
  handle a person grabs is the editor's surface, and the alternative is one
  click. Shipping a drag that only the daemon could interpret would be a control
  whose behaviour lives somewhere the user cannot see.
- **One recording at a time.** The board shows the newest analyzed source of the
  newest project and says which. Phase 1 has no project picker, and inventing
  one here would be a navigation surface the design does not have.
- **No caption overlay before approval.** The burned-in grouping lives in the
  document approving creates; drawing cues before that would show captions the
  render has not been asked to draw.

## Gate

- `just gate-inspector` — the director's goldens and the boundary swap; the
  lattice arithmetic on its own, because it is where a person's hand meets the
  search's rules; a decision that survives the daemon dying between the write
  and the read; and the board's joins, checked without mounting anything.

It needs no pinned media, so it runs in the plain `inspector` CI job.
