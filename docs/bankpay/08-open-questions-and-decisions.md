# E.Sun Bank ERP Integration — Open Questions and Decisions

**Document**: 08-open-questions-and-decisions.md
**Version**: 1.0
**Source**: E.Sun Bank 技術文件 V2.8
**Status**: Draft

---

## 1. Open Questions

| ID | Question | Status | Resolution |
|---|---|---|---|
| OQ-E1 | Is integration file-based or API-based? | **Resolved** | API-based via security component (V2.8 §1). Media file (422-char BIG-5) is optional transport only. |
| OQ-E2 | Authentication mechanism? | **Resolved** | 2-Way SSL + eToken (FXML) via security component; no app-level password. |
| OQ-E3 | FlapsErp COM interop / sandbox credentials availability | **Pending** | Awaiting E.Sun account manager; ETA TBD. Blocks P1 exit criteria. Mitigation: build mock security component from V2.8 spec. |
| OQ-E4 | API response shape / result codes | **Resolved** | `rsStatus.code` 910000 = success; full table in `05-security-and-audit.md` §4. |

## 2. Decisions (ADRs)

### ADR-BP-001: API-first integration
- **Decision**: Use `formatType=2` (JSON) as primary transport; `formatType=1` (media file) only for legacy/bank-required cases.
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
- **Decision**: Build a mock security component from V2.8 spec for CI tests until sandbox (OQ-E3) available.
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
2. Implement HTTP client with certificate pinning (in progress — `http.rs` `EsunHttpClient` exists; pinning pending).
3. Implement RSA/signing layer (delegated to component — verify signing flow).
4. Add fixed-width 422-char media assembler (gap G-MEDIA) — implemented in `media.rs` (`assemble_twd_media_file`).
5. Complete TWD payment batch support (PayType 02/03 → paymentType 1/2).
6. Complete status code mapping (06, 99 → 1006/9999).
7. Implement cancellation flow.
8. Add integration test suite against mock component.

## 5. Traceability

Every requirement traces: requirement ID/PRD → system design/ADR → contract → ticket → implementation/Git commit → automated test → release evidence → retro. See `Wellfit/docs/requirements/TRACEABILITY.md`.
