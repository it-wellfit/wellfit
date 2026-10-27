# Complete Implementation Plan — All Remaining Items

## Goal

Deliver every remaining item from the Master Prompt (v1 + Addendum v2) and the Research Framework, closing all gaps from current MVP to a production-grade, auditable, local-first enterprise knowledge & process intelligence platform.

## Current State (verified 2026-10-25)

### Completed
- **Rounds 1–3**: Research DB (28 papers, 11 standards), tech selection, rule schema v0.1.1, process verification selection, Rust crates, determinism engine, governance web, causal module, agent framework, knowledge graph module, evidence/claim layer, research framework doc.
- **Tests**: 70 unit + 3 web integration + 8 integration = 81 pass; clippy zero warnings (platform + eval).
- **Research framework**: `docs/research-framework-v1.md` (six-layer architecture, technology comparison matrices, audit-specific evaluation set, benchmark experiments, maturity classification, three-way separation, evidence chain preservation).

### Remaining
- Phase 3 Completion: Vector search, Tantivy full-text, reranker, unified retrieval engine
- Phase 4 Completion: BPMN parser, XES/OCEL import, PM4Py sidecar contract
- Phase 6 Integration: Agent framework into engine (iterative loop)
- Phase 7a: API layer (axum REST endpoints, OpenAPI 3.1)
- Phase 7b: Web UI expansion (document management, knowledge tree, process flow, case timeline, audit dashboard, evidence highlight, rule editor, diff comparison, agent trace, uncertainty list, report export, evaluation analysis)
- Phase 8: 50+ seed cases, dual annotation κ, CI integration, A-G ablation formal report
- Phase 9: WORM evidence storage, tenant isolation, audit event log
- Phase 10: Deployment docs (6 docs), local-first workflow (check scripts, code review checklist, success metrics), remote collaboration upgrade criteria
- Deliveries E-G: Ablation formal report, deployment docs, research plan
- Remove test_engine.rs (leftover from debugging)

## Implementation Strategy

### Priority 1: High-Value, Low-Risk

#### 1.1 Phase 6 Integration: Agents into Engine
- Add `AgentOrchestrator` to Platform
- Implement iterative loop: task analysis → retrieval → claim building → evidence verification → gap discovery → supplemental retrieval → rule check → final validation
- Stopping conditions per A10 (no new evidence/claims/rules → stop)
- Tests: loop execution, stopping conditions, iteration tracking

#### 1.2 Phase 3: Vector Search + Full-Text + Reranker + Unified Retrieval
- `platform/src/vector_search.rs` — Trait VectorIndex + in-memory brute-force cosine
- `platform/src/fulltext.rs` — Trait FullTextIndex + in-memory BM25
- `platform/src/reranker.rs` — Trait Reranker + score-fusion
- `platform/src/retrieval_v2.rs` — Unified RetrievalEngine composing all strategies
- Tests: ≥4 per module

### Priority 2: Medium-Value, Medium-Risk

#### 2.1 Phase 4: BPMN + XES/OCEL + PM4Py
- `platform/src/bpmn.rs` — BPMN 2.0 XML parser (tasks, gateways, events, sequence flows)
- `platform/src/xes_import.rs` — XES parser + 6 quality gates (GR-1..GR-6)
- `platform/src/pm4py_sidecar.rs` — Hash-based artifact contract
- Tests: ≥4 per module

#### 2.2 Phase 7a: API Layer
- `platform/src/api/mod.rs` — axum router with REST endpoints
- `platform/src/api/docs.rs` — document endpoints
- `platform/src/api/rules.rs` — rule endpoints
- `platform/src/api/cases.rs` — case adjudication endpoints
- `platform/src/api/processes.rs` — process conformance endpoints
- OpenAPI 3.1 spec generation
- Tests: endpoint integration tests (≥6)

#### 2.3 Phase 7b: Web UI Expansion
- Extend governance_web.rs with: document management, knowledge tree browser, SOP process flow diagram (SVG), case status & event timeline, audit findings & risk dashboard, evidence source & verbatim highlight, rule editor & approval, multi-document diff, agent execution trace, uncertainty & unresolved list, report export, evaluation & error analysis

### Priority 3: High-Value, High-Effort

#### 3.1 Phase 8: 50+ Cases + CI + Ablation
- Expand seed_cases.jsonl from 9 → 50+ (15 categories per master prompt)
- Dual annotation + Cohen's κ in metrics
- CI workflow (.github/workflows/ci.yml) with build/test/clippy/security/mtrix/release
- A-G ablation formal report with failure case → regression test entries
- Risk-coverage curve computation

#### 3.2 Phase 9: Security
- `platform/src/worm.rs` — WORM evidence storage with SHA-256 hash chain
- `platform/src/tenant.rs` — Tenant isolation with row-level filtering
- `platform/src/audit.rs` — Audit event log (append-only, SHA-256 chain)
- Tests: ≥4 per module

#### 3.3 Phase 10: Deployment & Local-First
- `docs/deployment/install.md`, `config.md`, `backup.md`, `security.md`, `monitoring.md`, `troubleshooting.md`
- `scripts/check.ps1`, `scripts/check.sh` — fixed local check scripts
- `docs/code-review-checklist.md`
- `docs/remote-collaboration-criteria.md`
- `docs/success-metrics.md` — quantifiable success metrics

#### 3.4 Deliveries E-G
- Delivery E: A-G ablation formal report + failure case regression
- Delivery F: Deployment docs
- Delivery G: Research plan (next batch methods, hypotheses, success criteria, cost estimates)

#### 3.5 Cleanup
- Remove `platform/src/test_engine.rs` (leftover from debugging)

## Validation Plan

### Per-Module Validation
- Unit tests (≥4 per module, happy path + edge cases)
- Integration tests where applicable
- Clippy zero warnings
- cargo fmt clean
- Selftest assertion where applicable

### Hard Gates (all must pass)
- H-1: Forbidden transition release = 0
- H-2: Miss rate = 0
- H-3: Insufficient-evidence judged comply = 0
- H-4: Citation anchor resolvable = 100%
- H-5: Injection ignored = 100%
- H-6: Replay consistency = 100%
- H-7: Causal cannot_identify rate = 100% when data insufficient
- H-8: Agent stopping condition triggered when no new evidence
- H-9: WORM hash chain integrity = 100%
- H-10: Tenant isolation violation = 0

### Success Metrics
- Accuracy: verdict accuracy ≥ 0.95
- Processing time: P95 < 500ms per case
- Manual handoff rate: < 20%
- Error rate: < 1% wrong_allow
- Cost: < $0.01 per case
- Completion rate: 100% of cases get a verdict

## Implementation Order

1. Remove test_engine.rs (cleanup)
2. Phase 6 Integration (agents into engine)
3. Phase 3 (vector + Tantivy + reranker + unified retrieval)
4. Phase 4 (BPMN + XES/OCEL + PM4Py)
5. Phase 7a (API layer)
6. Phase 7b (web UI expansion)
7. Phase 8 (50+ cases + CI + ablation)
8. Phase 9 (WORM + tenant + audit)
9. Phase 10 (deployment docs + local-first)
10. Deliveries E-G

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Scope creep | Strict phase ordering; clear acceptance criteria |
| Encoding corruption | Use cargo fmt after every edit; never PowerShell Set-Content without -Encoding UTF8 |
| Test count inflation | Only count tests verifying real behavior |
| Performance regression | Benchmark each phase; P95 < 500ms gate |
| Security gaps | Security review before each phase completion |

## Open Questions

1. Should API layer (Phase 7a) come before retrieval improvements (Phase 3)?
2. PM4Py sidecar: hard dependency or optional (feature-gated)?
3. Target deployment environment (Windows/Linux/both)?
4. Knowledge graph: separate crate or within platform?
5. Minimum viable case count for 50+ expansion?
6. Multi-tenant support needed?
7. Offline LLM support needed?
