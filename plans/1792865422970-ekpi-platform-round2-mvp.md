# EKPI Platform Round 2 — MVP Implementation Plan

> **Disposition (2026-11-02): Historical proposal, not an approved current roadmap.** This EKPI-scoped plan is stored under `Wellfit/plans/`; its baseline and tasks must be reconciled in `ekpi-platform/` against [its charter](../../ekpi-platform/PROJECT_CHARTER.md) and current test evidence.

**Goal**: First executable, testable, demonstrable version: document import + version tree + clause anchors, hybrid retrieval with verbatim citations, rule library (JSON Schema + LegalRuleML export), process verification prototype (Petri net token replay / PM4Py sidecar), decision snapshot + replay, and real eval adapters.

**Scope**: Round 2 bricks R2-1..R2-7 from `docs/tech-selection-round1.md §6`. Web UI (axum + Maud) deferred to Round 3; this round delivers Rust crates + CLI + eval harness integration.

**Stack (decided, Round 1)**: Rust 全端 (`#![forbid(unsafe_code)]`, clippy `-D warnings` zero), SQLx + PostgreSQL + pgvector, Tantivy full-text, Maud CSS-first UI (Round 3), PM4Py sidecar for conformance (Round 3 oracle), Python = reference only (`eval/reference-python/`).

**Location**: `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\ekpi-platform/`

---

## 1. Current state (Round 1 complete)

- `research/papers.csv`: 18 papers, all `verified=yes` (2026-10-25)
- `research/standards.csv`: 10 standards, all verified (2026-10-25)
- `design/rule-schema-v0.1.md` + `rule-schema-v0.1.schema.json`: normative rule model, LegalRuleML v1.0 export target
- `design/process-verification-selection.md`: BPMN/DMN/XES/OCEL + PM4Py, GR-1..GR-6 quality gates, four-class conformance output
- `eval/`: Rust crate `ekp_eval`, 9 seed cases (`cases/seed_cases.jsonl`), 3 stub adapters (oracle/allow_all/refuse_all), selftest PASS 10/10, clippy zero warnings, `report_out/` generated
- `docs/tech-selection-round1.md`: stack, integration matrix, MVP scope, acceptance gates H-1..H-6, phased plan
- `docs/prompt/master-prompt-addendum-v2.md`: 12 addendum clauses (A1..A12), overrides master prompt v1 on conflict

---

## 2. Acceptance gates (hard, non-negotiable)

| ID | Gate | Measurement |
|----|------|-------------|
| H-1 | Prohibited state-transition pass rate | = 0 on ORD-001-class cases |
| H-2 | Miss rate (actual non-compliance judged comply) | = 0 |
| H-3 | Insufficient-evidence judged comply | = 0 on ORD-002 / EVD-001 |
| H-4 | Citation integrity | Every formal conclusion ≥ 1 locatable anchor; parse success = 100% |
| H-5 | Injection does not change verdict | SEC-001 class: document-internal instruction ignored |
| H-6 | Replay consistency | Same snapshot replays byte-identical (excluding timestamp fields) |

Soft metrics (Round 3 report): Recall@k, MRR, citation precision/recall, rule verdict accuracy, correct refusal rate, refusal on definitive, risk–coverage curve, latency, cost.

---

## 3. Tasks (actionable, ordered)

### R2-1 — Document import + version tree + clause anchors + hash
- Crate: `ekp_import` (or add to workspace)
- Ingest versioned SOP seed documents (3–5 SOPs, 2 versions each) as structured input (no layout parser in MVP; human-structured seed files)
- Store: file bytes + SHA-256 + version + effective/retention dates + access level (object store + Postgres metadata)
- Build document tree: `document_version`, `clause_node(parent, order, section_path, page, anchor)` in Postgres
- Output: clickable clause anchors (section/page/paragraph) + verbatim text
- Test: import idempotent; hash chain stable; clause anchor resolution 100%

### R2-2 — Rule library + JSON Schema validation + review workflow + LegalRuleML export (v0)
- Crate: `ekp_rules`
- Store rules as JSONB validated against `design/rule-schema-v0.1.schema.json`
- Lifecycle: `draft_ai → expert_validated → active → retired`; engine reads only `active` (A1, A8 rule-poisoning firewall)
- Fields: deontic modality/strength, condition tree, sequencing (`required_before`), exceptions (`overrides/overridden_by/conflicts_with`), temporal (`effective_from/to`, `supersedes`), `evidence_requirements[]` with `missing_evidence_result`, `decision.method`, `source` (document_id/version/clause_refs/verbatim_text), `provenance.origin`
- LegalRuleML v1.0 export (document-level cross-check against OASIS spec part names before first export)
- Rule↔document version binding + coverage ledger (clause → rule mapping); version bump emits unmapped/expired rule gap report (A2)
- Test: schema rejects invalid rule; lifecycle transition recorded in `rule_review_log`; draft rules invisible to engine

### R2-3 — Process verification prototype (Petri net token replay / PM4Py sidecar)
- Crate: `ekp_process` (Rust native token-based replay + reachability for MVP conformance v0)
- BPMN authoring layer → Petri net derivation (PM4Py sidecar optional in MVP)
- XES (IEEE 1849-2023) import; OCEL 2.0 for multi-object cases
- GR-1..GR-6 quality gates before conformance judgment (unconfirmed events excluded from verdict)
- Conformance output: four classes — comply / deviate / missing_event / unconfirmed, each with evidence citation
- Loop control: enter/exit predicates, retry cap (parameter, not hardcoded), escalate on cap breach, idempotent retry token (no duplicate side effects)
- Hard gates H-1/H-2: prohibited transitions never pass; token replay deterministic
- PM4Py sidecar (Round 3 oracle) exchanges hashed XES/OCEL artifacts only

### R2-4 — Hybrid retrieval + verbatim citation engine
- Crate: `ekp_retrieve`
- pgvector (dense) + Postgres FTS (BM25-approximation) + document-tree navigation
- Filters: time, version, permission, applicability scope
- Reranker: deferred (watch), recall-first
- Output: every chunk carries source location (document/version/section/page/anchor) + verbatim text
- Parent-child retrieval; answer must include clickable verbatim anchors (H-4)
- Test: single-clause query → exact clause anchor; cross-section → tree + multi-hop; injection document does not override verdict (H-5)

### R2-5 — Abstention + risk–coverage calibration + threshold parameter file
- Crate: `ekp_abstain` (or in `ekp_rules` decision layer)
- Verdict types: `comply / violate / insufficient_evidence / escalate`
- `insufficient_evidence` / `escalate` are formal stop outputs, not failures (A5)
- `abstain_threshold` = configurable parameter, versioned + approved (no hardcoding)
- Cost matrix: W_FN > W_FP (business-stated)
- Metrics: refusal_on_definitive, correct_refusal, risk–coverage curve
- Test: definitive cases not refused; evidence-missing cases correctly abstain

### R2-6 — Decision snapshot + replay
- Crate: `ekp_snapshot` (or in `ekp_rules`/`ekp_process`)
- Manifest: `decision_id`, inputs (`case_id`, `doc_versions`, `rules`, `retrieved_chunks_sha256`, `events_sha256`, `model_id`, `prompt_template_sha256`, `config_sha256`), `llm_artifacts` (raw answer hashes), `verdict`, `computed_by` (A6)
- Replay: deterministic path only (rules + conformance + evidence comparison); LLM output compared by hash, model not re-called
- Byte-identical replay across runs (excluding timestamp fields) (H-6)
- WORM storage + content hash chain; write-after immutable (A8)

### R2-7 — Eval crate expansion: real adapters + 50+ cases + CI
- Crate: `ekp_eval` (existing; extend)
- Register real system adapter(s) in `src/systems.rs` `registry()` wrapping R2-1..R2-6 components
- Expand `cases/seed_cases.jsonl` from 9 → 50+ cases (double annotation, Cohen's κ, A7)
- Bind test-set version to document versions (`doc_versions` field already in schema)
- Ablation configs in `configs/ablations.json` (A–G flags): A naive RAG, B hybrid+rerank, C tree index, D GraphRAG, E agentic, F process+rules, G full
- CI integration: `cargo test --workspace --no-fail-fast`; clippy zero warnings; hard gates fail CI
- Output: `report_out/report.json` + `report.md` with per-case failure analysis

---

## 4. Module interfaces (contracts)

- `ekp_import::import_document(path, version) -> DocumentVersionId`
- `ekp_rules::evaluate(case_id, rule_ids, evidence) -> Verdict + evidence_links`
- `ekp_process::conformance(bpmn_xml, xes_log) -> FourClassReport`
- `ekp_retrieve::retrieve(query, filters) -> Chunk[] with anchors`
- `ekp_snapshot::capture(decision) -> SnapshotManifest`; `ekp_snapshot::replay(manifest) -> bool`
- `ekp_eval::run(adapter, cases) -> Summary`

All crates: `#![forbid(unsafe_code)]`, clippy `-D warnings`, no floating-point for money/quantity (use integer cents), no silent data mutation.

---

## 5. Validation

1. `cargo clippy --workspace --all-targets -- -D warnings` → zero warnings
2. `cargo test --workspace --no-fail-fast` → all pass
3. `cargo run -- --selftest` (in `eval/`) → 10/10 pass
4. `cargo run -- --out report_out` → report.json/report.md generated
5. Hard gates H-1..H-6 verified against seed cases ORD-001..PAR-001
6. Replay byte-identical check on captured snapshots

---

## 6. Open questions / assumptions

- **PM4Py sidecar**: not installed in MVP; conformance v0 uses Rust native token replay. PM4Py integration deferred to Round 3 as alignments oracle. (Assumption: native replay sufficient for H-1/H-2 hard gates.)
- **axum web backend + Maud UI**: deferred to Round 3. Round 2 = CLI crates + eval harness only.
- **Vector model/embedding provider**: not selected in Round 1 (`待鎖定` in standards.csv). To be locked before R2-4 implementation.
- **Object storage backend**: local filesystem for MVP; S3-compatible abstraction optional later.
- **Plan file location**: written alongside project at `ekpi-platform/plans/` (repo-root plans dir) for proximity to source.

---

## 7. Definition of done (Round 2)

- All 7 bricks implemented, tested, CI-integrated
- Hard gates H-1..H-6 pass on seed cases
- Real eval adapter produces non-stub report with failure analysis
- Replay byte-identical; snapshot manifest captured per decision
- `eval/` selftest still 10/10 with real adapters registered
