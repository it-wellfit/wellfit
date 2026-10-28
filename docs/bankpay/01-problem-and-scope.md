# E.Sun Bank ERP Integration — Problem Statement and Scope

**Document**: 01-problem-and-scope.md
**Version**: 1.0
**Source**: E.Sun Bank 企業 ERP 系統銀企直連 技術文件 V2.8
**Project**: Wellfit / Fuyi ERP (FlapsErp) accounts-payable voucher wave
**Status**: Draft

---

## 1. Problem Statement

The Fuyi ERP (FlapsErp) needs to settle accounts-payable (AP) vouchers through E.Sun Bank (玉山銀行) corporate net banking. Today, payment initiation, status tracking, and result reconciliation are performed manually via the bank's web portal, which is error-prone, slow, and not auditable. The integration must let ERP operators release payments, query transaction status, reconcile results, and print payment proofs without leaving the ERP, while satisfying the bank's security and compliance requirements.

### 1.1 Pain Points

| # | Pain Point | Impact |
|---|---|---|
| P1 | Manual payment key-in on bank portal duplicates ERP data | Re-keying errors, double payment risk |
| P2 | No real-time status; operators poll the portal | Delayed reconciliation, stale AP ledger |
| P3 | No automated audit trail linking ERP voucher ↔ bank transaction | Audit gaps, traceability failures |
| P4 | Foreign-currency payments require manual FX rate/fee handling | Rate slippage, fee misclassification |
| P5 | Payment proofs (PDF) fetched manually | Missing evidence in audit packs |

### 1.2 Critical Finding (OQ-E1 Resolved)

> **Integration is API-based via the bank's security component, NOT media file exchange.**

The original assumption that E.Sun Bank integration relies on media file exchange (file-based batch upload/download) is **incorrect**. V2.8 confirms the integration is conducted through the **玉山銀企直連安控元件** (security component) that exposes REST API endpoints. All payment initiation, status inquiry, and result retrieval flow through API calls secured by the component's certificates and hardware-token (eToken) authentication.

Implications:

- No file-based batch upload/download pipeline is required for the core flow.
- The ERP must implement an HTTP client layer talking to the local security component (default port 8443).
- Authentication is handled by the security component's installed certificate + corporate bank certificate + eToken, not by application-level passwords alone.
- Real-time transaction status is available via API, eliminating the need for media file reconciliation polling.
- The 422-char fixed-length BIG-5 media file format remains supported as an optional transport (`formatType=1`) but is not mandatory; `formatType=2` (JSON) is the recommended path.

---

## 2. Scope

### 2.1 In Scope (V2.8 Core)

| # | Capability | V2.8 API |
|---|---|---|
| 1 | TWD payment release (single / batch) | `POST /payment/createTwdPaymentInstruction` |
| 2 | TWD transaction status query | `POST /payment/retrieveTwdPaymentInstructionResult` |
| 3 | Transaction detail query (statement) | `POST /account/retrieveStatements` |
| 4 | FX pre-check (rate/fee validation) | `POST /payment/validateFxPaymentInstruction` |
| 5 | FX payment release | `POST /payment/createFxPaymentInstruction` |
| 6 | FX transaction status query | `POST /payment/retrieveFxPaymentInstructionResult` |
| 7 | Account balance query | `POST /account/retrieveAccountBalance` |
| 8 | TWD payment result print (PDF) | `POST /payment/retrieveTwdPaymentInstructionPrintResult` |

### 2.2 Out of Scope (this phase)

- Bank-side certificate issuance / eToken provisioning (bank responsibility).
- Custom ERP UI beyond existing FlapsErp screens.
- Mobile / browser-based release (desktop eToken only).
- Multi-bank abstraction (E.Sun only in this phase).

### 2.3 Assumptions

- A.1: The bank security component is installed on both the release-manager workstation and the ERP (API) server.
- A.2: E.Sun provides `SourceSystemId`, `ClientId`, and the enable code in writing.
- A.3: E.Sun provides the certificate files for 2-Way SSL.
- A.4: Network ports 8443 (internal) and 9443 (external) are open per §7 of deployment doc.
- A.5: The ERP operator holds a valid eToken and is registered in the bank's net-banking flow.

---

## 3. Success Criteria

| Metric | Target | Measurement |
|---|---|---|
| Payment release success rate | ≥ 99% (sandbox) | API result code `910000` ratio |
| Status query latency | < 3 s per batch (≤500 items) | p95 measured |
| Reconciliation accuracy | 100% voucher↔bank match | Automated traceable query |
| Audit completeness | 100% of API calls logged with payload hash | Ledger findings count |
| Test coverage | ≥ 32 unit tests green | `cargo test -p wellfit_bankpay` |

---

## 4. Stakeholders

| Role | Responsibility |
|---|---|
| Finance operator | Initiates payment release, reviews status |
| Release manager | Holds eToken, approves via net-banking flow |
| ERP administrator | Installs security component, manages certificates |
| E.Sun account manager | Provides credentials, sandbox access (OQ-E3) |
| Development team | Implements and maintains the integration |

---

## 5. References

- E.Sun Bank 企業 ERP 系統銀企直連 技術文件 V2.8 (in `references/`)
- `Dev/Wellfit/crates/wellfit_bankpay` — current implementation
- Plan: `Dev/.kilo/plans/1793034142477-esun-bankpay-integration-analysis.md`
