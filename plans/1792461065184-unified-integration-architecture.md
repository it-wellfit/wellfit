# Unified Integration Architecture Plan

> **Disposition (2026-11-02): Proposal only; not an approved cross-project architecture.** The listed Kafka/PostgreSQL/Redis/Kubernetes choices and “confirmed” scope are not evidenced by current project contracts. Reconcile independently with each project owner before implementation; do not treat historical test counts or crate lists below as current.

Plan path: `C:\Users\stephanie.huang\.local\share\kilo\plans\1792461065184-unified-integration-architecture.md`

## Decisions (confirmed with user)

| Decision | Result |
|----------|--------|
| Scope | Three Rust workspaces (Wellfit, SyncMesh, ekpi-platform) integrated; each independently evolves, scales, reliable, performant, stable; can each attach external modules/models |
| Deployment | Native binary per workspace (sc.exe/systemd/launchd), no Docker; independent deploy + API gateway interop; k3s + Service Mesh |
| Event backbone | Apache Kafka (high throughput, replayable, durable); PostgreSQL source of truth; Redis cache; NATS JetStream/RabbitMQ optional per workload |
| E-commerce integration | Standardized adapter + rules engine + intelligent engine + unified data model (UDM); hot-pluggable; supports momo/PChome/Cyberbiz + future platforms |
| Approval state machine | Dynamic beyond 5 levels, multi-urgency, conditional branching, timeout escalation, periodic reminders/cleanup; Agent decides what needs approval; permission control for paid actions; balances traditional + future evolution |

## Existing code to reuse

- `Wellfit/` — 5-crate workspace (core/shopee/reports/mtrix/cli), Release Gate state machine, ledger, 139 tests passing, clippy zero warnings
- `SyncMesh/` — 4-crate workspace (core/net/storage/cli), Ed25519 identity, CRDT version vectors, conflict resolution, SQLite content-addressed storage
- `ekpi-platform/platform/` — governance/eval crate, deterministic rules, evidence retrieval, decision snapshots

## Ordered implementation tasks

### Phase 1 — Infrastructure & core interop
1. Deploy Kafka + PostgreSQL + Redis on k3s + Service Mesh
2. Build API gateway with mTLS, auth, rate limiting
3. Deploy each workspace as native binary (sc.exe/systemd/launchd)
4. Implement Kafka publish/subscribe so the three workspaces interop via events
5. Create shared metadata schema in PostgreSQL (approval state, audit trail, adapter config)

Validation: 3 binaries start/stop independently; Kafka messages flow between them; PostgreSQL shared read/write; mTLS passes

### Phase 2 — E-commerce adapters & UDM
6. Define Unified Data Model (UDM) for orders/inventory/products
7. Implement `PlatformAdapter` trait + `AdapterRegistry`
8. Build momo adapter (REST + Webhook)
9. Build PChome adapter (REST + Webhook)
10. Build Cyberbiz adapter (REST + Webhook)
11. Implement rules engine + data format derivation (bidirectional UDM <-> platform format)
12. Verify hot-pluggable adapter loading without restart

Validation: 3 platforms sync orders/inventory simultaneously; UDM conversion lossless; new adapter hot-loadable

### Phase 3 — Approval state machine
13. Implement approval state machine: Draft → PendingReview → Approved → Executing → Completed/Failed/Rejected
14. Multi-level approval with per-level approver, conditions, timeout
15. Multi-urgency SLA + escalation paths
16. Conditional branching by amount/platform/data sensitivity/anomaly history
17. Timeout escalation + periodic reminders/cleanup
18. Agent approval-judgment module (auto-approve low risk, force human on high risk/anomaly)
19. Permission control for paid actions and sensitive data (Agent cannot bypass)
20. Audit trail + compliance reports

Validation: End-to-end approval executes; timeout escalation works; Agent auto-approves low risk; audit trail complete

### Phase 4 — Intelligent engine & dynamic optimization
21. Intelligent engine: anomaly pattern recognition, risk grading
22. Dynamic scheduling: monitor load, adjust worker count and batch size
23. Exception monitoring & alerting: circuit breaker, exponential backoff, dead-letter handling
24. Data replay + disaster recovery drill
25. Query/settings/recovery optimization: Redis cache, DB indexes, partitioning

Validation: Anomaly auto-detected & graded; dynamic scheduling reduces latency; RTO < 5 min, RPO = 0; P99 query < 100ms

### Phase 5 — Hardening & production
26. Security penetration testing + fixes
27. Load testing (>50,000 msg/s)
28. Production deployment (k3s + Service Mesh)
29. Disaster recovery drill
30. Monitoring dashboard + alerting
31. Documentation + operations manual

Validation: Security checks pass; throughput target met; DR drill succeeds; dashboard operational

## Risks

| Risk | Level | Mitigation |
|------|-------|------------|
| Kafka single point of failure | High | Multi-broker, ISR replication, monitoring alerts |
| Adapter API changes break integration | Medium | Versioned adapters, compatibility tests, rollback |
| Agent misapproves high-risk action | High | Dual verification, high-risk forces human, audit log |
| DB connection exhaustion | Medium | Connection pool limits, circuit breaker, auto-scale |
| Cross-platform data format inconsistency | Medium | UDM conversion layer, format validation, anomaly isolation |
| Approval SLA too slow | Medium | Agent auto-approve low risk, SLA monitoring, timeout escalation |

## Open questions (mark before execution)

1. Intelligent engine: rules-first in Phase 3, or add ML (Isolation Forest) in Phase 4? Recommended: Phase 3 rules, Phase 4 ML.
2. Kafka vs NATS JetStream: Phase 1 uses Kafka; Phase 4 evaluates adding NATS per workload.
3. PostgreSQL: primary-standby in Phase 1; evaluate sharding in Phase 5.
4. k3s vs other tooling: Phase 1 uses k3s (lightweight, suits on-prem/edge).
5. Shared `integration-core` crate across the three workspaces? Recommended: yes, containing UDM, event model, adapter trait.

## Related files

- `Wellfit/Cargo.toml`, `Wellfit/AGENTS.md` — MTRIX core, Release Gate, build/test rules
- `SyncMesh/Cargo.toml` — identity/storage/net, Ed25519, CRDT
- `ekpi-platform/platform/Cargo.toml`, `ekpi-platform/AGENTS.md` — governance, quality gates
