# Wellfit Plan Register

**Status:** Review register; not an approved implementation plan

**Reviewed:** 2026-11-02
**Last published verification snapshot:** [CURRENT_STATE.md](docs/project/CURRENT_STATE.md), dated 2026-10-25

This file replaces the stale, task-level plan previously stored at this path. It records plan ownership and conflicts; it does not certify the current code or test status. At review time, Wellfit had extensive uncommitted code and documentation changes. No build, test, or lint run was performed for this documentation review.

## Current authority and evidence

- Product entry and seven-crate inventory: [README.md](README.md) and [Cargo.toml](Cargo.toml). The README has local edits and must be reconciled before treating its claims as approved documentation.
- Last recorded test and release snapshot: [docs/project/CURRENT_STATE.md](docs/project/CURRENT_STATE.md). Its results are historical; they are not evidence for the current worktree.
- Project and document navigation: [docs/project/PROJECT_OVERVIEW.md](docs/project/PROJECT_OVERVIEW.md) is the Wellfit-wide overview; [docs/PROJECT_OVERVIEW.md](docs/PROJECT_OVERVIEW.md) is BankPay-specific; [docs/DOCUMENTATION_MAP.md](docs/DOCUMENTATION_MAP.md) maps the BankPay documentation.
- Plan corpus: [plans/](plans/). The 55 Markdown files include execution proposals, reference material, and ADRs; they are not 55 active plans.

## Plan disposition

| Workstream | Existing documents | Disposition and next gate |
|---|---|---|
| MTRIX / Wellfit completion | `1792486147134-mtrix-comprehensive-implementation.md`, `1792486147134-mtrix-deep-implementation.md`, `1792486147134-mtrix-full-completion-v2.md`, lifecycle and completion plans | Near-duplicate historical proposals use stale or conflicting baselines. Revalidate the worktree first, then retain only evidenced open requirements in one approved backlog. Do not execute their old test counts or LOCAL task lists as current facts. |
| Lifecycle / enterprise delivery / sync architecture | `docs/完整實作計畫.md`, `docs/COMPLETE_MULTI_LAYER_SOLUTION.md`, `docs/ENTERPRISE_PRODUCT_DELIVERY.md`, `docs/LOCAL_FIRST_PRODUCT_DELIVERY.md`, `docs/MULTIPLATFORM_SYNC_ARCHITECTURE.md`, `docs/ENTERPRISE_AI_AUTOMATION_RUNTIME.md` | Overlapping historical roadmaps and design proposals, not active task lists or delivery evidence. Preserve unique requirements, but use this register and the current project snapshot—not old “complete,” test-count, SLA, or technology claims—to authorize work. Consolidate only after checking inbound links and owner approval. |
| ISO / ISMS | `KNB-MASTER.md`, `KNB-ISO-MASTER.md`, `KNB-INDUSTRY-MASTER.md`, `KNB-PHASE2.md`, `KNB-ENG-SYSTEM.md`, `001`–`010`, full-scope ISMS proposals | **Recommended document structure (not owner approval):** use `KNB-MASTER.md` as a short governance/index document; `KNB-ISO-MASTER.md` as the sole detailed ISO-control reference; `KNB-INDUSTRY-MASTER.md` as a non-binding applicability proposal. The 001–010 set is historical input, not a second active plan. Keep implementation blocked until an accountable owner approves organizational scope, identity/SSO assumptions, applicable standards, and a single execution roadmap. |
| EKPI | EKPI plans currently stored in `Wellfit/plans/` and `Wellfit/.kilo/plans/` | These belong to a separate project and contain overlapping MVP / product-gap tracks. Do not treat them as Wellfit work. Reconcile only in `ekpi-platform/` against its charter and current evidence. |
| E.Sun / BankPay | `1793034142477-esun-flaps-ap-voucher-integration.md`, `docs/bankpay/`, `docs/esun-bank-flaps-erp-integration/` | **Protocol decision resolved from the checked-in E.Sun V2.8 technical PDF:** API through the bank security component is the primary integration; `formatType=2` is JSON and `formatType=1` is a fixed-width file for the documented one-debit/one-credit case. `docs/bankpay/` is the project decision/interface set. The four files under `docs/esun-bank-flaps-erp-integration/` are retained historical summaries with unverified completion and file-first claims; do not use them as acceptance evidence. Sandbox credentials, production account setup, actual CLI dispatch, and end-to-end bank validation remain blocked. |
| Returns / finance automation | `1793033568961-wellfit-returns-finance-automation.md` and E.Sun integration plans | Keep domain requirements separate, but identify shared ledger / release-gate tasks once source formats and decision owners are known. Returns assumptions and ownership remain unresolved. |
| Encoding / data formats | `1792994615266-encoding-modernization-plan.md`, `1792994615266-encoding-optimization-utf8-zh.md` | The old “missing UTF-8 baseline” is stale but the plan is only partly implemented: root `.editorconfig` and `scripts/encoding-tools.ps1` exist; the script lists Base91/Z85 commands, not Base122 or secure-pack. The root `.editorconfig` combines multiple properties on single lines and cannot be treated as valid EditorConfig; Wellfit `.gitattributes` lacks an encoding declaration and SyncMesh has no project `.gitattributes`. Keep one required baseline/config-validation task; defer speculative Base122, secure-envelope, and analytics-format work. |
| Workflow, toolchain, UI/UX, research, AI materialization, and skills | Remaining dated proposals in `plans/`; workspace proposals in `.kilo/plans/` | Proposals only unless an owner, current deliverable, dependency, source provenance, and acceptance evidence are supplied. The skill-pack task is blocked on the absent source archive/hash verification. Cross-project architecture and Flaps/JEV proposals are not implementation authority. ADRs and architecture/flowchart references are not execution plans. |

## Next actions (ordered)

1. Reconcile the current code, documentation, and test evidence in a clean review snapshot; update `CURRENT_STATE.md` only with commands actually run and results actually observed.
2. Keep the API-first E.Sun decision and remaining sandbox/production blockers maintained in the BankPay decision register; treat other copies as navigation summaries, not separate decisions.
3. Obtain accountable-owner approval for ISMS scope, identity assumptions, and applicable standards. Until then, the proposed document hierarchy above is navigation guidance only, not a compliance program.
4. Assign one owner and one approved execution plan per active workstream. Each plan must state status, last verified evidence/date, dependencies/blockers, acceptance criteria, and the document it supersedes.
5. Mark superseded proposals in place with a link to their replacement. Preserve ADRs and reference material; do not delete or bulk-move files until unique requirements and inbound links are reconciled.

## Evidence-based decision notes

- **Bank transport:** The checked-in [`V2.8 technical specification`](docs/bankpay/references/【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.pdf) documents API access through the E.Sun security component, JSON format (`formatType=2`), and a file format (`formatType=1`) restricted to the specified one-debit/one-credit transaction. Its separate 422-character media-file specification is not evidence that the whole integration should be file-drop based. Use the API as the canonical boundary; retain the media payload only where the API contract calls for it. The bank sandbox/account and live acceptance are still pending in [`08-open-questions-and-decisions.md`](docs/bankpay/08-open-questions-and-decisions.md).
- **AI/data boundary:** The Flaps architecture proposal explicitly says ERP data must remain local, while the Phase 2 plan proposes a cloud fallback. The consistent safe default is local-only inference; no cloud transfer until the data owner approves the data classification, provider, retention, and authorization controls in writing.
- **Standards claims:** Version/date assertions in the ISMS masters remain unverified against the official catalogue. The catalogue request returned HTTP 403 during this review; that is an access failure, not evidence for or against any edition. Do not use the tables as compliance evidence until each entry has a primary-source citation and applicability approval.
