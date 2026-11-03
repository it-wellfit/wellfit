# EKPI Platform — 0→1 MVP Completion + MVP→Production-Grade Path

> **Disposition (2026-11-02): Historical proposal, not an approved current roadmap.** This EKPI-scoped plan is stored under `Wellfit/plans/`; its gate and task status is an old snapshot. Reconcile only in `ekpi-platform/` against [its charter](../../ekpi-platform/PROJECT_CHARTER.md) and current test evidence.

## Framework

Matt Pocock skills-based product engineering framework, two phases:
- **Phase A (0 → 1 MVP):** Complete the prototype into a traceable, verifiable MVP that can execute one real authorized workflow end-to-end on non-sensitive data.
- **Phase B (MVP → Production-grade):** Production hardening — security, reliability, observability, delivery governance, AI-quality, operations, backup/DR, data governance.

---

## 1. Current State Summary (verified 2026-10-26)

- Platform crate: `fmt` clean, `clippy` clean (`-D warnings`), build clean, tests clean.
- Eval crate: `fmt`/`clippy`/build/tests clean.
- Phase gates: **A4 Met**, **A8 Met**; **A1 Not met**.
- Phase B: **Blocked** (depends on A1 closure).
- Engine: real rule-based adjudication (no longer uses `expected_verdict` heuristic).
- T1–T10 complete: hybrid retrieval, OCR, multimodal, formal/runtime verification, local auth/RBAC, backup scripts, annotation κ, CI workflow.

---

## 2. Phase A Tasks (A-01 … A-07)

### A-01 — A1 Gap Closure (traceability for one real authorized workflow)
- **Skills:** `grilling`, `senior-data-engineer`
- **Artifacts:** Traceability register entries (requirement ID → PRD → ADR → contract → ticket → implementation → automated test → release evidence → retro), one end-to-end workflow run on non-sensitive data.
- **Definition of done:** A1 gate marked Met; single authorized workflow executes end-to-end and is evidenced.
- **Verification:** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, traceability register audit.

### A-02 — Requirement-to-evidence chain wiring
- **Skills:** `grilling`
- **Artifacts:** Updated traceability register, linked ADRs, contracts.
- **Definition of done:** Every requirement ID in scope traces through the full chain.
- **Verification:** Register review + automated link check.

### A-03 — Non-sensitive data seed set & workflow fixture
- **Skills:** `senior-data-engineer`, `dataset-transformation`
- **Artifacts:** Seed dataset (non-sensitive), workflow fixture, expected outputs.
- **Definition of done:** Fixture loads and runs through the workflow without PII.
- **Verification:** Integration test green.

### A-04 — End-to-end authorized workflow execution (A4 gate)
- **Skills:** `webapp-testing`
- **Artifacts:** E2E test, run evidence, screenshots/logs.
- **Definition of done:** A4 Met (already) re-verified after A-01.
- **Verification:** E2E test pass + evidence bundle.

### A-05 — Annotation κ validation
- **Skills:** `senior-data-engineer`
- **Artifacts:** κ report, inter-annotator agreement evidence.
- **Definition of done:** κ ≥ threshold documented; evidence stored.
- **Verification:** Report review.

### A-06 — Local auth/RBAC verification
- **Skills:** `microsoft-code-reference`
- **Artifacts:** RBAC policy doc, permission matrix test.
- **Definition of done:** Roles enforced on the one real workflow path.
- **Verification:** Auth tests green.

### A-07 — MVP release evidence bundle
- **Skills:** `content-research-writer`
- **Artifacts:** Release evidence pack (tests, clippy, fmt, build, traceability, retro).
- **Definition of done:** Bundle complete and reviewable.
- **Verification:** Checklist sign-off.

---

## 3. Phase B Tasks (B-01 … B-08)

### B-01 — Security hardening (local-only governance web)
- **Skills:** `microsoft-code-reference`
- **Artifacts:** Security review doc, threat model, hardening checklist.
- **Definition of done:** No secrets in repo; `#![forbid(unsafe_code)]` enforced; clippy `-D warnings` zero.

### B-02 — Reliability & error handling
- **Skills:** `senior-data-engineer`
- **Artifacts:** Retry/circuit-breaker config, error taxonomy.
- **Definition of done:** Failure modes documented and tested.

### B-03 — Observability (metrics, logs, traces)
- **Skills:** `senior-data-engineer`
- **Artifacts:** Prometheus/Grafana dashboards, tracing spans.
- **Definition of done:** Key SLOs instrumented.

### B-04 — Delivery governance (CI/CD)
- **Skills:** `senior-data-engineer`
- **Artifacts:** CI workflow, release pipeline, gate definitions.
- **Definition of done:** Automated gates block bad merges.

### B-05 — AI-quality evaluation
- **Skills:** `dataset-transformation`, `senior-data-engineer`
- **Artifacts:** Eval harness results, quality report.
- **Definition of done:** Quality thresholds met and tracked.

### B-06 — Operations runbook
- **Skills:** `content-research-writer`
- **Artifacts:** Ops runbook, incident playbook.
- **Definition of done:** On-call procedures documented.

### B-07 — Backup/DR
- **Skills:** `senior-data-engineer`
- **Artifacts:** Backup scripts (existing), DR test evidence.
- **Definition of done:** Restore tested and timed.

### B-08 — Data governance
- **Skills:** `senior-data-engineer`, `purview-data-catalog`
- **Artifacts:** Data classification, retention policy, lineage.
- **Definition of done:** Governance controls documented and enforced.

---

## 4. Constraints (non-negotiable)

1. `#![forbid(unsafe_code)]` in all crates.
2. `clippy -D warnings` zero warnings.
3. `cargo fmt` clean.
4. No secrets committed to repo.
5. No fabricated claims (assumptions marked as hypotheses).
6. Governance web is local-only.
7. Evidence over assumptions; architecture simplicity; local-first git as authoritative history.

---

## 5. Validation per Work Item

TDD → implement → checks (`fmt`, `clippy -D warnings`, build, tests) → code review → baseline commit → traceability register update.

---

## 6. Open Decisions Requiring External Input

1. Threshold values for κ and quality gates (stakeholder sign-off).
2. SLO targets for production (ops + product).
3. Data classification tiers and retention periods (legal/compliance).
4. Release cadence and change-approval authority.
5. Scope of first production tenant / non-sensitive data source.

---

## 7. Out of Scope

- axum/Maud web UI (deferred, Round 3).
- PM4Py sidecar oracle (deferred).
- Expanded 50+ seed cases (deferred).
- Causal module (deferred).
- Agent roles (deferred).
- Microservices / Kubernetes / Kafka / vector DB / complex event architecture (forbidden by simplicity constraint).
- Docker-based Windmill deployment (use native binaries only).
