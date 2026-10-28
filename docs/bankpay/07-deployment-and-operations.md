# E.Sun Bank ERP Integration — Deployment and Operations

**Document**: 07-deployment-and-operations.md
**Version**: 1.0
**Source**: E.Sun Bank 技術文件 V2.8
**Status**: Draft

---

## 1. Prerequisites Checklist

- [ ] Outbound IP whitelisted by bank firewall.
- [ ] Corporate net-banking identity approved; accounts & limits agreed.
- [ ] eToken (FXML) issued and tested.
- [ ] Release-manager workstation + ERP/API server hostnames & MAC addresses submitted to bank.
- [ ] Ports opened: 8443 (internal, ERP server → release-manager workstation), 9443 (external, both → bank).
- [ ] Security component installed as administrator on both machines.
- [ ] Binding completed: 統編 + ClientId + 啟用碼.
- [ ] Bank certificate + ERP certificate exchanged.
- [ ] SourceSystemId received from bank (email).
- [ ] ClientId and 啟用碼 received (email + password letter).

---

## 2. Security Component Installation

1. Run installer as system administrator on the release-manager workstation and the ERP/API server.
2. On first launch, enter binding info: 企業集團統編, ClientId, 啟用碼.
3. Verify service starts and listens on port 8443.
4. Install bank certificate into ERP trust store; install ERP certificate into bank trust store.
5. Verify `/health` (or component health) returns version V2.8.

---

## 3. ERP Configuration

```toml
# wellfit config (example)
[bankpay]
base_url_internal = "https://localhost:8443"
base_url_external = "https://eosp.esunbank.com.tw:9443"  # or test/UAT
source_system_id = "<from bank>"
client_cert_path = "certs/erp-client.p12"
client_cert_password = "<secret>"
bank_ca_cert_path = "certs/esun-ca.pem"
timeout_ms = 30000
max_retries = 3
```

Secrets (`client_cert_password`, 啟用碼) must come from secret store, never committed.

---

## 4. Network Topology

```
ERP (API) Server ──8443──► Release-Manager Workstation ──9443──► Bank (eosp*.esunbank.com.tw)
        │                          │
        └─────── (both must reach bank 9443) ──────────────────────┘
```

- ERP server → release-manager workstation: port 8443 (internal firewall).
- Both devices → bank: port 9443 (external firewall).
- Domains: dev `eospt.testesunbank.com.tw`, UAT `eospu.testesunbank.com.tw`, prod `eosp.esunbank.com.tw`.

---

## 5. Operational Runbook

### 5.1 Health Check

- On startup, verify security component `/health` reports version V2.8; fail fast if mismatch.
- Periodic health probe every 60 s; alert on failure.

### 5.2 Certificate Expiry

- Monitor certificate expiry; alert 30 days before expiry.
- Renewal procedure: contact bank, exchange new cert files, restart component.

### 5.3 Retry & Backoff

- Transport errors: exponential backoff (1s, 2s, 4s), max 3 retries.
- `910068` (bank write failure): retry with backoff.
- `2001`-style bank busy: retry with backoff (not in V2.8 code list but general).
- Circuit breaker: open after 5 consecutive failures; half-open after 60 s.

### 5.4 DispatchUncertain Reconciliation

- On timeout, state becomes `DispatchUncertain`.
- Reconcile by re-querying bank status (`/payment/retrieveTwdPaymentInstructionResult`).
- If bank confirms → `Completed`; else → `Failed`, alert finance.

### 5.5 Partial Batch Failure (status 06 / per-tx errors)

- Parse `rsStatus.moreInfo` (semicolon-separated `txSeq|reason`).
- Update per-voucher status; quarantine failed vouchers.
- Re-run only failed items with new txSeq.

---

## 6. Backup & Recovery

- Ledger SQLite database backed up daily (`scripts/bankpay/Backup-Bankpay.ps1`).
- Media files retained per retention policy; SHA-256 hash in ledger for integrity.
- Restore drill: clone backup bundle, run `cargo build`, verify ledger queries.

---

## 7. Monitoring & Alerts

| Metric | Alert Threshold |
|---|---|
| API success rate | < 95% over 15 min |
| P95 status query latency | > 5 s |
| Consecutive transport failures | ≥ 5 |
| Certificate expiry | < 30 days |
| Quarantined rows ratio | > 2% of batch |

---

## 8. Rollback

- Disable payment release via feature flag; fall back to manual bank portal.
- Keep ledger read-only for reconciliation.
- Document rollback in release notes; notify finance.

---

## 9. Environment Promotion

dev (8080) → UAT (eospu, 9443) → prod (eosp, 9443). Each step requires: test report, security review, finance sign-off.
