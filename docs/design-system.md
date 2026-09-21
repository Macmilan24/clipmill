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

Settings → Appearance offers four workspace themes, each with light and dark
palettes. Paper & Ink is the default: warm surfaces, ink accents, compact corners,
underlined tabs and quiet dividers. Classic preserves the original sage light
palette and slate-blue dark palette. Warm Graphite uses stone neutrals; Soft Slate
uses cool neutrals and blue accents. All themes share the same fonts, spacing,
navigation and editing behavior. Error, warning and outbound-network indicators
retain their semantic meanings.

`packages/tokens/src/tokens.json` holds shared tokens and the original palettes.
`packages/tokens/src/workspace-themes.json` defines the theme catalog: stable IDs,
names, descriptions, component treatment, corner sizes and palette overrides.
Run `pnpm --filter @clipmill/tokens build` after changing either. The generated CSS
supplies component aliases and workspace styles; do not edit generated files.

Theme selection uses `data-workspace-theme`; light/dark uses `data-theme`.
`data-theme-chrome` selects shared component treatment (`soft` or `ink`). Stored
light/dark preferences remain valid. New or unrecognized theme preferences fall
back to Paper & Ink, and switching light/dark never changes the named theme.
Preferences are local to the device and do not change media or saved projects.

To add a theme, add a catalog entry with the same token keys and both palette
overrides. It appears in Settings automatically; miniature previews use the same
palette definitions as the app. Reuse a component treatment instead of creating
theme-specific copies of screens. Run token generation, contrast tests and visual
checks before publishing a new palette.

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
inspection. `screen` selects the page, `theme` accepts `light` or `dark`, and
`workspaceTheme` accepts a catalog ID such as `paper-ink` or `classic`. Preview
choices do not overwrite app preferences. The preview does not replace native
media or daemon integration checks.

Check the workspace at 1280 × 900 and 1024 × 768, in both appearances. Test navigation,
focus indicators, disclosures and disabled/error states. Token tests check text
contrast on the working surfaces and primary button states for every palette;
preference tests cover restoration, invalid values and unavailable storage. UI tests retain the real
selection, approval, edit and export contracts.
