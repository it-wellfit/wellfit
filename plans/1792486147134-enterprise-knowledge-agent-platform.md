# Enterprise Knowledge & Action Platform — Integration & Evolution Plan

> Plan ID: `1792486147134-enterprise-knowledge-agent-platform`
> Scope: Integrate Wellfit (data pipeline) + ekpi-platform (knowledge/RAG/governance) + SyncMesh (sync) into a unified, top-tier enterprise knowledge & action platform
> Method: Master Prompt lifecycle (G0→G10), evidence-driven, local-first
> Status: Plan only — no source edits in this pass

## 1. Objective

Transform the three existing Rust projects into a unified **Open Enterprise Knowledge & Action Platform** that:
- Provides evidence-grounded knowledge search with traceable citations and permission-aware retrieval
- Supports cross-system action execution with approval gates, idempotency, and rollback
- Delivers enterprise governance (identity, roles, audit, cost, reliability metrics)
- Is self-hostable, testable, continuously evolvable, and superior to market alternatives

## 2. G0 — Environment & Project Inventory (verified)

### 2.1 Three existing projects

| Project | Path | Stack | Status | Key capabilities |
|---|---|---|---|---|
| **Wellfit** | `Dev/Wellfit/` | Rust 5-crate workspace, SQLite | Production-quality, 162 tests, v1.0.7 | MTRIX 5-file/56-col pipeline, VR-01..15, ANOM-01..12, idempotent ledger, release gate, desktop mutex, scaled decimal, CR-only line endings |
| **ekpi-platform** | `Dev/ekpi-platform/` | Rust platform+eval crates, CSS-first zero-JS | Tested local prototype, Phase A not approved, Phase B blocked | Knowledge search (fulltext, vector, reranker, retrieval), governance web, process mining (XES/OCEL), causal analysis, formal verification, WORM audit log, tenant isolation (library-only), 20-rule engine, 5-doc knowledge base |
| **SyncMesh** | `Dev/SyncMesh/` | Rust 4-crate workspace | Prototype | Device discovery, CRDT sync, conflict resolution, identity management |

### 2.2 Available tooling
- Rust stable (MSVC), Cargo, rustfmt, clippy
- Build script: `Knowledge/tools/dev-env/build.ps1` (target: `%LOCALAPPDATA%\Temp\wellfit-target`)
- OS App Control constraint (blocks test binaries in some dirs)
- Git (local, no remote), git bundle for offline backup
- PowerShell 5.1, Python (available)
- No Docker, no external services, no cloud accounts

### 2.3 Key constraints
- Local-first: no cloud, no external services, self-hostable
- Rust primary stack, CSS-first zero-JS presentation
- No unsafe code, clippy zero warnings
- Phase A not approved, Phase B blocked (ekpi-platform)
- No enterprise auth, no tenant isolation in production, no off-host backup yet

### 2.4 Integration opportunities
1. **Wellfit → ekpi-platform**: MTRIX pipeline output (validated/repaired CSV) becomes knowledge source ingested by ekpi-platform's document pipeline
2. **SyncMesh → platform**: Multi-node knowledge sync for distributed deployments
3. **Wellfit's ledger → ekpi-platform's audit**: Unified append-only audit trail
4. **Wellfit's release gate → ekpi-platform's governance**: Shared release gate state machine

## 3. G1 — Product Exploration & Problem Validation

### 3.1 Candidate problems (hypotheses, not validated)

| Candidate | Hypothesis | Validation difficulty |
|---|---|---|
| A. Enterprise knowledge search with citations | Employees spend too long finding authoritative, current documents | Data quality, permissions, source correctness |
| B. AI cross-system work assistant | Employees repeat copy-paste across systems | Permissions, failure recovery, action safety |
| C. Enterprise process automation | Manual handoffs cause delays and rework | Process variation, exceptions, maintenance cost |
| D. Unified AI & integration governance | Admins struggle with connector maintenance, monitoring, permissions | Adoption threshold, integration complexity |

### 3.2 Recommended starting point
**Candidate A (knowledge search with citations)** as Phase 1, with B as Phase 2 direction. Rationale: knowledge search has clear quality benchmarks and controllable MVP scope; if research shows a high-frequency cross-system workflow is the bigger pain, pivot to that workflow.

### 3.3 What already exists (ekpi-platform)
- Document ingestion (`docs.rs`, `xes_import.rs`)
- Full-text search (`fulltext.rs`)
- Vector search (`vector_search.rs`, `retrieval.rs`, `retrieval_v2.rs`, `reranker.rs`)
- Knowledge graph (`knowledge_graph.rs`)
- Process mining (`process.rs`, `causal.rs`, `bpmn.rs`)
- Governance web (`governance_web.rs`)
- Evidence/audit (`evidence.rs`, `worm.rs`, `audit.rs`)
- Tenant isolation (library-only, `tenant.rs`)
- Formal verification (`formal_verification.rs`, `runtime_verification.rs`)

### 3.4 Key gaps to close
- No authenticated identity (local_auth is library-only)
- No real document ingestion pipeline from external sources
- No LLM integration (no model calls, no RAG orchestration)
- No permission-aware retrieval (retrieval doesn't check user permissions)
- No citation verification (answers not grounded in retrieved chunks)
- No evaluation harness (metrics exist but no fixed test set with ground truth)
- No deployment/monitoring/backup for production

## 4. G2 — Requirements Baseline (first vertical slice)

### 4.1 First vertical slice: "Knowledge Search with Citations"
A single end-to-end flow: user asks a question → system retrieves relevant chunks from authorized documents → presents answer with clickable citations → user can view source → system logs the interaction.

**Acceptance criteria (measurable):**
- Task completion rate ≥ 85% on predefined knowledge-lookup tasks
- Citation support rate ≥ 95% (human-judged sample)
- Permission isolation: zero unauthorized data leakage (negative tests)
- p95 latency < 2s for search (local, 100 docs)
- All search results include source document + version + chunk location

### 4.2 Requirements for the slice
| ID | Requirement | Priority | Source |
|---|---|---|---|
| REQ-KS-001 | User can submit a natural-language question | P0 | Master prompt §3 |
| REQ-KS-002 | System retrieves top-K relevant chunks from authorized docs | P0 | ekpi-platform retrieval |
| REQ-KS-003 | Each answer includes citations with doc ID, version, chunk location | P0 | Master prompt §3 |
| REQ-KS-004 | No answer without supporting evidence (refuse if no evidence) | P0 | Master prompt §3 |
| REQ-KS-005 | Permission check before retrieval (user only sees authorized docs) | P0 | Master prompt §9 |
| REQ-KS-006 | Empty results / no evidence handled gracefully | P1 | Master prompt §3 |
| REQ-KS-007 | Source conflict / outdated data flagged | P1 | Master prompt §3 |
| REQ-KS-008 | Interaction logged (privacy-minimized, auditable, deletable) | P1 | Master prompt §3 |
| REQ-KS-009 | Fixed evaluation set with ground truth | P1 | Master prompt §4 |

### 4.3 Non-functional requirements
| ID | Requirement | Target |
|---|---|---|
| NFR-001 | Search latency p95 < 2s (local, 100 docs) | Measured via benchmark |
| NFR-002 | Zero unsafe code, clippy zero warnings | CI gate |
| NFR-003 | All public APIs documented | Code review |
| NFR-004 | Permission isolation verified by negative tests | Test suite |
| NFR-005 | Audit log append-only, hash-chained | Existing WORM module |

## 5. G3 — Prototype & Technical Validation

### 5.1 High-risk assumptions to validate
1. **Retrieval quality**: Does the existing retrieval pipeline (fulltext + vector + reranker) produce relevant results for real queries?
2. **Permission-aware retrieval**: Can we filter results by user permissions without breaking retrieval quality?
3. **Citation accuracy**: Can we map retrieved chunks back to exact source locations?
4. **Latency**: Can we achieve <2s p95 with the existing stack on 100 docs?
5. **Evaluation**: Can we build a fixed test set with ground-truth answers?

### 5.2 Validation approach
- Use ekpi-platform's existing `docs.json` (5 documents, 8 versions) as initial knowledge base
- Build a 20-question evaluation set with ground-truth answers and expected citations
- Run retrieval + reranking, measure precision@K, recall@K, citation support rate
- Add permission filtering layer, verify no leakage via negative tests
- Benchmark latency on local hardware

### 5.3 Success criteria for G3
- Retrieval precision@5 ≥ 0.7 on evaluation set
- Citation support rate ≥ 0.9 (human-judged 20 samples)
- Permission isolation: 0 leakage in 50 negative test cases
- p95 latency < 2s

## 6. G4 — Implementation Plan (first vertical slice)

### 6.1 Work breakdown (ordered by dependency)

**Slice 1: Permission-aware retrieval (foundation)**
- Add `PermissionContext` to retrieval pipeline
- Filter documents/chunks by user permissions before scoring
- Add negative tests for cross-user/cross-tenant leakage
- Files: `platform/src/retrieval.rs`, `platform/src/tenant.rs`, `platform/src/local_auth.rs`

**Slice 2: Citation generation**
- Map each retrieved chunk to (doc_id, version, chunk_offset, char_range)
- Include citation metadata in search response
- Add citation verification (chunk text matches source)
- Files: `platform/src/retrieval.rs`, `platform/src/evidence.rs`, `platform/src/docs.rs`

**Slice 3: Search API + CLI**
- Wire retrieval into a `search` command (CLI first, web later)
- Output: question, answer (if LLM available), citations, confidence, latency
- If no LLM: return ranked chunks with citations (retrieval-only mode)
- Files: `platform/src/main.rs`, `platform/src/cli.rs` (if exists)

**Slice 4: Evaluation harness**
- Fixed test set (20 questions + ground truth)
- Metrics: precision@K, recall@K, citation support, latency, permission isolation
- CLI command: `ekp eval --test-set <file>`
- Files: `eval/src/` (extend existing)

**Slice 5: Governance & audit**
- Log every search interaction (query, user, results, citations, latency)
- Append to existing WORM audit log
- Add privacy controls (query deletion, retention policy)
- Files: `platform/src/audit.rs`, `platform/src/worm.rs`

### 6.2 Integration with existing projects
- **Wellfit data pipeline**: MTRIX output (validated CSV) → document ingestion → knowledge base
- **SyncMesh**: Multi-node knowledge sync (deferred to Phase 2)
- **Wellfit ledger**: Unified audit trail (deferred to Phase 2)

### 6.3 Testing strategy
- Unit tests: retrieval filtering, citation mapping, permission checks
- Integration tests: end-to-end search flow with test documents
- Negative tests: permission isolation (50 cases)
- Evaluation: fixed test set with ground truth
- Performance: latency benchmark on 100 docs

## 7. G5-G10 — Roadmap (not in this plan's scope)

| Phase | Focus | Trigger |
|---|---|---|
| G5 | Testing, security, quality review | After Slice 1-5 complete |
| G6 | User validation with real tasks | After G5 |
| G7 | Deployment, backup, monitoring, rollback | After G6 validation |
| G8 | Observability, continuous evolution | After production deployment |
| G9 | User acceptance & value measurement | After G7 |
| G10 | Documentation, traceability, retrospective | Continuous |

## 8. Risks & Unknowns

| Risk | Impact | Mitigation |
|---|---|---|
| ekpi-platform Phase A not approved | Cannot claim production readiness | Treat as prototype; validate before claiming |
| No LLM integration exists | Cannot generate answers, only retrieve chunks | Retrieval-only mode first; add LLM later |
| No authenticated identity | Cannot enforce real permissions | Use local_auth (library-only) for prototype; mark as unvalidated |
| Retrieval quality unknown | May not meet 0.7 precision target | Build eval set first; measure before claiming |
| Three projects not integrated | May have incompatible data models | Define integration contract first; defer actual integration |

## 9. Open Questions

1. Should the first vertical slice focus on retrieval-only (no LLM) or include a local LLM (e.g., Ollama)?
2. Should we integrate Wellfit's MTRIX output as a knowledge source in the first slice, or defer to Phase 2?
3. What is the target document count for the initial knowledge base (10? 100? 1000?)?

## 10. Recommended Next Action

Switch to an implementation-capable agent and execute **Slice 1 (Permission-aware retrieval)** as the first vertical slice, starting with a thorough read of `ekpi-platform/platform/src/retrieval.rs`, `tenant.rs`, and `local_auth.rs` to understand the existing retrieval and permission model before making changes.
