# EKPI Platform — Complete Implementation Plan (Phases 2–10 + Deliveries E–G)

## Goal

Deliver the full Evidence-Grounded Enterprise Knowledge & Process Intelligence Platform per the Master Prompt (v1 + Addendum v2), closing every remaining gap from Round 2/3 MVP to a production-grade, auditable, local-first system that exceeds comparable market software.

## Current State (verified 2026-10-25)

- **Rounds 1–3 delivered**: Research DB (27 papers, 11 standards), tech selection, rule schema v0.1.1, process verification selection, Rust crates (`ekp_platform` + `ekp_eval`), determinism engine, governance web workbench, causal module, agent framework.
- **Tests**: 77 unit + 3 web integration + 8 integration = 88 pass; selftest 49/49; eval selftest 10/10; clippy zero warnings.
- **Hard gates H-1..H-6**: ALL OK.
- **Remaining**: Phase 2 (knowledge graph, evidence/claim layer), Phase 3 (vector/Tantivy retrieval, reranker), Phase 4 (BPMN, XES/OCEL, PM4Py), Phase 5 integration (causal into engine), Phase 6 integration (agents into engine), Phase 7 (API layer, full web UI), Phase 8 (50+ cases, dual κ, CI), Phase 9 (security, WORM, tenant isolation), Phase 10 (deployment docs), Deliveries E–G.

## Deliverables & Acceptance Criteria

### Delivery A (Research Report) — PARTIALLY DONE
- [ ] Expand papers.csv to 40+ entries (add GraphRAG-Bench, CausalGraph2LLM, aProCheCk, RAPTOR, HippoRAG, Self-RAG, Corrective RAG, PageIndex)
- [ ] Add independent reproduction status column verification
- [ ] Write `docs/research-gaps.md` listing unverified methods and research-only branches

### Delivery B (System Design) — PARTIALLY DONE
- [ ] Architecture diagram (Mermaid) in `docs/architecture.md`
- [ ] Full data model DDL in `docs/data-model.md` (PostgreSQL schema for all 5 layers)
- [ ] API contract (OpenAPI 3.1) in `docs/api-contract.yaml`
- [ ] Permission matrix in `docs/permissions.md`

### Delivery C (MVP) — DONE (Round 2)
- Already complete: document import, version tree, clause anchors, hybrid retrieval, rule validation, process verification, snapshot replay.

### Delivery D (Process Verification Prototype) — DONE (Round 2)
- Already complete: Petri net token replay, 4-class conformance, forbidden pairs, loop policy.

### Delivery E (Evaluation Framework) — IN PROGRESS
- [ ] Expand `eval/cases/seed_cases.jsonl` from 9 → 50+ cases covering all 15 master-prompt categories
- [ ] Dual annotation: add `annotator_a`/`annotator_b` fields, compute Cohen's κ
- [ ] CI integration: GitHub Actions workflow (`.github/workflows/ci.yml`) running clippy + test + selftest on both crates
- [ ] A-G ablation formal report with failure case → regression test entries
- [ ] Risk-coverage curve computation in `ekp_eval::metrics`
- [ ] CausalGraph2LLM-type query set as regression test

### Delivery F (Deployment Docs) — NOT DONE
- [ ] `docs/deployment/install.md` — installation guide
- [ ] `docs/deployment/config.md` — configuration reference
- [ ] `docs/deployment/backup.md` — backup & disaster recovery
- [ ] `docs/deployment/security.md` — security hardening
- [ ] `docs/deployment/monitoring.md` — observability setup
- [ ] `docs/deployment/troubleshooting.md` — troubleshooting guide

### Delivery G (Research Plan) — NOT DONE
- [ ] `docs/research-plan.md` — next batch of methods, hypotheses, success criteria, cost estimates

## Implementation Phases (remaining work)

### Phase 2 Completion: Knowledge Graph & Evidence/Claim Layer

**2a. Knowledge Graph Module** (`platform/src/knowledge_graph.rs`)
- Typed entities: Standard, Clause, Requirement, SOP, Department, Role, Activity, Form, Evidence, Risk, Control, Exception, Event, Decision, Finding, CAPA
- Typed relations: contains, references, applies_to, requires, prohibits, precedes, triggers, supersedes, exception_of, verifies, produces_evidence, causes, conflicts_with
- Each relation: source, scope, version, extraction_method, verification_status
- LLM-extracted relations default to `unverified` (never auto-trusted)
- Tests: entity CRUD, relation validation, provenance tracking, unverified default

**2b. Evidence & Claim Layer** (`platform/src/evidence.rs`)
- Claim struct: id, text, claim_type (fact/rule_derived/event_derived/statistical/causal/hypothesis/undetermined), supporting evidence, contradicting evidence, scope, reasoning steps, confidence, human_review_status
- Evidence struct: id, source (doc_id, version, section, page, paragraph), verbatim_text, hash, access_level, timestamp
- Claim-Evidence linking with support/contradiction polarity
- Tests: claim creation, evidence linking, polarity, confidence bounds

### Phase 3 Completion: Hybrid Retrieval Engine

**3a. Vector Search** (`platform/src/vector_search.rs`)
- Trait `VectorIndex` with `build`, `search`, `add`, `remove`
- In-memory implementation (brute-force cosine) for MVP
- pgvector adapter stub (feature-gated, R4)
- Tests: index build, top-k retrieval, similarity threshold

**3b. Full-Text Search (Tantivy)** (`platform/src/fulltext.rs`)
- Trait `FullTextIndex` with `build`, `search`, `add_document`
- In-memory BM25 implementation for MVP
- Tantivy adapter stub (feature-gated, R4)
- Tests: BM25 scoring, term matching, field filtering

**3c. Unified Retrieval Interface** (`platform/src/retrieval_v2.rs`)
- `RetrievalEngine` composing: document tree navigation, vector search, BM25, hybrid fusion (RRF), reranker, knowledge graph multi-hop, citation anchoring, temporal/version/permission filtering
- Strategy selection by task type
- All results carry source location + version
- Tests: fusion scoring, strategy selection, citation integrity

**3d. Reranker** (`platform/src/reranker.rs`)
- Trait `Reranker` with `rerank(query, candidates) -> ranked`
- Cross-encoder stub (feature-gated)
- Score-fusion reranker (MVP)
- Tests: reranking order, score normalization

### Phase 4 Completion: Process Verification

**4a. BPMN Parser** (`platform/src/bpmn.rs`)
- Parse BPMN 2.0 XML into ProcessModel
- Extract: tasks, gateways (exclusive/parallel/inclusive), events, sequence flows, subprocesses
- Tests: parse valid BPMN, extract elements, handle malformed XML

**4b. XES/OCEL Import** (`platform/src/xes_import.rs`)
- XES (IEEE 1849-2023) parser → EventLog
- OCEL 2.0 parser for multi-entity cases
- 6 quality gates (GR-1..GR-6): entity resolution, out-of-order, duplicates, clock skew, event completeness, role mapping
- Tests: parse XES, quality gate validation, three-state events (observed/assumed/unconfirmed)

**4c. PM4Py Sidecar** (`platform/src/pm4py_sidecar.rs`)
- Hash-based artifact contract: input hash → process model, output hash → conformance report
- Sidecar invocation via subprocess with timeout
- Tests: contract validation, timeout handling, hash integrity

### Phase 5 Integration: Causal into Engine

**5a. Wire causal module into engine**
- Add causal analysis step to `Platform::decide()` after process replay
- For each rule with causal implications, run `CausalAnalyzer::analyze()`
- Include causal verdict in EngineOutcome (new field: `causal_results: Vec<CausalAnalysisResult>`)
- Tests: causal step in decide(), cannot_identify doesn't degrade verdict

### Phase 6 Integration: Agents into Engine

**6a. Wire agent framework into engine**
- Add `AgentOrchestrator` to Platform
- Implement iterative loop: task analysis → retrieval → claim building → evidence verification → gap discovery → supplemental retrieval → rule check → final validation
- Stopping conditions per A10
- Tests: loop execution, stopping conditions, iteration tracking

### Phase 7 Completion: Full Platform

**7a. API Layer** (`platform/src/api/`)
- axum router with REST endpoints:
  - `POST /api/v1/documents` — import document
  - `GET /api/v1/documents/{id}/versions` — list versions
  - `POST /api/v1/rules` — create rule
  - `GET /api/v1/rules` — list rules (filter by status, scope)
  - `POST /api/v1/cases/{id}/decide` — adjudicate case
  - `GET /api/v1/cases/{id}/snapshot` — get decision snapshot
  - `POST /api/v1/processes/{id}/conformance` — run conformance check
  - `GET /api/v1/coverage` — coverage report
- OpenAPI 3.1 spec generation
- Tests: endpoint integration tests

**7b. Web UI Expansion** (extend `governance_web.rs`)
- Document management & version comparison
- Knowledge tree browser
- SOP process flow diagram (SVG, CSS-first)
- Case status & event timeline
- Audit findings & risk dashboard
- Evidence source & verbatim highlight
- Rule editor & approval workflow
- Multi-document diff comparison
- Agent execution trace
- Uncertainty & unresolved issues list
- Report export
- System evaluation & error analysis

**7c. Security Hardening**
- WORM-style evidence storage with hash chain
- Tenant isolation (row-level security in PostgreSQL)
- Permission control (RBAC)
- Prompt injection defense (already in SEC-001 test)
- Agent operation whitelist
- Human approval for high-risk operations
- Audit event log (append-only, SHA-256 chain)
- Tests: permission enforcement, injection resistance, audit integrity

### Phase 8 Completion: Evaluation Framework

**8a. Expand seed cases to 50+**
- Cover all 15 master-prompt categories:
  1. Precise clause retrieval (5 cases)
  2. Cross-chapter/cross-document multi-hop (5 cases)
  3. Table/appendix location (3 cases)
  4. Version conflict (4 cases)
  5. Negation/exception clauses (4 cases)
  6. Precondition/order errors (4 cases)
  7. Legal/illegal loops (3 cases)
  8. Missing evidence (4 cases)
  9. Contradictory sources (3 cases)
  10. Causal vs correlational (3 cases)
  11. Long process/parallel branches (3 cases)
  12. Document parse failure (2 cases)
  13. Insufficient permissions (2 cases)
  14. Malicious document/prompt injection (3 cases)
  15. Undetermined/escalate (2 cases)
- Total: 50+ cases

**8b. Dual annotation & Cohen's κ**
- Add `annotator_a`, `annotator_b` fields to Case
- Compute Cohen's κ in metrics
- Report inter-annotator agreement

**8c. CI Integration**
- `.github/workflows/ci.yml` with jobs: build, test, clippy, security, mtrix, release
- Matrix: Windows + Linux
- Cache cargo registry

**8d. A-G Ablation Formal Report**
- Run all 7 ablation configurations
- Generate formal report with failure case → regression test entries
- Risk-coverage curve computation

### Phase 9 Completion: Security & Governance

**9a. WORM Evidence Storage** (`platform/src/worm.rs`)
- Append-only storage with SHA-256 hash chain
- Tamper detection
- Tests: append, verify chain, detect tampering

**9b. Tenant Isolation** (`platform/src/tenant.rs`)
- Tenant context in all operations
- Row-level filtering
- Tests: isolation verification

**9c. Audit Event Log** (`platform/src/audit.rs`)
- Append-only event log with hash chain
- Event types: document_import, rule_change, case_decision, process_conformance, evidence_review
- Tests: append, verify, tamper detection

### Phase 10 Completion: Deployment & Documentation

**10a. Deployment Documentation** (Delivery F)
- All 6 deployment docs listed above

**10b. Local-First Workflow**
- Fixed local check scripts: `scripts/check.ps1`, `scripts/check.sh`
- Test-driven implementation slices (each feature has tests first)
- Code review checklist: `docs/code-review-checklist.md`
- Quantifiable success metrics dashboard

**10c. Remote Collaboration Upgrade Criteria**
- Document in `docs/remote-collaboration-criteria.md`:
  - Team size ≥ 3 developers
  - Need for CI/CD pipeline
  - Multi-tenant requirement
  - Regulatory compliance requiring audit trail
  - Performance requirements exceeding single machine

## Validation Plan

### Per-Module Validation
Each new module must have:
1. Unit tests (≥ 5 per module, covering happy path + edge cases)
2. Integration tests (where applicable)
3. Clippy zero warnings
4. `cargo fmt -- --check` clean
5. Selftest assertion (where applicable)

### Hard Gates (all must pass)
- H-1: Forbidden transition release = 0
- H-2: Miss rate = 0
- H-3: Insufficient-evidence judged comply = 0
- H-4: Citation anchor resolvable = 100%
- H-5: Injection ignored = 100%
- H-6: Replay consistency = 100%
- H-7 (new): Causal cannot_identify rate = 100% when data insufficient
- H-8 (new): Agent stopping condition triggered when no new evidence
- H-9 (new): WORM hash chain integrity = 100%
- H-10 (new): Tenant isolation violation = 0

### Success Metrics
- Accuracy: verdict accuracy ≥ 0.95 on seed cases
- Processing time: P95 < 500ms per case
- Manual handoff rate: < 20% of cases
- Error rate: < 1% wrong_allow
- Cost: < $0.01 per case (LLM calls)
- Completion rate: 100% of cases get a verdict (including abstain/escalate)

## Implementation Order (priority)

1. Phase 5 Integration (causal into engine) — highest value, lowest risk
2. Phase 6 Integration (agents into engine) — completes iterative loop
3. Phase 3 Completion (vector + Tantivy + reranker + unified retrieval) — improves accuracy
4. Phase 4 Completion (BPMN + XES/OCEL + PM4Py) — completes process verification
5. Phase 2 Completion (knowledge graph + evidence/claim) — enriches knowledge model
6. Phase 7 (API + web UI + security) — makes it usable
7. Phase 8 (50+ cases + CI + ablation report) — validates everything
8. Phase 9 (WORM + tenant + audit) — hardens security
9. Phase 10 (deployment docs + local-first workflow) — delivers product
10. Deliveries E–G (reports + docs) — finalizes deliverables

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Scope creep from master prompt breadth | Strict phase ordering; each phase has clear acceptance criteria |
| Encoding corruption (recurring) | Use `cargo fmt` after every edit; never use PowerShell Set-Content without -Encoding UTF8 |
| Test count inflation | Only count tests that verify real behavior, not tautologies |
| Performance regression | Benchmark each phase; P95 < 500ms gate |
| Security gaps | Security review before each phase completion |

## Open Questions for User

1. Should we prioritize the API layer (Phase 7a) before or after the retrieval improvements (Phase 3)?
2. Do you want the PM4Py sidecar to be a hard dependency or optional (feature-gated)?
3. What is the target deployment environment (Windows-only, Linux, or both)?
4. Should we implement the knowledge graph as a separate crate or within platform?
5. What is the minimum viable case count for the 50+ expansion (exact number)?
