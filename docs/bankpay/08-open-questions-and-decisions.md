# E.Sun Bank ERP Integration — Open Questions and Decisions

**Document**: 08-open-questions-and-decisions.md
**Version**: 1.0
**Source**: E.Sun Bank 技術文件 V2.8
**Status**: Draft

---

## 1. Open Questions

| ID | Question | Status | Resolution |
|---|---|---|---|
| OQ-E1 | Is integration file-based or API-based? | **Resolved** | API-based via security component (V2.8 §1). `formatType=2` is JSON; `formatType=1` is a fixed-width payload only for the documented one-debit/one-credit case. This is not a file-drop workflow. |
| OQ-E2 | Authentication mechanism? | **Resolved** | 2-Way SSL + eToken (FXML) via security component; no app-level password. |
| OQ-E3 | FlapsErp COM interop / sandbox credentials availability | **Pending** | Awaiting E.Sun account manager; ETA TBD. Blocks P1 exit criteria. Mitigation: build mock security component from V2.8 spec. |
| OQ-E4 | API response shape / result codes | **Resolved** | `rsStatus.code` 910000 = success; full table in `05-security-and-audit.md` §4. |

## 2. Decisions (ADRs)

### ADR-BP-001: API-first integration
- **Decision**: Use `formatType=2` (JSON) as the normal API payload; use `formatType=1` only for the documented one-debit/one-credit case.
- **Rationale**: Real-time status, no file pipeline, simpler reconciliation.

### ADR-BP-002: Security component as trust boundary
- **Decision**: ERP delegates signing to the bank's security component; ERP never holds private keys or eToken.
- **Rationale**: Compliance, non-repudiation, reduced key-management burden.

### ADR-BP-003: Idempotency via ledger
- **Decision**: Lock each voucher by `VOUCHER_NO:AMOUNT:BATCH_DATE` before dispatch; reject duplicates.
- **Rationale**: Prevents double payment; aligns with bank's `txSeq` uniqueness.

### ADR-BP-004: Rust-first implementation
- **Decision**: Implement in `wellfit_bankpay` crate (Rust), no JS.
- **Rationale**: Project constraint (Rust-first fullstack); memory safety; reuse of wellfit_core.

### ADR-BP-005: Mock component for CI
- **Decision**: Build a mock security component from V2.8 spec for CI tests until sandbox (OQ-E3) is available. This remains a proposal; no mock-backed integration acceptance is recorded.
- **Rationale**: Unblocks development; tests contract compliance.

## 3. Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Sandbox unavailable (OQ-E3) | Blocks P1 exit | Mock component from V2.8 spec; defer real validation |
| Certificate expiry mid-integration | Auth failures | Expiry monitoring + 30-day renewal alert |
| Batch partial failure (per-tx errors) | Ledger inconsistency | Per-beneficiary status tracking via `moreInfo` |
| Security component version drift | API breakage | Pin to V2.8; verify version on startup |
| Field length mismatch (JSON vs media) | Data truncation | Validate against both specs; note notifyEmail 400 (JSON) vs 50 (media) |
| FX declaration code errors | Rejection | Pre-validate declaration codes before release |

## 4. Pending Actions

1. Resolve OQ-E3 with E.Sun account manager (sandbox credentials).
2. Verify and implement actual CLI-to-security-component HTTP dispatch; the quality review records that the current CLI transport closure returns `Ok(true)` without sending a request ([`QUALITY_REVIEW_REPORT.md`](../QUALITY_REVIEW_REPORT.md), QR-06).
3. Confirm TLS/client-certificate requirements against the bank specification and owner policy; certificate pinning is not implemented, and no active implementation status is evidenced.
4. Verify the security component's signing flow in an approved test environment; do not add ERP-side private-key/signing behavior without an explicit requirement.
5. Reconcile TWD batch payment types and status-code mappings against V2.8, then record source/test evidence; the current traceability matrix is not proof of bank acceptance.
6. Build or use an isolated mock component only after its contract and test scope are approved; no mock-backed bank integration acceptance is recorded.
7. Keep fixed-width media assembly as a supported payload format only for the documented case; source presence does not establish a successful bank dispatch.
8. Do not treat cancellation as an active requirement until scope and bank support are confirmed.

## 5. Traceability

Target traceability for approved work: requirement ID/PRD → system design/ADR → contract → ticket → implementation/commit → automated test → release evidence → retrospective. The current BankPay documents do not yet establish this complete chain; use [`TRACEABILITY_MATRIX.md`](../TRACEABILITY_MATRIX.md) as a partial source/test map and record missing evidence as open.
