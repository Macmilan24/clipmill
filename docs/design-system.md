# Design system

ClipMill uses DM Sans for interface text and IBM Plex Mono for timecodes and
technical values. Both fonts are bundled with the app and work offline. The
caption-rendering font is independent and remains identical in preview and export.

DM Sans and IBM Plex Mono retain their SIL Open Font License 1.1 licenses. Complete
upstream copyright and license notices are kept in
`apps/desktop/public/licenses/dm-sans-OFL-1.1.txt` and
`apps/desktop/public/licenses/ibm-plex-mono-OFL-1.1.txt`, which are copied into the
production bundle's `licenses/` directory with the other public assets. Preserve
these notices when updating or redistributing the fonts.

The light theme uses warm paper surfaces and a sage accent. The dark theme uses
cool charcoal surfaces and a slate-blue accent. Each palette defines its own
accent foreground, hover, pressed, selection and focus colors. Error, warning and
outbound-network indicators retain their semantic meanings.

`packages/tokens/src/tokens.json` is the source of truth. Run
`pnpm --filter @clipmill/tokens build` after changing it. The generated CSS supplies
both the component aliases and the workspace styles; do not edit generated files.

## Interface hierarchy

- Video previews and direct editing take priority over technical explanations.
- A compact activity rail, contact-sheet media grids and monospaced transports
  establish the editing workspace. Keep panels functional and leave footage
  room to breathe.
- A page has one clear primary action, with secondary actions grouped nearby.
- Scores, runtime measurements and successful checks may open on demand. Failures,
  pending saves, required consent and recovery actions must remain visible.
- Editor tools occupy one contextual panel. Focus preview hides surrounding
  controls without remounting the player or resetting the playhead.
- Use quiet dividers, opaque surfaces, compact corners and normal sentence case.
  Avoid decorative gradients, artificial statistics and unnecessary nested cards.

## Verification

Use `apps/desktop/preview.html` through Vite for synthetic, non-destructive screen
inspection. `screen` selects the page and `theme` accepts `light` or `dark`. The
preview does not replace native media or daemon integration checks.

Check the workspace at 1280 × 900 and 1024 × 768, in both themes. Test navigation,
focus indicators, disclosures and disabled/error states. Token tests check text
contrast on the working surfaces and primary buttons; UI tests retain the real
selection, approval, edit and export contracts.
