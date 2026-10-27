# MASTER PROMPT v2.0 — Full Completion Plan (Operational & Validation Layer)

## Purpose

Complete every remaining project item to full, verifiable completion. The deterministic core is already built; this plan closes the operational, measurement, governance, and rollout layer so results are traceable, auditable, derivable, and explainable, and so the system can later upgrade to remote collaboration on explicit criteria.

## Current state (verified, not assumed)

- Platform crate `ekp_platform`: all modules declared and compiling (docs, rules, process, retrieval, snapshot, engine, validate, params, export, coverage_report, governance, governance_web, causal, agents, knowledge_graph, evidence, vector_search, fulltext, reranker, retrieval_v2, bpmn, xes_import, pm4py_sidecar, api, worm, tenant, audit, ocr, multimodal, formal_verification, runtime_verification, benchmark).
- Eval crate `ekp_eval`: types, metrics, systems, report, benchmark, failure_cases.
- `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo fmt -- --check`: clean in both crates (platform 143 unit tests pass).
- Hard gates H-1..H-6 all OK (engine `--out` audit `engine_hard_gates.json`).
- CLI flags working: `--selftest`, `--declared-rules-schema` (20 records clean), `--params` (fail-closed), `--export-legalruleml`, `--coverage-report` (55 cases), `--html` (zero JS).
- Seed 9 + casebook 46 cases (55 total) run end-to-end.

## What remains (operational layer)

None of these require new core engine logic. All are measurable, testable slices.

| Slice | Item | Why it matters |
|---|---|---|
| S1 | Measurable specification & target register (repository-resident) | Turns vague "top-tier" into numeric, reviewable targets |
| S2 | Quantifiable success metrics instrumentation | accuracy, processing time, manual handoff rate, error rate, cost, completion rate |
| S3 | Local-first workflow + offline-capable Git version control | Solve the real problem locally before any remote/collaboration step |
| S4 | Secrets & sensitive data handling policy | No secrets in repo; env-var boundary; CI secret scan |
| S5 | Verified local backups | Git bundle + restore drill; backup evidence in repo |
| S6 | Normal + edge-case test suites (R2-7: 50+ cases, double-annotation κ) | Expand eval corpus; inter-annotator agreement |
| S7 | Fixed local check scripts + CI matrix (Linux/Windows) | Reproducible baseline per quality-gates.md |
| S8 | TDD implementation slices + code review | Every change: failing test → minimal fix → review → rerun checks |
| S9 | Traceability: requirements ↔ decisions ↔ tests ↔ documentation | Auditable derivation of every public number |
| S10 | Remote collaboration upgrade criteria | Explicit trigger to move local Git → remote (GitHub) |
| S11 | Final validation & report | All gates, metrics evidence, release notes |

## Guiding principles (non-negotiable)

- `#![forbid(unsafe_code)]` on every Rust file; clippy `-D warnings` zero warnings; `cargo fmt -- --check` clean.
- Fail-closed: invalid/missing params, unknown rule IDs, draft rules, missing evidence → refuse, never guess.
- Evidence first: no fabricated integrations, permissions, approvals, scan results, or compliance claims.
- Presentation layer: semantic HTML + CSS-first, zero JavaScript (`<script>` count = 0).
- Local-first: every capability must work offline with local data before any remote integration is considered.
- PowerShell boundary: multi-command stitched with `;`, never `&&`/`|`/`&` in a single shell line.
- UTF-8 only for file writes; never PowerShell `Set-Content` without `-Encoding UTF8` (use Write tool / Python).

## S1 — Measurable specification & target register

Location: `docs/metrics/targets.md` (new, repository-resident). Format: a table with one row per metric: ID, metric, definition, measurement method, target value, status, owner, review date.

Proposed targets (business decision — **confirm with product owner before treating as acceptance**):

| ID | Metric | Definition | Proposed target | Status |
|---|---|---|---|---|
| M-1 | Verdict accuracy (deterministic cases) | cases where engine verdict == expected verdict / total deterministic cases | 100% | proposed |
| M-2 | Wrongful-allow rate | actual violations judged comply | 0 (hard gate H-2) | locked |
| M-3 | Correct refusal rate | insufficient-evidence cases correctly abstained | confirm | proposed |
| M-4 | Unwarranted refusal rate | definitive cases wrongly abstained | confirm | proposed |
| M-5 | Processing time per case | wall-clock ms for `decide` end-to-end | confirm (p50/p99) | proposed |
| M-6 | Manual handoff rate | cases requiring human resolution / total cases | confirm (<10%) | proposed |
| M-7 | Error rate | failed loads / total loads | confirm (<1‰) | proposed |
| M-8 | Cost per case | token/compute cost estimate per adjudication | confirm | proposed |
| M-9 | Completion rate | cases reaching a terminal verdict/escalate/abstain | 100% | proposed |

Acceptance: target register exists; every target has a named owner and a measurement command; numeric targets marked "proposed" require product-owner sign-off before they block a release.

## S2 — Metrics instrumentation

Add to `ekp_eval::metrics` (and expose via `report.rs`): per-case latency, verdict distribution, refusal breakdown, citation recall, anchor resolvability (H-4), wrongful-allow (H-2), correct refusal, unwarranted refusal, cost estimate. Emit into `report.json`/`report.md` per run. Validation: metrics present in every ablation/run report; values derivable from `engine_snapshots.json` + `engine_hard_gates.json`.

## S3 — Local-first workflow + offline Git

Policy (`docs/operations/local-first-workflow.md`):
- All development occurs on the local clone; no feature requires network access to function.
- Git is the single source of truth; commits are made locally and can be pushed later.
- Offline-capable: `git bundle` for transport when no remote is available; `git status`/`git diff`/`git log` usable offline.
- No behavior gated on remote API, external LLM, or cloud service in the core path (PM4Py sidecar is optional oracle only; LLM = proposal, engine = decision).
- Regression checklist and quality gates run entirely offline.

## S4 — Secrets & sensitive data handling

Policy (`docs/operations/secrets-policy.md`):
- No secrets, keys, passwords, or tokens committed to the repo (enforced by CI secret scan on commit range).
- Configuration via environment variables (`EKP_*`) and local config files excluded from version control via `.gitignore`.
- Provide `.env.example` with placeholder names only (no real values).
- Governance web prototype is local-only, no enterprise authentication, no external-source permission enforcement (per AGENTS.md).
- Sensitive data (org facts, PII) never prefilled in compliance deliverables (constraint `no_org_facts_prefilled`).

## S5 — Verified local backups

Procedure (`docs/operations/local-backup.md`):
- Weekly `git bundle create` of the full repo to a local backup directory outside the workspace.
- Restore drill: clone bundle to a temp path, run `cargo test --locked` + `cargo clippy --locked --all-targets -- -D warnings`, confirm exit 0.
- Record each backup + restore result in `docs/operations/backup-log.md` (date, bundle path, restore result, operator).
- Backup is verified only when the drill passes; otherwise block release.

## S6 — Test suites (normal + edge cases)

- Expand `eval/cases` to 50+ cases (R2-7 carryover): add edge cases for forbidden pairs, stale/future/pinned rule versions, draft/unregistered rule IDs, injection directives, unsorted/equal timestamps, duplicate activities, close-before-execute, multi-rule severity precedence.
- Double-annotation κ: two annotators independently label expected verdicts; report Cohen's κ; threshold confirm (e.g., κ ≥ 0.80) before using as ground truth.
- Every new case has a regression test entry; failures map to `AT-01..AT-31` acceptance matrix where applicable.
- Validation: `cargo test --locked` green; new cases run through `--selftest` / `--out` ablation.

## S7 — Fixed local check scripts + CI matrix

- `scripts/check.ps1` and `scripts/check.sh`: run the full baseline from quality-gates.md in each crate dir (fmt, clippy, test, selftest, schema). Fixed, deterministic, offline-runnable.
- CI (`.github/workflows/ci.yml`): matrix on ubuntu-latest + windows-latest; security job audits both Cargo lockfiles and scans commit range for secrets; require all three jobs to pass before merge.
- Validation: run check scripts manually on both platforms; attach output to PR.

## S8 — TDD slices + code review

Process per `docs/development/README.md`:
1. Issue/spec with acceptance criteria + risk tier.
2. Write a failing regression test first (TDD).
3. Minimal implementation to pass; rerun affected checks.
4. Code review (PR): problem/outcome, issue/spec links, risk tier, acceptance evidence, commands/results, UI a11y evidence where relevant, data/compat impact, rollback approach.
5. Fix review findings, rerun affected checks, then merge (requires CI success + human approval; high-risk needs independent reviewer).

Definition of Done (per quality-gates / regression-checklist): acceptance criteria pass; every Must requirement has traceable test or review evidence; affected existing behavior covered; risk-appropriate checks pass; review findings fixed and checks rerun; docs and known limits current; PR links source issue/spec.

## S9 — Traceability register

Repository-resident mapping (one source of truth): `docs/traceability/register.md`.
Each entry: requirement/issue ID → spec/ticket → code change(s) → test evidence → PR/decision. Every public number in reports must be derivable from a named, documented SQL view or persisted `validation_findings` row count (per ADR: metric definitions owned by ledger; in-memory counts agree with persisted counts). No number invented.

## S10 — Remote collaboration upgrade criteria

Move local Git → remote (GitHub) ONLY when ALL of the following hold:
1. GitHub repo URL and team access are available and configured.
2. Branch protection is enabled on the default branch.
3. CI (validate ubuntu + validate windows + security) passes on a push/PR.
4. A human approval process exists; high-risk changes have an independent QA/security/domain reviewer.
5. Secrets policy (S4) and secret scanning are enforced in CI.
6. The local-first baseline (S3) is documented and verified offline first.

Until criteria 1–6 are met, collaboration stays local-only (offline Git, bundle transport). Do not prematurely remoteize.

## S11 — Final validation & report

Run the full baseline (quality-gates.md) in both crates, attach output:
- `cargo fmt -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`
- `cargo run --locked -- --selftest` (eval) and `cargo run --locked --bin ekp_platform -- --selftest` + `--declared-rules-schema` (platform)
- Hard gates H-1..H-6 ALL OK
- Secrets scan clean; zero `<script>` in generated HTML
- Backup restore drill passed (S5)
- Target register (S1) reviewed by product owner

Output: `docs/operations/final-validation-report.md` with dates, commands, results, and open items.

## Open decisions (require product/team input, not plan-agent)

- Numeric target values for M-3, M-4, M-5, M-6, M-7, M-8 (M-1/M-2/M-9 are locked at 100%/0/100%).
- Backup schedule cadence and backup storage path.
- κ threshold for double-annotation.
- Exact remote (GitHub) org/repo and access model when criteria are met.

## Out of scope for this plan

- New core engine logic (already complete).
- Enterprise authentication, external connectors, AI workflow drafts (prototype remains local-only).
- PM4Py sidecar production integration (remains optional R3 oracle per hash-artifact contract).

## Validation summary

Each slice is validated by: (a) targeted tests, (b) the fixed local check scripts, (c) CI matrix on both platforms, (d) hard-gate audit where applicable, (e) code review + human approval. Results are recorded in the traceability register and final-validation report so every claim is derivable from an artifact in the repo.