# Design: UI Enhancements (classic shell)

## Context

The app-shell capability already provides docking, light/dark themes with accent, settings import/export, multi-board tabs, i18n, and shortcut remapping. This change adds the classic-shell behaviors the reference product is known for that app-shell does not cover, while deliberately fixing the points its users criticize.

## Key Decisions

1. **Element registry**: every panel and toolbar registers a stable id; visibility toggles, context menus, and toolbar customization all read from this one registry — one source of truth, no per-panel special cases.
2. **Font scaling as a service**: a global scale factor plus per-panel overrides, applied through CSS custom properties; OS DPI scaling is respected; layouts re-flow without docking glitches.
3. **Detached board windows**: a board-window controller duplicating the in-tab board state to a native top-level window (Tauri multiwindow), synchronized both ways; used for multi-monitor analysis setups.
4. **Manual as content bundle**: the manual ships as structured Markdown rendered in an in-app viewer with TOC and full-text search; features added by later changes (e.g., selected-moves analysis) get dedicated sections — fixing the reference's discoverability flaw.
5. **Theme preview**: the board/piece theme chooser renders live thumbnails from the actual rendering pipeline, so users see exactly what they will get — answering the "unclear themes" criticism.
6. **Tip of the Day** content packs are versioned with the app and include the tools added by later changes; the dialog is disabled persistently via settings.

## Risks / Trade-offs

- Tauri multiwindow state sync adds complexity; the detached window is a projection of one source of truth, so conflicts are impossible by construction.
- Font scaling can break dense engine-line panels; minimum sizes per panel type are enforced and tested.

## Naming (trademark-clean)

Tip of the Day, toolbar customization, and manual viewer are generic UI terms used as-is; no reference-product names appear.
