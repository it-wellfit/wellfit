# E.Sun Bank API V2.8 - Complete Technical Specification

## Overview

This document consolidates the E.Sun Bank enterprise ERP integration technical specification (V2.8) for the Wellfit project.

**Version**: V2.8
**Source**: `【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.pdf`
**Status**: Secondary transcription of the local bank specification; not implementation or acceptance evidence
**Last Updated**: 2026-10-28

> **Authority note (2026-11-02):** This file duplicates [`docs/bankpay/04-interface-specification.md`](../bankpay/04-interface-specification.md). Verify exact endpoints, ports, and field semantics against the cited bank PDF before use; this summary is not the bank's source document and does not prove that the endpoints are implemented.

---

## 1. System Architecture

### 1.1 Components

| Component | Description | Port |
|-----------|-------------|------|
| **ERP Server** | Wellfit ERP system | Internal |
| **E.Sun Security Component** | 銀企直連安控元件 | 8443 (internal), 9443 (external) |
| **E.Sun Bank Host** | Bank processing system | 9443 |
| **eToken** | Enterprise certificate for transaction signing | - |

### 1.2 Communication Flow

```
ERP Server (Port 8443) → E.Sun Security Component → E.Sun Bank Host (Port 9443)
```

---

## 2. API Access Rules

### 2.1 TLS Requirements
- **Protocol**: TLS 1.2 or higher
- **Authentication**: 2-Way SSL (mutual TLS)
- **Encoding**: UTF-8 JSON format
- **Timezone**: UTC+8

### 2.2 HTTP Headers

| Header | Required | Description |
|--------|----------|-------------|
| `Content-Type` | Y | `application/json` |
| `Source-System` | Y | Agreed SourceSystemId |

### 2.3 URL Endpoints

| Environment | URL |
|-------------|-----|
| DEV Test | `http://IP:8080` |
| Test | `http://IP:8080` |
| Production | `http://IP:8443` |

---

## 3. API Specifications

### 3.1 TWD Payment Release API

**Endpoint**: `POST /payment/createTwdPaymentInstruction`
**Purpose**: Submit TWD payment instructions to E.Sun Bank

#### Request Parameters

| Field | Type | Required | Length | Description |
|-------|------|----------|--------|-------------|
| rqHeader:msgId | String | Y | 23-25 | Message ID: 統編(8-10) + YYYYMMDD + 7-digit sequence |
| rqHeader:requestTime | String | Y | 19 | Request time: YYYY/MM/DD HH:MM:SS |
| rqHeader:customerIxd | String | Y | 8-10 | Enterprise group tax ID |
| rqHeader:sourceSystem | String | Y | - | SourceSystemId |
| rqData:formatType | String | Y | 1 | 1=file (base64 media), 2=json |
| rqData:paymentType | String | Y | 1 | 1=one debit one credit, 2=one debit many credits |
| rqData:verificationType | String | Y | 1 | 1=ERP verification, 2=bank verification |
| rqData:flowInfo:operatorName | String | N | 32 | Net-banking operator name |
| rqData:flowInfo:flowName | String | N | 64 | Flow name (required if type 2) |
| rqData:flowInfo:flowKey | Long | N | 18 | Flow key |
| rqData:content | String | N | - | Base64 encoded content |

#### TWD Payment Transaction (twdTxList)

| Field | Type | Required | Length | Description |
|-------|------|----------|--------|-------------|
| payerAccountNo | String | Y | 16 | E.Sun account (≥13 digits) |
| payerId | String | Y | 15 | Payer tax ID (≥10 digits) |
| payeeBankId | String | Y | 3 | Payee bank code |
| payeeBranchId | String | Y | 4 | Payee bank branch |
| payeeAccountNo | String | Y | 16 | Payee account |
| payeeName | String | Y | 80 | Payee name (max 39 full-width chars) |
| payeeId | String | N | 15 | Payee tax ID |
| txDate | String | Y | 8 | Payment date (YYYYMMDD) |
| txAmt | String | Y | 13 | Payment amount (>0) |
| chargeFeeType | String | Y | 1 | 0=external (payer), 1=internal (payee) |
| memo | String | N | 80 | Memo (max 39 full-width chars) |
| notifyMethod | String | N | 1 | 1=none, 2=email, 3=fax, 9=all |
| notifyEmail | String | N | 400 | Email (semicolon-separated) |
| notifyFax | String | N | 20 | Fax (min 10 including area code) |
| writeOffNo | String | N | 40 | Write-off number |
| remark | String | N | 38 | Remark (max 18 full-width chars) |
| txSeq | String | Y | 22 | ERP transaction sequence (unique) |
| detailList | Object[] | N | - | Invoice details |

#### Response Parameters

| Field | Type | Length | Description |
|-------|------|--------|-------------|
| rsHeader:msgId | String | 23-25 | Echoed msgId |
| rsHeader:logId | String | 40 | Component tracking sequence |
| rsHeader:responseTime | String | 19 | Reply time |
| rsStatus:code | String | 6 | Result code (910000=success) |
| rsStatus:message | String | 60 | Description |
| rsStatus:moreInfo | String | - | Per-transaction error detail |
| rsData:dataCount | String | 5 | Transaction count |
| rsData:txRecordList | Object[] | - | Transaction records |

---

### 3.2 TWD Transaction Status Query API

**Endpoint**: `POST /payment/retrieveTwdPaymentInstructionResult`
**Purpose**: Query status of TWD payment transactions

#### Request Parameters

| Field | Type | Required | Length | Description |
|-------|------|----------|--------|-------------|
| rqHeader:msgId | String | Y | 23-25 | Message ID |
| rqHeader:requestTime | String | Y | 19 | Request time |
| rqHeader:customerIxd | String | Y | 8-10 | Enterprise group tax ID |
| rqHeader:sourceSystem | String | Y | - | SourceSystemId |
| rqData:searchList | List | Y | - | Query list (max 500) |
| rqData:searchList.payerIxd | String | Y | 8-10 | Customer tax ID |
| rqData:searchList.txSeq | String | Y | 22 | Transaction sequence |

#### Response Parameters

| Field | Type | Length | Description |
|-------|------|--------|-------------|
| rsData:txInfo | List | - | Transaction info list |
| rsData:txInfo.payerIxd | String | 15 | Payer tax ID |
| rsData:txInfo.txSeq | String | 22 | Transaction sequence |
| rsData:txInfo.actualTxDate | String | 8 | Actual transaction date |
| rsData:txInfo.actualTxTime | String | 8 | Actual transaction time |
| rsData:txInfo.actualChargeFee | String | 10 | Actual charge fee |
| rsData:txInfo.status | String | 4 | Status code (0000=success) |
| rsData:txInfo.statement | String | 60 | Status description |
| rsData:txInfo.sysErrorDesc | String | 120 | System error description |

#### Transaction Status Codes

| Code | Description |
|------|-------------|
| `0000` | Transaction succeeded |
| `1000-1008` | Pending/processing |
| `1030-1032` | FX related |
| `3000-3001` | FX pre-check |
| `4000` | Transaction failed |
| `9999` | No data |

---

### 3.3 Transaction Statement Query API

**Endpoint**: `POST /account/retrieveStatements`
**Purpose**: Query account transaction statements

#### Request Parameters

| Field | Type | Required | Length | Description |
|-------|------|----------|--------|-------------|
| rqHeader:msgId | String | Y | 23-25 | Message ID |
| rqHeader:requestTime | String | Y | 19 | Request time |
| rqHeader:customerIxd | String | Y | 8-10 | Enterprise group tax ID |
| rqHeader:sourceSystem | String | Y | - | SourceSystemId |
| rqData:companyId | String | Y | 8-10 | Company tax ID |
| rqData:demandDeptAcc | String | Y | 13 | E.Sun account to query |
| rqData:sTxDate | String | Y | 8 | Start date (YYYYMMDD) |
| rqData:eTxDate | String | Y | 8 | End date (YYYYMMDD) |
| rqData:sTxTime | String | N | 8 | Start time (HHMMSSFF) |
| rqData:eTxTime | String | N | 8 | End time (HHMMSSFF) |
| rqData:curr | String | N | 3 | Currency code |
| rqData:dbCkn | String | N | 7 | Check number |
| rqData:idNoAssetCode | String | N | 5 | Asset code |
| rqData:orderDescType | String | Y | 1 | 1=new to old, 2=old to new |
| rqData:queryStartIndex | String | Y | 5 | Start index (1 for first query) |

---

### 3.4 Foreign Currency Transaction Verification API

**Endpoint**: `POST /fx/verifyFxTransaction`
**Purpose**: Pre-verify foreign currency transactions

### 3.5 Foreign Currency Payment Release API

**Endpoint**: `POST /payment/createFxPaymentInstruction`
**Purpose**: Submit foreign currency payment instructions

### 3.6 Foreign Currency Transaction Status Query API

**Endpoint**: `POST /payment/retrieveFxPaymentInstructionResult`
**Purpose**: Query status of foreign currency transactions

### 3.7 Account Balance Query API

**Endpoint**: `POST /account/retrieveBalance`
**Purpose**: Query account balances

### 3.8 TWD Payment Result Printing API

**Endpoint**: `POST /payment/retrieveTwdPaymentResult`
**Purpose**: Get printable payment results

---

## 4. Currency Codes

Supported currency codes (from E.Sun appendix):
- USD, CNY, HKD, JPY, EUR, AUD, CAD, GBP, ZAR, NZD, CHF, SEK, SGD, MXN, THB

---

## 5. API Result Codes

| Code | Description |
|------|-------------|
| `910000` | Success - rsData is present |
| `910071` | Field validation error - moreInfo contains per-tx error detail |
| `999999` | Unknown error |

---

## 6. Implementation Status

### 6.1 Implemented in Wellfit

| API | Endpoint | Implementation Status |
|-----|----------|----------------------|
| TWD Payment Release | `/payment/createTwdPaymentInstruction` | ✅ Implemented |
| TWD Status Query | `/payment/retrieveTwdPaymentInstructionResult` | ✅ Implemented |
| Statement Query | `/account/retrieveStatements` | ✅ Implemented |
| FX Transaction Verification | `/fx/verifyFxTransaction` | ✅ Implemented |
| FX Payment Release | `/payment/createFxPaymentInstruction` | ✅ Implemented |
| FX Status Query | `/payment/retrieveFxPaymentInstructionResult` | ✅ Implemented |
| Account Balance Query | `/account/retrieveBalance` | ✅ Implemented |
| TWD Payment Result | `/payment/retrieveTwdPaymentResult` | ✅ Implemented |

### 6.2 Wellfit Crate Structure

```
crates/wellfit_bankpay/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Module declarations and re-exports
│   ├── api.rs          # E.Sun V2.8 API models
│   ├── bankpay.rs      # BankPay engine
│   ├── encoding.rs     # CP950/Big5/UTF-8 conversion
│   ├── http.rs         # HTTP client with TLS
│   ├── media.rs        # Media file assembly
│   ├── query.rs        # Traceable query
│   └── release.rs      # Release gate
└── tests/
    └── integration_tests.rs
```

---

## 7. References

- **Original Spec**: `【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.pdf`
- **Implementation Plan**: `plans/1793034142477-esun-flaps-ap-voucher-integration.md`
- **Knowledge Base**: `../Knowledge/`
- **Wellfit AGENTS.md**: `AGENTS.md`

---

## 8. Network & Environment Configuration (V2.8 Spec)

### 8.1 Environments

| Environment | Domain | Port | Description |
|-------------|--------|------|-------------|
| DEV | eospt.testesunbank.com.tw | 9443 | Development / testing |
| UAT | eospu.testesunbank.com.tw | 9443 | User acceptance testing |
| Production | eosp.esunbank.com.tw | 9443 | Production |

### 8.2 Ports

| Port | Description |
|------|-------------|
| 8443 | Internal security component (ERP -> 8443 -> approver computer) |
| 9443 | External (outbound to E.Sun environments) |

### 8.3 Prerequisites Checklist (from spec)

- [ ] Enterprise external IP whitelisted
- [ ] Enterprise identity and account agreement with bank
- [ ] eToken for transaction approval
- [ ] Device info (hostname, MacAddress) provided to bank
- [ ] Internal firewall opened (Port 8443)
- [ ] External firewall opened (Port 9443)
- [ ] Security component installed on approver computer and ERP server
- [ ] Security component bound with enterprise tax ID, ClientId, activation code
- [ ] Bank-provided certificate installed on ERP
- [ ] ERP-provided certificate installed on bank

### 8.4 Certificate Exchange

| Direction | Description |
|-----------|-------------|
| Bank -> ERP | Bank provides certificate for ERP message source verification |
| ERP -> Bank | ERP provides certificate for Bank message source verification |

### 8.5 TLS Requirements

- Protocol: TLS 1.2 or higher
- Authentication: 2-Way SSL (mutual TLS)
- Encoding: UTF-8 JSON format
- Timezone: UTC+8
- SourceSystemId: Required in all API requests
