# E.Sun Bank ERP Integration — Business Workflow

**Document**: 02-business-workflow.md
**Version**: 1.0
**Source**: E.Sun Bank 技術文件 V2.8
**Status**: Draft

---

## 1. Actors

| Actor | Description |
|---|---|
| ERP Operator | Creates AP voucher batch in Fuyi ERP, triggers payment release |
| Release Manager | Holds eToken; performs 2nd-factor approval on workstation |
| Security Component | 玉山銀企直連安控元件 — local HTTPS service (port 8443) |
| E.Sun Bank Backend | Processes payments, returns status |

---

## 2. End-to-End Payment Workflow (TWD)

```
┌────────────┐  1. Select AP vouchers   ┌──────────────┐
│  ERP Core  │ ───────────────────────► │  BankPay     │
│            │                          │  Engine      │
└────────────┘                          └──────┬───────┘
                                               │ 2. Generate msgId (ERP統編+YYYYMMDD+7碼流水)
                                               │ 3. Validate beneficiaries, amounts
                                               │ 4. Build payload (JSON or 422-char media)
                                               │ 5. POST /payment/createTwdPaymentInstruction
                                               ▼
                                    ┌──────────────────────┐
                                    │  Security Component  │  (localhost:8443, 2-Way SSL)
                                    └──────┬───────────────┘
                                           │ 6. eToken sign + forward
                                           ▼
                                    ┌──────────────────────┐
                                    │  E.Sun Bank Backend  │
                                    └──────┬───────────────┘
                                           │ 7. Result code 910000 / other
                                           ▼
                                    ┌──────────────────────┐
                                    │  ERP: map status,    │
                                    │  persist BankTxnId,   │
                                    │  audit log           │
                                    └──────┬───────────────┘
                                           │ 8. If non-terminal → poll status
                                           │ 9. On terminal → notify finance, update ledger
                                           ▼
                                    ┌──────────────────────┐
                                    │  Optional: print PDF │  /payment/retrieveTwdPaymentInstructionPrintResult
                                    └──────────────────────┘
```

### 2.1 Steps Detail

1. **Voucher selection** — operator selects a batch of AP vouchers; engine validates column contract (`VOUCHER_NO`, `VENDOR_CODE`, `AMOUNT`, `BANK_ACCOUNT` required).
2. **Idempotency lock** — each voucher keyed `VOUCHER_NO:AMOUNT:BATCH_DATE` is locked in `IdempotencyLedger`; duplicates are quarantined.
3. **Payload assembly** — `formatType=2` (JSON) recommended; `formatType=1` (422-char BIG-5 media, base64 in `content`) for legacy.
4. **Payment release** — `POST /payment/createTwdPaymentInstruction` with `rqHeader` (msgId, requestTime, customerIxd, sourceSystem) and `rqData` (formatType, paymentType, verificationType, flowInfo, content).
5. **verificationType**:
   - `1` = ERP 驗證放行 (ERP validates & releases)
   - `2` = 網銀驗證放行 (requires flowInfo: operatorName, flowName, flowKey; release manager approves in net banking)
6. **Result handling** — `rsStatus.code == 910000` → success, parse `rsData.txRecordList`; else failure, inspect `moreInfo` (per-tx error detail).
7. **Status polling** — for transactions whose status is not terminal, call `/payment/retrieveTwdPaymentInstructionResult` (max 500 items per call).
8. **Reconciliation** — match bank results to vouchers via `txSeq`; update AP ledger; record findings.
9. **Print** — for success/failed transactions, fetch PDF via `/payment/retrieveTwdPaymentInstructionPrintResult` (max 5 items per call).

---

## 3. Foreign-Currency Payment Workflow

1. **Pre-check** — `POST /payment/validateFxPaymentInstruction` validates beneficiary data, computes applicable exchange rate and fees; returns `status` `3000` (檢核成功) or `3001` (檢核失敗) with `sysErrorDesc`.
2. **Release** — `POST /payment/createFxPaymentInstruction` with `verificationType` 1/2; returns `transactionReferenceNo` (bank-side) and initial status.
3. **Status polling** — `POST /payment/retrieveFxPaymentInstructionResult`; statuses include `1030` (扣款已完成), `1031` (電文已發送), `1032` (取消發電).
4. **Reconciliation** — match via `txSeq`; record FX rate, fee, value date.

---

## 4. Account & Statement Queries

- **Balance** — `POST /account/retrieveAccountBalance` with `currType` (1=TWD, 2=FX); returns `actBalance`, `avlBalance`, `todayExgAmt`, `nextExgAmt` per account.
- **Statement** — `POST /account/retrieveStatements` with date range, account, currency, pagination (`queryStartIndex`, `firstDeptTxDtlSeqNo`); max 500 rows per call; continue until fewer than 500 returned.

---

## 5. Status Code Semantics

| Code | Meaning | Terminal? | ERP Action |
|---|---|---|---|
| 0000 | 交易成功 | Yes | Mark paid, close voucher |
| 1000 | 預約完成 | No | Wait for auto-execution |
| 1001 | 交易處理中 | No | Poll |
| 1002 | 交易已取消 | Yes | Re-open voucher |
| 1003 | 交易處理中(待轉通匯) | No | Poll |
| 1004 | 交易處理中(餘額不足多扣) | No | Alert finance, poll |
| 1005 | 交易處理中(天候因素暫停匯款) | No | Poll next business day |
| 1006 | 待審核 | No | Wait for net-banking approval |
| 1007 | 待放行 | No | Wait for release manager |
| 1008 | 經辦退件待處理 | No | Operator rework in net banking |
| 1030 | 扣款已完成 (FX) | No | Poll |
| 1031 | 電文已發送 (FX) | Yes | Mark sent |
| 1032 | 取消發電 (FX) | Yes | Re-open voucher |
| 3000 | 檢核成功 (FX pre-check) | Yes | Proceed to release |
| 3001 | 檢核失敗 (FX pre-check) | Yes | Fix data, re-validate |
| 4000 | 交易失敗 | Yes | Quarantine, notify finance |
| 9999 | 查無資料 | Yes | Investigate missing txSeq |

---

## 6. Business Rules

- BR-1: `msgId` = 企業集團統編 (8–10 碼) + YYYYMMDD + 7 碼流水序號 (total 23–25).
- BR-2: A transaction with `txDate` ≤ system date and status not in {0000, 4000} must be re-queried.
- BR-3: `txSeq` must be unique per ERP payment draft (third-party transaction number).
- BR-4: FX `paymentType=2` (one debit, many credits) requires identical payer account, currency, and payment date across all items.
- BR-5: Statement pagination continues while returned rows == 500 and code == 910000.
- BR-6: Print API limited to 5 items per call.

---

## 7. Release Gate Integration

The `BankpayEngine` wraps a `ReleaseGate` state machine:

`DraftGenerated → VerifiedPass / Quarantined → AuthorizedDispatch → Completed / Failed / DispatchUncertain`

- Two-way door: `verify_payload` checks for isolated rows / encoding errors.
- One-way door: `dispatch` invokes the transport (security component call).
- On `DispatchUncertain` (timeout), reconcile via `reconcile_uncertain_dispatch` by re-querying bank status.
