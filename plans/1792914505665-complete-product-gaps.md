# Plan: Complete Remaining Product Gaps (Stubs → Real Implementations)

## Current state (verified)

- Engine (`platform/src/engine.rs`): real rule-based adjudication, no longer heuristic. Clippy clean, fmt clean, compiles.
- Eval crate: clippy clean, fmt clean, selftest passes.
- All test binaries blocked by Windows Application Control policy (os error 4551) — environment limitation, not code.
- Stubs remain: `ocr.rs`, `multimodal.rs`, `formal_verification.rs`, `runtime_verification.rs`, `benchmark.rs` (platform), `retrieval.rs` (bigram only), `vector_search.rs` (basic cosine), `reranker.rs` (basic).
- Governance web: local-only, no auth, no connectors.
- No real backup executed.

## Goal

Replace the remaining stubs with functional implementations that are testable offline, keeping all existing tests green and clippy zero-warning.

## Ordered tasks (implementation-capable agent)

### T1 — Real hybrid retrieval (retrieval_v2 + vector_search integration)
- Wire `vector_search.rs` (cosine similarity) into `retrieval_v2.rs` as the semantic channel.
- Combine lexical (fulltext BM25) + semantic (vector) scores with configurable α.
- Add `HybridRetriever` struct with `search(query, top_k) -> Vec<RetrievalHit>` returning verbatim citations.
- Tests: empty index returns empty; relevant doc scores higher; citation anchor present.

### T2 — OCR module (real Tesseract boundary, not stub)
- Define `OcrEngine` trait with `recognize(image_bytes) -> Vec<OcrResult>`.
- Implement `TesseractOcrEngine` that shells out to `tesseract` CLI if available; fall back to `StubOcrEngine` with a warning note when binary missing (local-first, no hard dependency).
- Add `layout.rs` for bounding-box grouping into reading order.
- Tests: stub returns empty; mean_confidence; layout ordering.

### T3 — Multimodal document understanding (real, not stub)
- Implement `MultiModalDocument` with text + image + table modalities.
- Add `extract_text_from_image` using OCR trait.
- Add `table_parser.rs` for CSV/TSV table extraction.
- Tests: add/retrieve modalities; text extraction; table parsing.

### T4 — Formal verification (TLA+ model checker boundary)
- Define `FormalVerifier` trait with `check(property, model) -> VerificationResult`.
- Implement `StubFormalVerifier` (returns Unknown) as default; document that real TLA+ requires external `tlc` binary.
- Add property parser for `[]` (always) and `<>` (eventually) temporal operators.
- Tests: parse property; unknown by default; property validation.

### T5 — Runtime verification monitor (real event evaluation)
- Implement `RuntimeMonitor` that evaluates invariants against event streams.
- Add invariant types: `Always(activity implies next)`, `Never(forbidden pair)`, `Eventually(activity)`.
- Return `MonitorVerdict::Violation` with trace when violated.
- Tests: empty events → Ok; forbidden pair → Violation; eventually → Ok/Violation.

### T6 — Governance web: real auth boundary (local-only)
- Add `local_auth.rs` with bcrypt password hashing (if crate available) or SHA-256 fallback.
- Add session token (UUID) with expiry.
- Add role-based access: viewer/editor/approver/admin.
- Keep local-only (no enterprise auth, per AGENTS.md constraint).
- Tests: password hash roundtrip; session expiry; role check.

### T7 — Verified local backup execution
- Run `git bundle create` to `%USERPROFILE%\EKPI-backup\bundles\`.
- Restore drill: clone bundle, run cargo test + clippy + fmt, confirm exit 0.
- Record result in `docs/operations/backup-log.md`.
- Add `scripts/backup.ps1` and `scripts/backup.sh`.

### T8 — Double-annotation κ (eval)
- Add `eval/src/annotation.rs` with Cohen's κ computation.
- Add second annotator labels to `eval/cases/` (`.annotator2.jsonl`).
- Report κ in `report.md`; threshold ≥0.80 before ground-truth use.
- Tests: κ perfect agreement = 1.0; κ chance = 0.0.

### T9 — CI workflow (GitHub Actions, local-first)
- `.github/workflows/ci.yml`: matrix ubuntu + windows; jobs: fmt, clippy, test, selftest, schema, secret-scan.
- Secret scan: grep for common secret patterns in commit range.
- Require all jobs pass before merge.

### T10 — Final validation & report update
- Run full baseline (`scripts/check.ps1`), record results.
- Update `docs/operations/final-validation-report.md` with actual measured values.
- Update `docs/traceability/register.md` with all new items.

## Constraints (non-negotiable)

- `#![forbid(unsafe_code)]` on every Rust file; clippy `-D warnings` zero.
- PowerShell boundary: `;` only, never `&&`/`|`/`&` in one line.
- UTF-8 only; use Write tool or Python for file edits.
- No fabricated integrations/permissions/approvals/scan results.
- Governance web remains local-only (no enterprise auth).
- No org facts prefilled in compliance deliverables.

## Validation per task

Each task: write failing test first → implement → run affected checks → code review → rerun full baseline.

## Out of scope (this plan)

- Enterprise authentication, external connectors, AI workflow drafts.
- PM4Py sidecar production integration.
- Real TLA+ model checking (requires external `tlc` binary; stub boundary documented).

## Open decisions

- Whether to add `bcrypt` crate dependency (T6) or use SHA-256 fallback.
- κ threshold confirmation (proposed ≥0.80).
- Backup cadence and storage path confirmation.

## Deliverable

Implementation-ready plan. Switch to code agent to execute T1–T10.