# Tauri + React for the desktop shell; system webview, board outside the React tree

The `desktop` crate is a Tauri shell hosting a React UI, as pinned in `openspec/config.yaml`,
and the way it was grilled deserves the trade-off in writing: alternatives were native toolkits
(Slint, GTK, Qt) and an embedded Chromium. Native toolkits lose the panel story — dozens of
virtualized lists, trees, and engine tables the specs demand — and embedded Chromium buys
rendering fidelity at the cost of shipping and patching a browser. The accepted shape:
system-provided webview only (WebKitGTK on Linux, no embedded Chromium or bundled browser),
Linux X11 and Wayland first-class per ADR-0005, and board rendering a canvas layer with a
deterministic layout API that never depends on React's DOM or render cycle — the board's budgets
(input latency, drag preview, 60 fps) must survive independently of UI re-renders. Consequence
everyone should know before filing a perf bug: WebKitGTK is the slowest webview in common use, so
the performance budgets in `performance-budgets` are acceptance budgets against it, not against
Chrome on the dev machine.
