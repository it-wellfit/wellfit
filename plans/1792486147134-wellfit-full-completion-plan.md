# Wellfit MTRIX — Full Completion Phased Plan

> Plan ID: `1792486147134-wellfit-full-completion-plan`
> Scope: `Dev/Wellfit` (5-crate Rust workspace) + `哈瓦士串接專案` (spec/samples) + `Knowledge` (knowledge base)
> Method: local-first workflow, TDD per slice, evidence-based, no counterparty assumptions without registration
> Status: Plan only — no source edits in this pass

## 1. Objective

Complete every remaining project item to full completion, organized as a phased roadmap that an implementation-capable agent can execute slice by slice. Counterparty-blocked items are explicitly gated and never silently assumed. The plan covers local-first infrastructure (git, check entry point, backup), knowledge-base synchronization, pipeline wiring, rule/field externalization, test generation, quality gates, and gated counterparty items.

## 2. Current State (verified)

- Workspace: `wellfit_core`, `wellfit_mtrix`, `wellfit_cli`, `wellfit_reports`, `wellfit_shopee` (5 crates), Rust 2021, MSVC toolchain.
- Build/test/clippy via `Knowledge/tools/dev-env/build.ps1` with target dir `%LOCALAPPDATA%\Temp\wellfit-target` (OS App Control constraint).
- Last verified: build 0 errors, clippy 0 warnings, 143 tests pass (per CHANGELOG/audit; re-verify in Phase 0).
- No git repository exists in `Dev/Wellfit` or anywhere under `Projects/2026`. This is the single largest infrastructure gap.
- `pipeline.rs` exists and is declared in `lib.rs`, but all 7 stages are no-ops; `process_drop` does not use it.
- `ValidationConfig` exists (config.rs) — foundation for externalization.
- Counterparty-blocked: C-01 (CP format), C-02 (VR-12 mgmt report), C-03 (mandatory list), C-04 (decimals), C-05 (vendor prefix), plus G-01..G-12, P-01..P-10.
- Knowledge-base sync gaps: D-01 (mtx-issue-register), D-02 (claims-vs-evidence).
- Documentation gaps: AGENTS.md (T13), docs/ARCHITECTURE.md (T14) — partially current; CLI.md already has LOCAL-053.

## 3. Success Metrics (per MVP)

| Metric | Target | Measurement |
|---|---|---|
| Build | 0 errors | `build --workspace` |
| Clippy | 0 warnings (`-D warnings`) | `clippy --workspace --all-targets` |
| Tests | 100% pass, no regression | `test --workspace --no-fail-fast` |
| Processing time | < 5s/file | `scripts/benchmark_million.ps1` / new bench |
| Error rate (quarantined) | < 1% on delivered samples | report `rows_quarantined / rows_total` |
| Spec fidelity | 56 fields, 14 rules, 12 repairs | `wellfit spec` + tests |
| Traceability | every number → file:line or issue ID | claims-vs-evidence.md |
| Backup restore | verified | `git bundle verify` + restore drill |

## 4. Phase 0 — Local-First Foundation (week 1)

**Objective:** Establish git, a fixed local check entry point, and offsite backup from the first commit.

### 4.1 Tasks
1. Initialize git repo in `Dev/Wellfit` (standard layout; no remote yet).
2. Create `.gitattributes` with CR/LF handling for CSV fixtures (preserve CR-only bytes; do not normalize).
3. Create `scripts/check` (PowerShell, equivalent to `scripts/check.ps1`) running in order: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --no-fail-fast`, `cargo test -p wellfit_cli --test integration_tests`, plus SHA-256 fixture verification against `PROVENANCE.md`.
4. Add `.env.example` (no secrets; document env vars: target dir, db path, batch IDs).
5. Ensure `.gitignore` already excludes `.env`, `*.sqlite`, `target/`, `*.exe` — verify and extend for `.env.*` except `.env.example`.
6. Baseline verification: run full check; record results in `CHANGELOG.md` [Unreleased] or a new `VERIFICATION.md`.
7. First commit: `chore: initial commit of wellfit workspace (v1.0.2 state)`.
8. Tag `v1.0.2`.
9. Backup: `git bundle create backup-YYYYMMDD.bundle --branches --tags` to a second physical location (encrypted external drive/NAS); run `git bundle verify`; perform a restore drill into a temp dir and confirm `git log` + `cargo build` succeed.
10. Document backup/restore in `docs/OPS-BACKUP.md` (or extend RUNBOOK).

### 4.2 Validation
- `git status` clean after commit; `git bundle verify` passes; restore drill succeeds; `scripts/check` exits 0.

### 4.3 Risks
- App Control blocks test binaries only in certain target dirs — keep target at `%LOCALAPPDATA%\Temp\wellfit-target`.
- Chinese paths — use `-LiteralPath` in all scripts.

### 4.4 Exit criteria
- Git repo exists with ≥1 commit + tag; backup bundle verified and restore-drilled; `scripts/check` green.

## 5. Phase 1 — Knowledge Base Sync & Documentation (week 1–2)

**Objective:** Close D-01, D-02, T13, T14; reach 100% doc sync.

### 5.1 Tasks
1. Update `Knowledge/issues/mtx-issue-register.md`: register this round's corrections (LOCAL-048..053, C-01..C-08 statuses) with file:line evidence.
2. Update `Knowledge/traceability/claims-vs-evidence.md`: add C-23..C-27 (or current) claims with evidence; mark withdrawn claims.
3. Update `AGENTS.md`: document all six CLI subcommands with flags/exit codes, env vars, release-gate state machine, local check entry point, backup commands, target-dir constraint.
4. Update `docs/ARCHITECTURE.md`: add three-stage pipeline diagram (Parse→Repair→Validate→Aggregate→CrossFile→Report→Output), release-gate state machine (already partial — verify), data flow (already present — verify currency_groups and row_lines sync).
5. Verify `docs/CLI.md` LOCAL-053 note matches `AGENTS.md`.
6. Add `docs/knowledge-index.md` cross-links to new/updated docs.
7. Run `scripts/check`; commit `docs: sync knowledge base and architecture docs`; tag `v1.0.3-docs`.

### 5.2 Validation
- `scripts/check` green; manual diff review of doc accuracy against code (every CLI flag verified via `--help`).

### 5.3 Exit criteria
- D-01, D-02, T13, T14 marked complete with evidence; doc sync rate 100%.

## 6. Phase 2 — Pipeline Wiring (week 2–4)

**Objective:** Turn `pipeline.rs` from a no-op foundation into the actual orchestration, preserving behavior (143+ tests).

### 6.1 Tasks
1. Define a `PipelineContext`-driven refactor plan: each stage delegates to existing `MtrixEngine` methods (discover, parse, repair_header, validate_row, format_warnings, aggregate_duplicates, apply_cross_file_rules, currency_groups, report, output).
2. Implement `ParseStage::execute`: file discovery + parsing + row_lines tracking; populate `ctx.parsed`.
3. Implement `RepairStage`: header repair (ANOM-01/02/03/12) + row repair; record `AppliedRepair`.
4. Implement `ValidateStage`: VR-01..VR-04, VR-07, VR-15 per row; full rule evaluation (not first-violation).
5. Implement `AggregateStage`: VR-05 duplicate aggregation with row_lines sync.
6. Implement `CrossFileStage`: VR-08/09/10/11/13/14.
7. Implement `ReportStage`: assemble `ValidationReport`, tallies, deviations, currency_groups, spec_conformance.
8. Implement `OutputStage`: write repaired files + JSON report; only after audit write succeeds (no partial output on failure).
9. Refactor `process_drop` to construct `default_pipeline()` and call `run`; remove duplicated inline logic.
10. Add `--skip-validation` flag (optional stage skip) — gated behind config; default off.
11. Add stage-level unit tests (each stage independently testable with a fixture context).
12. Regression: full `scripts/check`; integration tests unchanged results (22E/24W/38R on delivered).

### 6.2 Validation
- Behavior equivalence: delivered sample yields identical report JSON (diff against golden) before/after.
- 143+ tests pass; clippy 0.

### 6.3 Risks
- Risk of subtle behavior drift during refactor — mitigate with golden-file comparison and TDD.

### 6.4 Exit criteria
- `process_drop` uses `Pipeline`; all stages implemented (no no-ops); golden diff identical; tests green.

## 7. Phase 3 — Rule & Field Externalization (week 4–8)

**Objective:** Move VR rules, field contract, and repair rules to external config/schema so new rules need no recompile.

### 7.1 Tasks
1. Design `rules/mtrix-vr.toml` (or JSON) schema per audit §3.2.A; migrate VR-01..VR-15 definitions (rule id, scope, field, check, severity, action, source, issue).
2. Implement `RuleEngine` in `wellfit_mtrix`: load config, execute generic checks (calendar_date, allowed_set, range, cross-file key existence, etc.).
3. Refactor `validators.rs` to consume `RuleEngine` output; keep `VrRule` enum as typed IDs for report compatibility.
4. Design `spec/mtrix-schema.json` per audit §3.2.B; migrate 56 `FieldSpec` entries with tier/status/page/issue.
5. Implement schema loader in `spec.rs` (compile-time `include_str!` first; runtime override optional).
6. Design `rules/mtrix-anom.toml` per audit §3.2.C; migrate ANOM-01..ANOM-12.
7. Implement `RepairEngine`: load repair config, apply transforms (split_country_prefix, normalize_barcode, etc.).
8. Add `ValidationConfig` integration: config-driven mandatory list, CP format, decimals, strict mode already present — extend to load from TOML.
9. Update `wellfit spec` to render from schema (not hardcoded).
10. Update `tools/reconciliation/build-field-diff.ps1` to read schema; fail on mismatch.
11. Tests: rule-config round-trip, schema validation, repair-config application.

### 7.2 Validation
- `wellfit spec --format markdown` identical output (golden diff).
- Adding a new rule via config (no code change) produces a finding on a crafted fixture.
- `scripts/check` green.

### 7.3 Risks
- Large refactor; keep behind feature flag or parallel module to avoid breaking 143 tests mid-flight.

### 7.4 Exit criteria
- Rules/fields/repairs defined in external files; engine loads and executes them; spec output identical.

## 8. Phase 4 — Test Generation & Quality Gates (week 8–10)

**Objective:** Auto-generate tests from rule definitions; expand check entry point.

### 8.1 Tasks
1. Implement test generator: from each VR rule definition, emit positive + negative + boundary cases (e.g., leap year, empty, overflow).
2. Generate into `crates/wellfit_mtrix/tests/generated/` (or `tests/`); wire into `cargo test`.
3. Expand `scripts/check` per feature risk: add e2e (integration_tests), security scan (secrets, dependency audit), performance (bench <5s/file), accessibility N/A (CLI).
4. Mutation harness: run against temp copy; assert byte-identity restoration of live repo; never mutate Dev/Wellfit.
5. Add `cargo bench` target for process_drop on delivered samples; assert <5s.
6. Add contract tests for API/data formats (spec.rs, validators.rs, repairs.rs).
7. Add release-gate smoke test.

### 8.2 Validation
- Generated tests pass; mutation harness restores byte-identical; bench <5s.

### 8.3 Exit criteria
- `scripts/check` includes e2e/security/perf; mutation harness verified; generated tests in CI.

## 9. Phase 5 — Counterparty-Gated Items (on receipt, no silent defaults)

**Objective:** Implement C-01..C-05 only when counterparty provides spec; register every assumption as a deviation otherwise.

### 9.1 Gate policy
- No implementation of C-01..C-05 without a registered counterparty response (issue ID + source).
- If a reasonable default is proposed for demo, it must be logged as a deviation (ADR) and marked `unverified`/`pending`, never presented as confirmed.

### 9.2 Tasks (per item, on receipt)
1. **C-01 CP format (P0, MTRIX-001):** implement `is_valid_cp` + VR-15 Warning; update config default; test with counterparty samples.
2. **C-02 VR-12 mgmt report reconciliation (P1, G-07):** implement total-amount cross-check against management report; needs report source format.
3. **C-03 Mandatory field list (P1, MTRIX-003):** replace `undefined_mandatory_columns` heuristic with authoritative list; update format_warnings.
4. **C-04 Decimal places (P2, MTRIX-024):** align scaled.rs scale (currently 4) to decided value (2 or 4); update tests and docs.
5. **C-05 Vendor prefix length (P2, G-11):** confirm 2 vs 3 letter; adjust split/join; update FieldSpec length.
6. **C-06 Output CR/LF & header policy (P3):** already decided (CR default, Canonical header) — document only; no code change.
7. **C-07 scaled.rs docs (P3):** clarify "raw text always preserved" caveat for aggregated rows.
8. **C-08 ALTHCI col 12 status (P3):** already ObservedOnly — verify and document.

### 9.3 Validation
- Each item: counterparty-provided test vectors pass; deviations registered; `scripts/check` green.

### 9.4 Exit criteria
- All counterparty-provided items implemented or explicitly deferred with ADR; no silent assumptions.

## 10. Phase 6 — Long-Term (month 3–6, gated on need)

**Objective:** Remote collaboration, multi-format, API, multi-tenant — only when needed.

### 10.1 Tasks (only when triggered)
1. **Remote onboarding:** connect existing local repo to GitHub/GitLab; add PR/MR, required CI checks, permissions, deployment workflows; use `/pr`. Keep standard Git layout (no rebuild).
2. **CI/CD:** GitHub Actions running `scripts/check` + generated tests + mutation harness.
3. **Multi-format support:** parser plugins (Excel/JSON/XML) behind trait.
4. **Real-time validation API:** REST endpoint wrapping `process_drop`.
5. **Multi-tenant:** tenant isolation + config management.
6. **Windmill integration:** native binary deployment (no Docker) per correction; four-layer dynamic optimization.

### 10.2 Trigger conditions
- Remote work, cloud backup, peer review, cross-team collaboration, or automated deployment needed.

### 10.3 Exit criteria
- Triggered items delivered with monitoring; non-triggered remain out of scope.

## 11. Phase 7 — Retro & A+ Rating

### 11.1 Tasks
1. Run `/retro`: capture lessons, update ADRs, update knowledge base.
2. Re-run full verification (build/clippy/test/integration/mtrix/audit/spec/bench).
3. Update `CHANGELOG.md` with version bump (1.1.0 or 2.0.0 per semver).
4. Update `README.md` status from Draft to Reviewed/Approved.
5. Produce final delivery report (top-tier practical product functionality).
6. Backup bundle + restore drill; tag final version.

### 11.2 Exit criteria
- All P0/P1 resolved; tests 100%; clippy 0; docs sync 100%; backup verified; quality rating A+.

## 12. Risk Register

| ID | Risk | Mitigation |
|---|---|---|
| R-ENV | App Control blocks test binaries in some target dirs | Use `%LOCALAPPDATA%\Temp\wellfit-target` exclusively |
| R-PATH | Chinese paths break naive scripts | Use `-LiteralPath`; test on actual paths |
| R-BEH | Pipeline refactor drifts behavior | Golden-file diff before/after; TDD |
| R-CFG | Externalization breaks compile-time guarantees | Keep `include_str!` + runtime override; tests |
| R-COUNTER | Counterparty delays block Phase 5 | Gate explicitly; no silent defaults |
| R-GIT | No git history; large initial commit | Single baseline commit; incremental thereafter |
| R-BACKUP | Bundle excludes uncommitted files | Commit before bundle; verify restore drill |

## 13. Local-First Workflow Mapping

| Step | Phase |
|---|---|
| Identify pain points & users | Done (README/AGENTS) |
| Define measurable success metrics | §3 |
| /grill-with-docs | Phase 1 (docs) |
| /research (technical/domain) | Phase 3, 5 |
| /prototype (high-risk/UI) | N/A (CLI only) |
| /to-spec | This plan |
| /to-tickets (multiple slices/deps) | Phase 2–6 task lists |
| /implement or /implement-spec | Implementation agent (next) |
| Local testing, eval, code review | Each phase validation |
| Fix issues, rerun checks | Each phase |
| Commit, tag, backup | Phase 0, 1, ... |
| /retro | Phase 7 |

## 14. Out of Scope (explicit)

- Modifying Dev/Wellfit source in this plan pass (plan agent only).
- Remote/GitHub integration until Phase 6 trigger conditions met.
- Counterparty-blocked items without registered input (Phase 5 gated).
- Certifications/compliance assertions (no prohibited claims).
- Pre-filling organization-specific facts in compliance docs.

## 15. Open Questions

1. Confirm Phase 0 git init uses `main` as default branch (recommended) — yes/no?
2. Confirm backup target location (encrypted external drive vs NAS) — which path?
3. Confirm whether generated tests should live in `crates/*/tests/generated/` (recommended) or a top-level `tests/`?

## 16. Recommended Next Action

Switch to an implementation-capable agent and execute Phase 0 (git init + scripts/check + backup) as the first slice, since it unblocks all subsequent phases and satisfies the local-first workflow's day-one backup requirement. Then proceed Phase 1.
