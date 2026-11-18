# Linux desktop first-class; macOS best-effort; Windows out of scope

Spitz targets Linux desktops on X11 and Wayland as the tested, CI-gated platform; macOS runs
best-effort without promises because it is the development machine of several contributors; Windows
gets no requirements at all. This keeps panel virtualization, file watching, and input-latency
budgets honest against one windowing reality instead of a lowest-common-denominator UI, and matches
the audience in `openspec/config.yaml`. The rejected alternative (all-three tier-one) triples CI
cost to serve users the stated audience does not have. The UI toolkit itself is pinned in the
project context (Tauri shell, React UI) and gets its own ADR when the `desktop` crate is proposed;
this ADR fixes only the platform promise.
