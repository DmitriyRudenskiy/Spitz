# Change: add-ui-enhancements

## Why

The classic desktop chess suites this product is modelled on stay beloved because their shell is fully user-shaped — every element toggleable, a customizable toolbar, context menus everywhere, a Tip of the Day, a built-in manual — and because their criticized weaknesses (unreadable fonts, unclear themes, no discoverability for advanced tools) are all shell concerns that can be fixed. The app-shell capability covers docking, themes, and shortcuts; this change adds the missing shell behaviors that make an expert workflow fast.

## What Changes

- One new capability: `classic-ui` — Tip of the Day, customizable toolbar (add/remove/reorder, icon sizes), per-element visibility toggles, context menus on all major panels, global and per-panel font scaling, detached board windows, built-in manual viewer, and a theme preview for board and piece sets.

## Impact

- Affected specs: new capability classic-ui.
- Affected code: `app/ui` shell layer only; toolbar/element registry, font-scale service, detached-window controller, manual-content bundle, theme-preview component. No engine or core changes.
