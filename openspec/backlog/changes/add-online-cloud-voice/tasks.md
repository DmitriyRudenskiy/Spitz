# Tasks: add-online-cloud-voice

## 1. accounts-licensing

- [ ] 1.1 Registration/login: email+password plus 2 OAuth providers; sessions with refresh tokens; optional TOTP MFA
- [ ] 1.2 Entitlement service and purchase flow: one-time unlock of local features, premium for server features, configurable trials
- [ ] 1.3 Device management: max 3 concurrent activations with self-service deactivation; 30-day offline grace
- [ ] 1.4 Privacy: data export and account deletion honored within 30 days; consent gate for voice audio storage

## 2. cloud-sync

- [ ] 2.1 Sync API for games, training progress, repertoire, settings subset; per-device selective toggles
- [ ] 2.2 Conflict resolution: per-entity last-writer-wins with tombstones; losing versions retained 30 days
- [ ] 2.3 Offline queue integration (idempotent ingestion); quota 2 GB with usage display and warnings
- [ ] 2.4 TLS 1.3, encryption at rest, key management

## 3. crowd-analysis

- [ ] 3.1 Query API with metadata (eval, depth, engine class, contribution count, last update) and cache layer
- [ ] 3.2 Contribution API: opt-in, dedup by (position, depth bucket), minimum-depth gate ≥18, pseudonymous identity
- [ ] 3.3 Blunder verification flow comparing played moves against crowd consensus with configurable threshold

## 4. online-play

- [ ] 4.1 WebSocket server: sessions, reconnect with full game-state restore within 60 s
- [ ] 4.2 Rated pools with Glicko-2 and provisional handling; casual games and challenges with shareable links
- [ ] 4.3 Matchmaking with expanding ±200 window; P95 pairing ≤10 s at 10,000 concurrent seekers (load test)
- [ ] 4.4 Spectating and broadcasts with kibitz evaluation and spectator chat
- [ ] 4.5 Chat moderation: mute/report/block, moderator tools, per-account rate limits
- [ ] 4.6 Anti-cheat heuristics producing flags for human review only

## 5. voice-interface

- [ ] 5.1 Local ASR with grammar-constrained decoding over chess vocabulary (EN/DE/RU); downloadable models
- [ ] 5.2 Push-to-talk and continuous modes with audible cue; confidence threshold and confirmation for ambiguous parses
- [ ] 5.3 TTS commentary of moves and evaluation events; configurable rate/pitch/voice
- [ ] 5.4 Latency budget: recognition ≤1.5 s round trip on reference hardware

## 6. service-reliability delta

- [ ] 6.1 SLO dashboards and monthly availability report (99.5% target)
- [ ] 6.2 Load test harness: 10,000 concurrent users on matchmaking and spectating paths
- [ ] 6.3 Durability: write-ahead logged rated results verified by a crash-recovery test
- [ ] 6.4 Rate limiting and abuse-control rollout with incident runbook
