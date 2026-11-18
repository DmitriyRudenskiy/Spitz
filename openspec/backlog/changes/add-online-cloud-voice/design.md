# Design: Online, Cloud and Voice

## Context

M1 and M2 delivered a complete desktop suite. This milestone adds the service layer: a Rust backend, cloud sync, crowd analysis, online play, voice interaction, and accounts/licensing. The M2 live-book client and offline queue plug into these services.

## Key Decisions

1. **One modular monolith**: a single Rust service (axum/tokio) with internal crates per domain; PostgreSQL for durable data, Redis for presence and matchmaking; WebSocket for play and broadcasts; REST for sync and CRUD. Scaling out happens later behind the same contract.
2. **Ratings and matchmaking**: Glicko-2 per time-control pool; the rating window starts at ±200 and expands with wait time; provisional (high RD) players get wider windows.
3. **Anti-cheat**: engine-match heuristics produce flags for human review only; automatic bans are prohibited (see online-play spec). Moderator tooling ships with the service.
4. **Crowd analysis store**: position-hash sharding in PostgreSQL with a cache layer; contributions deduplicated by (position, depth bucket) with a minimum-depth gate; contributions are pseudonymous.
5. **Sync**: per-entity last-writer-wins with tombstones; losing versions retained 30 days; the M2 offline queue feeds the same idempotent ingestion endpoint; TLS 1.3 in transit, encryption at rest, 2 GB default quota.
6. **Voice**: local ASR with grammar-constrained decoding over the chess vocabulary (EN/DE/RU) plus local TTS; server fallback optional; audio is never persisted without explicit consent.
7. **Accounts and entitlements**: email + OAuth registration, optional TOTP MFA; an entitlement service drives feature flags; one-time purchase unlocks local features permanently, premium unlocks server features; offline grace 30 days.

## Risks / Trade-offs

- Server abuse (chat flooding, cheating): rate limits and human-review workflow from day one; no ban automation.
- Load: matchmaking and spectating are the hot paths; load test at 10,000 concurrent users is part of the milestone (service-reliability delta).
- Voice quality across languages: grammar-constrained decoding keeps the recognition space small; local models are downloadable and the latency budget (≤1.5 s) is testable.

## Migration / Rollout

The M2 offline queue already stores mutations; the sync endpoint accepts the same payloads, so no client data migration is needed. Entitlement flags default to "trial" until a purchase backend is configured.
