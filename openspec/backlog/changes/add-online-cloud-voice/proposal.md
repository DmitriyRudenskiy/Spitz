# Change: add-online-cloud-voice

## Why

The suite completes with its service layer: crowd-sourced position analysis, cloud storage, an online play service, voice interaction, and the account/licensing system that gates premium features. These turn the desktop suite into a connected product.

## What Changes

- Implements the baseline capabilities `crowd-analysis`, `cloud-sync`, `online-play`, `voice-interface`, `accounts-licensing` (requirements already seeded in `openspec/specs/`).
- Adds one delta capability: `service-reliability` — server SLOs, matchmaking latency, durability, and abuse controls — see `specs/service-reliability/spec.md` in this change.
- Ships the server components (Rust/axum, PostgreSQL, Redis, WebSocket) and connects the M2 live-book client and offline queue to them.

## Impact

- Affected specs: five new baseline capabilities listed above; new capability service-reliability (delta).
- Affected code: new server crates; clients gain sessions, sync, voice input/output; the placeholder `app/server` crate from M1 is implemented.
