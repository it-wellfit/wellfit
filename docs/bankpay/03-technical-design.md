# E.Sun Bank ERP Integration — Technical Design

**Document**: 03-technical-design.md
**Version**: 1.0
**Source**: E.Sun Bank 技術文件 V2.8
**Status**: Draft

---

## 1. Architecture Overview

```
┌─────────────────┐  HTTPS (TLS 1.2+, 2-Way SSL)  ┌──────────────────────┐
│  Wellfit ERP    │ ─────────────────────────────► │  E.Sun Security      │
│  (Rust core)    │  localhost:8443 (internal)     │  Component (local)   │
│                 │  eospt/eospu/eosp (external)  │                      │
└────────┬────────┘                               └──────────┬───────────┘
         │                                                   │
         │  BankPay Engine (wellfit_bankpay crate)            │  eToken sign
         │  - encoding (CP950/Big5/Big5-HKSCS/UTF-8)         │  + forward
         │  - ColumnContract validation                      │
         │  - ScaledDecimal amounts                          ▼
         │  - IdempotencyLedger (SQLite)             ┌──────────────────┐
         │  - ReleaseGate state machine              │  E.Sun Bank      │
         │  - traceable query                        │  Backend         │
         └──────────────────────────────────────────►└──────────────────┘
```

### 1.1 Crate Layout

```
Wellfit/crates/wellfit_bankpay/
├── src/
│   ├── lib.rs          # public re-exports
│   ├── api.rs          # V2.8 API request/response models
│   ├── bankpay.rs      # engine, domain models, media assembly
│   ├── encoding.rs     # CP950/Big5/Big5-HKSCS/UTF-8 conversion
│   ├── http.rs         # EsunHttpClient (ureq + rustls, 2-way SSL)
│   ├── media.rs        # 422-char fixed-width BIG-5 media assembler
│   ├── query.rs        # traceable query (voucher↔tx↔batch↔sha256)
│   └── release.rs      # ReleaseGate wrappers
└── tests/              # 32 unit tests
```

### 1.2 Dependencies

- `wellfit_core`: `ColumnContract`, `IdempotencyLedger`, `ReleaseGate`, `ScaledDecimal`, `Finding`.
- `encoding_rs`: Big5 / Big5-HKSCS codecs.
- `serde`, `sha2`: serialization and content hashing.
- HTTP client: `ureq` + `rustls` (2-way SSL, cert pinning pending) in `http.rs`.

---

## 2. Data Flow (TWD Payment)

1. **Generate TxnId** — `msgId` = 統編 + YYYYMMDD + 7 碼流水; persist with status `pending`.
2. **Call `/auth`-equivalent** — security component handles cert + eToken; ERP supplies `Source-System` header.
3. **Build payload** — JSON (`formatType=2`) with `twdTxList`; validate each field (length, required, format).
4. **RSA/SSL signing** — performed by security component using installed corporate certificate.
5. **POST `/payment/createTwdPaymentInstruction`**.
6. **Map `rsStatus.code`** — `910000` → success; else failure with `moreInfo` detail.
7. **Persist** — `BankTxnId` (echoed txSeq), status, `actualTxDate/Time`, `actualChargeFee`.
8. **Poll** — if status non-terminal, poll `/payment/retrieveTwdPaymentInstructionResult`.
9. **Terminal** — notify finance, update AP ledger, record audit finding.
10. **Optional print** — fetch PDF via print API for success/failed transactions.

---

## 3. Media File Format (optional, formatType=1)

Although the core flow is API-based, the bank still accepts a 422-char fixed-length BIG-5 media file (base64-encoded in the `content` field). Spec:

| Property | Value |
|---|---|
| Encoding | BIG-5 |
| Record length | 422 chars (fixed) |
| Max records | 1000 (暫定) |
| Transport | Base64 in `content` field (`formatType=1`) |

Field layout (start position, length):

| # | Field | Pos | Len | Required | Padding |
|---|---|---|---|---|---|
| 1 | payerAccountNo | 1 | 16 | Y | left, space |
| 2 | payerId | 17 | 15 | Y | left, space |
| 3 | payeeBankId | 32 | 3 | Y | — |
| 4 | payeeBranchId | 35 | 4 | Y | left, space |
| 5 | payeeAccountNo | 39 | 16 | Y | right, zero |
| 6 | payeeName | 55 | 80 | Y | left, space |
| 7 | payeeId | 135 | 15 | N | left, space |
| 8 | txDate | 150 | 8 | Y | YYYYMMDD |
| 9 | txAmt | 158 | 13 | Y | right, zero (>0) |
| 10 | chargeFeeType | 171 | 1 | Y | 0=外收, 1=內扣 |
| 11 | memo | 172 | 80 | N | left, space |
| 12 | notifyMethod | 252 | 1 | N | 1/2/3/9 |
| 13 | notifyEmail | 253 | 50 | N | left, space (note: JSON spec says 400; media uses 50) |
| 14 | notifyFax | 303 | 20 | N | left, space |
| 15 | writeOffNo | 323 | 40 | N | left, space |
| 16 | remark | 363 | 38 | N | left, space |
| 17 | txSeq | 401 | 22 | Y | left, space |
| 18+ | detailList (invoiceDate 8, invoiceNo 20, invoiceAmt 10, invoiceReturn 10, invoiceActualAmt 10, invoiceRemark 40) | 423+ | 98 each | N | — |

The Wellfit engine's `media.rs` already implements `assemble_twd_media_file`, which emits the V2.8-compliant 422-char fixed-width BIG-5 media file (base64-encoded into the API `content` field for `formatType=1`). CSV output is not used.

---

## 4. Amount Handling

- All amounts carried as integer cents via `ScaledDecimal` (wellfit_core).
- `txAmt` field is string in JSON (e.g. `"12345"`), numeric in media.
- FX amounts support 2 decimals (`15.2`).
- `invoiceAmt`, `invoiceReturn`, `invoiceActualAmt` are `18.2` in JSON, 10-digit zero-padded in media.

---

## 5. Idempotency & Ledger

- `IdempotencyLedger` (SQLite, `:memory:` for tests / file for prod).
- File batch registration: `register_file_batch(batch_id, type, filename, bytes, 0)` — duplicate content rejected.
- Order lock: `check_and_lock_order(key, batch_id, amount)` where key = `VOUCHER_NO:AMOUNT:BATCH_DATE`.
- Every API call recorded as a `Finding` (rule codes `VR-BANKPAY-01..03`, `VR-BANKPAY-QUERY`) for traceability.
- Quarantine: unparseable rows logged via `log_quarantine`.

---

## 6. Release Gate State Machine

```
DraftGenerated → VerifiedPass / Quarantined → AuthorizedDispatch → Completed / Failed / DispatchUncertain
```

- `verify_payload(is_valid, reason)` — two-way door; checks isolated rows + encoding errors.
- `dispatch(transport, recipients)` — one-way door; invokes security component call.
- `reconcile_uncertain_dispatch(status_check)` — resolves `DispatchUncertain` by re-querying bank.
- All transitions recorded in `history: Vec<TransitionRecord>`.

---

## 7. Error Handling Strategy

| Layer | Strategy |
|---|---|
| Transport (TLS/conn) | Retry with exponential backoff; circuit breaker after N failures |
| HTTP non-2xx | Log full payload, escalate |
| `rsStatus.code != 910000` | Map to internal error enum; `moreInfo` parsed for per-tx detail |
| Encoding errors | Quarantine file, `had_errors=true` blocks dispatch |
| Isolated rows | Quarantine, batch not dispatched |

---

## 8. Security Design

See `05-security-and-audit.md`. Summary:

- TLS 1.2+ with 2-Way SSL (client cert + server cert).
- Certificate pinning on HTTP client.
- `Source-System` header = agreed `SourceSystemId`.
- All payloads logged with SHA-256 hash (never raw secrets).
- eToken signing delegated to security component (ERP never holds private key).

---

## 9. Traceability

`query(engine, kind, key)` cross-references:
- `voucher` → voucher number → processed flag + findings.
- `tx` → bank reference → cross-reference findings.
- `batch` → batch id → registered batch count + findings.

Every query recorded as `VR-BANKPAY-QUERY` finding, making the act of tracing auditable.
