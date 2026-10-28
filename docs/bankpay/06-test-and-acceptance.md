# E.Sun Bank ERP Integration — Test and Acceptance

**Document**: 06-test-and-acceptance.md
**Version**: 1.0
**Source**: E.Sun Bank 技術文件 V2.8
**Status**: Draft

---

## 1. Test Strategy

| Layer | Tooling | Scope |
|---|---|---|
| Unit tests | `cargo test -p wellfit_bankpay` | Engine, encoding, column contract, ledger, release gate, query |
| Integration tests | Mock security component (HTTP server) | End-to-end API flow |
| Contract tests | V2.8 field spec | Field length/required/format validation |
| Manual UAT | Bank sandbox (OQ-E3) | Real security component |

Current baseline: **18 unit tests** in `wellfit_bankpay` (encoding round-trip, export dedup, isolation, media SHA-256, import findings, traceability chain, release gate).

---

## 2. Test Cases — Core Payment (TWD)

| ID | Scenario | Expected |
|---|---|---|
| TC-P01 | Single TWD payment, valid beneficiary | `910000`, txRecordList echoed, status `0000`/`1001` |
| TC-P02 | Batch (paymentType=2) 3 beneficiaries | `910000`, dataCount=3 |
| TC-P03 | Duplicate txSeq | `910069` with per-tx error in `moreInfo` |
| TC-P04 | Invalid beneficiary account | `910069`, `moreInfo` lists txSeq |
| TC-P05 | Insufficient balance | status `4000` or result code bank-side |
| TC-P06 | verificationType=2 (net-banking flow) | Requires flowInfo; release manager approves |
| TC-P07 | Payment with invoice detailList | detailList echoed |
| TC-P08 | Auto-delay (txDate on holiday) | `autoDelay=1` |

## 3. Test Cases — Status Query

| ID | Scenario | Expected |
|---|---|---|
| TC-S01 | Query single txSeq, status 0000 | actualTxDate/Time/ChargeFee populated |
| TC-S02 | Query status 4000 | sysErrorDesc populated |
| TC-S03 | Query 501 items | `910073` (max 500) |
| TC-S04 | Query non-existent txSeq | status `9999` 查無資料 |
| TC-S05 | Batch status poll for non-terminal | Poll until terminal |

## 4. Test Cases — Statement Query

| ID | Scenario | Expected |
|---|---|---|
| TC-ST01 | Date range, 50 rows | Returns ≤50, no continuation |
| TC-ST02 | Date range, 500 rows | Continue with queryStartIndex=501 |
| TC-ST03 | Currency filter | Only matching curr rows |
| TC-ST04 | orderDescType=1 (new→old) | firstDeptTxDtlSeqNo positioning |

## 5. Test Cases — FX

| ID | Scenario | Expected |
|---|---|---|
| TC-FX01 | FX pre-check valid | status `3000` 檢核成功 |
| TC-FX02 | FX pre-check invalid data | status `3001`, sysErrorDesc |
| TC-FX03 | FX release cross-border | transactionReferenceNo returned |
| TC-FX04 | FX status poll → 1031 電文已發送 | Terminal (Y) |
| TC-FX05 | FX status 1030 扣款已完成 | Non-terminal, poll |

## 6. Test Cases — Balance & Print

| ID | Scenario | Expected |
|---|---|---|
| TC-B01 | TWD balance (currType=1) | actBalance, avlBalance |
| TC-B02 | FX balance (currType=2) | Per-currency rows |
| TC-PRT01 | Print for success tx | Base64 PDF in content |
| TC-PRT02 | Print 6 items | Split into 2 calls (max 5) |

## 7. Test Cases — Encoding & Media

| ID | Scenario | Expected |
|---|---|---|
| TC-ENC01 | UTF-8 ↔ Big5 round-trip | No loss |
| TC-ENC02 | CP950 label | "CP950 (Big5-HKSCS)" |
| TC-MED01 | Assemble 422-char record | Length 422, BIG-5 |
| TC-MED02 | >1000 records | Reject / split |
| TC-MED03 | Base64 content in payment API | Decodes to valid media |

## 8. Test Cases — Idempotency & Release Gate

| ID | Scenario | Expected |
|---|---|---|
| TC-ID01 | Re-export same batch id | Rejected (duplicate) |
| TC-ID02 | Re-lock same voucher key | Quarantined (isolated) |
| TC-RG01 | verify_payload(true) → dispatch | Completed |
| TC-RG02 | verify_payload(false) → dispatch | Rejected (Quarantined) |
| TC-RG03 | DispatchUncertain → reconcile(true) | Completed |

## 9. Acceptance Criteria

- [ ] All 8 APIs implemented against mock security component.
- [ ] All unit tests (≥18) green; clippy zero warnings.
- [ ] Full result code table (910000–999999) handled.
- [ ] Full transaction status table (0000–9999) mapped.
- [ ] TWD payment fields validated per V2.8 lengths/required.
- [ ] FX pre-check + release + status flow end-to-end.
- [ ] Statement pagination verified with ≥500-row fixture.
- [ ] Balance query for TWD and FX.
- [ ] Print API returns decodable PDF.
- [ ] Audit: every API call logged with payload hash.
- [ ] Traceable query returns findings for voucher/tx/batch.
- [ ] UAT against bank sandbox (OQ-E3) — pending credentials.

## 10. Exit Criteria per Phase

- **P0**: token acquisition + health check succeed against real component.
- **P1**: single + batch TWD payment end-to-end against sandbox.
- **P2**: reconciliation report matches bank statement.
- **P3**: all 8 docs complete; CI green; mock component in CI.
