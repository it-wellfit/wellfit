# E.Sun Bank ERP Integration — Security and Audit

**Document**: 05-security-and-audit.md
**Version**: 1.0
**Source**: E.Sun Bank 技術文件 V2.8
**Status**: Draft

---

## 1. Prerequisites (Enterprise)

| # | Item | Description |
|---|---|---|
| 1 | 企業對外連線 IP 申請 | Outbound IP whitelisted by bank firewall |
| 2 | 企業身分與帳戶約定 | Apply as corporate net-banking identity; agree transfer limits, designated accounts |
| 3 | 交易放行憑證 (eToken) | Hardware token (FXML) for release signing |
| 4 | 企業設備資訊提供 | Hostname + MAC address of release-manager workstation and ERP/API server |
| 5 | 企業網路開通 | Port 8443 internal (ERP server → release-manager workstation); port 9443 external (both → bank) |
| 6 | 安裝安控元件 | Install as administrator on both workstation and ERP server |
| 7 | 安控元件綁定 | First launch requires: 企業集團統編, `ClientId` (from bank email), 啟用碼 (from bank password letter) |
| 8 | 玉山提供憑證檔 | Bank cert for message source verification |
| 9 | ERP 提供憑證檔 | ERP cert for message source verification |

### 1.1 Environments

| Env | Domain |
|---|---|
| DEV (暫) | `http://IP:8080` |
| Test | `http://IP:8080` |
| UAT | `eospu.testesunbank.com.tw` (port 9443) |
| Prod | `eosp.esunbank.com.tw` (port 9443) |

---

## 2. Transport Security

- Protocol: HTTPS, TLS 1.2+ only.
- Mode: **2-Way SSL** — client certificate + server certificate mutual verification.
- Certificate pinning on the ERP HTTP client (gap: implement in `client.rs`).
- `Source-System` header carries agreed `SourceSystemId`.
- eToken signing delegated to the security component; ERP never holds the private key.

---

## 3. Authentication Flow

The security component mediates authentication:

1. ERP calls `https://localhost:8443/...` with client cert.
2. Security component validates cert + SourceSystemId.
3. For release APIs, security component triggers eToken signing on the release-manager workstation.
4. Component forwards signed request to bank backend over 9443.
5. Bank verifies signature, processes, returns result.

No application-level password is used; identity is device + certificate + eToken.

---

## 4. API Result Codes (Appendix 六.(一))

| Code | Message | Handling |
|---|---|---|
| 910000 | 訊息回覆成功 | Success — parse `rsData` |
| 910001 | 查詢條件不足，無法查詢 | Reject, fix query params |
| 910004 | 查無資料 | Investigate (missing txSeq) |
| 910032 | 未提供查詢主機來源，無法查詢 | Check SourceSystemId |
| 910063 | 必要資訊不足，無法進件 | Fix payload, retry |
| 910064 | 尚未約定第三方交易放行業務 | Contact bank |
| 910065 | 驗簽失敗 | Re-install cert, escalate |
| 910066 | CN 比對失敗 | Certificate mismatch |
| 910067 | 資料解密失敗 | Payload encoding issue |
| 910068 | 交易資料寫入銀行失敗 | Retry with backoff |
| 910069 | 交易資料檢核有誤 | Per-tx errors in `moreInfo` |
| 910070 | 外部系統串接設定的約定資料有誤 | Config error, escalate |
| 910071 | 約定業務尚未開啟 | Contact bank |
| 910072 | 查詢條件錯誤，無法查詢 | Fix query |
| 910073 | 超過查詢上限 (500 筆) | Split batch |
| 910074 | 放行憑證資料查詢異常 | Escalate |
| 910075 | 最高安控非 FXML | eToken not FXML, escalate |
| 910097 | 送審經辦不存在 | Fix flowInfo.operatorName |
| 910098 | 送審經辦狀態異常 | Escalate |
| 910099 | 送審流程不存在 | Fix flowInfo.flowName |
| 910144 | 電文異常訊息 | Log full payload, escalate |
| 999999 | 未知錯誤 | Log full payload, escalate |

**Rule**: Only `910000` yields `rsData`; all others are failures.

---

## 5. Transaction Status Codes (Appendix 六.(二))

See `02-business-workflow.md` §5 for the full table and terminal flags. Summary of pending (non-terminal) states: `1000, 1001, 1003, 1004, 1005, 1006, 1007, 1008, 1030`. Terminal: `0000, 1002, 1031, 1032, 3000, 3001, 4000, 9999`.

---

## 6. Audit & Traceability

### 6.1 Ledger

- `IdempotencyLedger` (SQLite) records:
  - File batches: `register_file_batch(batch_id, type, filename, bytes, 0)`.
  - Order locks: `check_and_lock_order(key, batch_id, amount)` with key `VOUCHER_NO:AMOUNT:BATCH_DATE`.
  - Findings: `log_finding` with rule codes `VR-BANKPAY-01` (export), `VR-BANKPAY-02` (import), `VR-BANKPAY-03` (quarantine), `VR-BANKPAY-QUERY` (traceable query).
  - Quarantine: `log_quarantine` for unparseable rows.
- Every API call logged with payload SHA-256 hash (never raw secrets or PII beyond needed fields).

### 6.2 Traceable Query

`query(engine, kind, key)`:
- `voucher` → voucher number → processed flag + findings.
- `tx` → bank reference → cross-reference findings.
- `batch` → batch id → registered batch count + findings.

Each query recorded as `VR-BANKPAY-QUERY` finding, making tracing itself auditable.

### 6.3 Release Gate Audit

All state transitions (`DraftGenerated → VerifiedPass/Quarantined → AuthorizedDispatch → Completed/Failed/DispatchUncertain`) written to `history: Vec<TransitionRecord>`.

---

## 7. Data Protection

- No secrets (eToken, private keys, enable codes) stored in ERP database or logs.
- Certificate files stored under `docs/bankpay/references/` gitignored for secrets/certs.
- Payloads with PII (notifyEmail, payeeId) logged only as hashes where possible; full payload retention governed by retention policy.
- RBAC: finance role gated for payment release; release manager role for eToken approval.

---

## 8. Compliance Mapping

| Control | Implementation |
|---|---|
| Least privilege | RBAC finance/release-manager roles |
| Audit trail | Ledger findings + release gate history |
| Non-repudiation | eToken signature by release manager |
| Data integrity | SHA-256 content hash per batch |
| Idempotency | Order lock prevents duplicate payment |
| Encryption in transit | TLS 1.2+ 2-Way SSL |
