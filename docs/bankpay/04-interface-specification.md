# E.Sun Bank ERP Integration — Interface Specification

**Document**: 04-interface-specification.md
**Version**: 1.0
**Source**: E.Sun Bank 技術文件 V2.8
**Base URL**: `http://<IP>:8080` (dev/test) / `http://<IP>:8443` (prod)
**Content-Type**: `application/json` (UTF-8)
**Timezone**: UTC+8

---

## 1. Common Headers & Envelope

### 1.1 Request Header

| Param | Required | Description |
|---|---|---|
| `Content-Type` | Y | `application/json` |
| `Source-System` | Y | Agreed `SourceSystemId` |

### 1.2 Request Envelope (`rqHeader`)

| Field | Type | Required | Length | Description |
|---|---|---|---|---|
| `msgId` | string | Y | 23–25 | 統編(8–10) + YYYYMMDD + 7 碼流水 |
| `requestTime` | string | Y | 19 | `YYYY/MM/DD HH:MM:SS` |
| `customerIxd` | string | Y | 8–10 | 企業集團統編 |
| `sourceSystem` | string | Y | — | `SourceSystemId` |

### 1.3 Response Envelope (`rsHeader` / `rsStatus`)

| Field | Type | Length | Description |
|---|---|---|---|
| `rsHeader.msgId` | string | 23–25 | Echoed msgId |
| `rsHeader.logId` | string | 40 | Component tracking seq |
| `rsHeader.responseTime` | string | 19 | Reply time |
| `rsStatus.code` | string | 6 | Result code (910000 = success) |
| `rsStatus.message` | string | 60 | Description |
| `rsStatus.moreInfo` | string | — | Per-tx error detail (semicolon-separated) |

> `rsData` is present **only** when `rsStatus.code == 910000`.

---

## 2. API #1 — TWD Payment Release

**Endpoint**: `POST /payment/createTwdPaymentInstruction`

### 2.1 Request `rqData`

| Field | Type | Req | Len | Description |
|---|---|---|---|---|
| `formatType` | string | Y | 1 | `1`=file (base64 media), `2`=json |
| `paymentType` | string | Y | 1 | `1`=one debit one credit, `2`=one debit many credits |
| `verificationType` | string | Y | 1 | `1`=ERP 驗證放行, `2`=網銀驗證放行 |
| `flowInfo.operatorName` | string | N | 32 | Net-banking operator (required if type 2) |
| `flowInfo.flowName` | string | Y* | 64 | Net-banking flow name (required if type 2) |
| `flowInfo.flowKey` | long | N | 18 | Flow key |
| `content` | string | Y | — | Base64 (file) or Base64(JSON) |

### 2.2 TWD Payment Data (`twdTxList`) — fields

| Field | Type | Req | Len | Description |
|---|---|---|---|---|
| `payerAccountNo` | string | Y | 16 | 玉山帳號 ≥13 碼 |
| `payerId` | string | Y | 15 | 歸戶統編 ≥10 碼 |
| `payeeBankId` | string | Y | 3 | 收款銀行代碼 (fixed 3) |
| `payeeBranchId` | string | Y | 4 | 收款銀行分行 |
| `payeeAccountNo` | string | Y | 16 | 收款帳號 |
| `payeeName` | string | Y | 80 | 收款戶名 (全形 ≤39) |
| `payeeId` | string | N | 15 | 收款人統編 |
| `txDate` | string | Y | 8 | YYYYMMDD |
| `txAmt` | string | Y | 13 | > 0 |
| `chargeFeeType` | string | Y | 1 | `0`=外收, `1`=內扣 (default 0) |
| `memo` | string | N | 80 | 附言 (全形 ≤39) |
| `notifyMethod` | string | N | 1 | `1`=不通知, `2`=Email, `3`=Fax, `9`=均發送 (default 1) |
| `notifyEmail` | string | N | 400 | Semicolon-separated emails |
| `notifyFax` | string | N | 20 | ≥10 incl. area code |
| `writeOffNo` | string | N | 40 | 銷帳編號 |
| `remark` | string | N | 38 | 備註 (全形 ≤18) |
| `txSeq` | string | Y | 22 | ERP payment draft number (unique) |
| `detailList[].invoiceDate` | string | N | 8 | YYYYMMDD |
| `detailList[].invoiceNo` | string | N | 20 | 單據號碼 |
| `detailList[].invoiceAmt` | string | N | 18.2 | 應付金額 |
| `detailList[].invoiceReturn` | string | N | 18.2 | 折讓退回 |
| `detailList[].invoiceActualAmt` | string | N | 18.2 | 實付金額 |
| `detailList[].invoiceRemark` | string | N | 40 | 單據備註 |

### 2.3 Response `rsData`

| Field | Type | Len | Description |
|---|---|---|---|
| `dataCount` | string | 5 | Total records |
| `txRecordList[].txSeq` | string | 22 | Echoed |
| `txRecordList[].autoDelay` | string | 1 | `0`=no delay, `1`=delayed |
| `txRecordList[].payerAccountNo` | string | 13 | |
| `txRecordList[].payerId` | string | 15 | |
| `txRecordList[].payeeBankId` | string | 3 | |
| `txRecordList[].payeeBranchId` | string | 4 | |
| `txRecordList[].payeeAccountNo` | string | 16 | |
| `txRecordList[].payeeName` | string | 80 | |
| `txRecordList[].payeeId` | string | 15 | |
| `txRecordList[].txDate` | string | 8 | |
| `txRecordList[].txAmt` | string | 13 | |
| `txRecordList[].chargeFeeType` | string | 1 | |
| `txRecordList[].memo` | string | 80 | |
| `txRecordList[].notifyMethod` | string | 1 | |
| `txRecordList[].notifyEmail` | string | 400 | |
| `txRecordList[].notifyFax` | string | 20 | |
| `txRecordList[].writeOffNo` | string | 40 | |
| `txRecordList[].remark` | string | 38 | |
| `txRecordList[].detailList` | object[] | — | Invoice detail |

---

## 3. API #2 — TWD Transaction Status Query

**Endpoint**: `POST /payment/retrieveTwdPaymentInstructionResult`

### 3.1 Request `rqData`

| Field | Type | Req | Len | Description |
|---|---|---|---|---|
| `searchList` | list | Y | ≤500 | Query list |
| `searchList[].payerIxd` | string | Y | 8–10 | 顧客統編 |
| `searchList[].txSeq` | string | Y | 22 | ERP txSeq |

### 3.2 Response `rsData.txInfo[]`

| Field | Type | Len | Description |
|---|---|---|---|
| `payerIxd` | string | 15 | 付款統編 |
| `txSeq` | string | 22 | |
| `actualTxDate` | string | 8 | YYYYMMDD (success only) |
| `actualTxTime` | string | 8 | HHmmss (success only) |
| `actualChargeFee` | string | 10 | 實際手續費 (success only) |
| `status` | string | 4 | Transaction status code |
| `statement` | string | 60 | Status description |
| `sysErrorDesc` | string | 120 | Error detail (on failure) |

---

## 4. API #3 — Transaction Detail Query (Statement)

**Endpoint**: `POST /account/retrieveStatements`

### 4.1 Request `rqData`

| Field | Type | Req | Len | Description |
|---|---|---|---|---|
| `companyId` | string | Y | 8–10 | 帳號所屬公司統編 |
| `demandDeptAcc` | string | Y | 13 | 玉山帳號 |
| `sTxDate` | string | Y | 8 | 起日 YYYYMMDD |
| `eTxDate` | string | Y | 8 | 迄日 YYYYMMDD |
| `sTxTime` | string | N | 8 | 起時 HHMMSSFF (default 00000000) |
| `eTxTime` | string | N | 8 | 迄時 HHMMSSFF (default 23595999) |
| `curr` | string | N | 3 | 幣別 (blank = all) |
| `dbCkn` | string | N | 7 | 票據號碼 |
| `idNoAssetCode` | string | N | 5 | 企業識別碼 |
| `orderDescType` | string | Y | 1 | `1`=新到舊, `2`=舊到新 |
| `queryStartIndex` | string | Y | 5 | 起始筆序 (first = 1) |
| `firstDeptTxDtlSeqNo` | string | N | 19 | 定位序號 (first call blank) |

### 4.2 Response `rsData.deptTxDtlList[]`

| Field | Type | Len | Description |
|---|---|---|---|
| `deptTxDtlSeqNo` | string | 19 | 交易明細序號 |
| `demandDeptAcc` | string | 13 | 玉山帳號 |
| `curr` | string | 3 | 幣別 |
| `txDate` | string | 8 | 實際交易日期 |
| `bookingDate` | string | 8 | 銀行帳務日期 |
| `exchgRate` | string | 12.6 | 匯率 |
| `txTime` | string | 8 | 實際交易時間 |
| `txAmt` | string | 18.2 | 交易金額 |
| `chkNo` | string | 10 | 票號 |
| `debitCredit` | string | 2 | `DB`=借, `CR`=貸 |
| `blncBeforeTx` | string | 18.2 | 交易前餘額 |
| `blncAfterTx` | string | 18.2 | 交易後餘額 |
| `idNoOfCom` | string | 5 | 企業識別碼 |
| `summary` | string | 14 | 交易摘要 |
| `txDtlsRemark` | string | 40 | 交易明細備註 |
| `txType` | string | 1 | 1=現金,2=轉帳,3=今交票,4=次交票 |
| `rvrsTxMark` | string | 1 | N=一般, Y=沖正 |
| `oppositeBankCode` | string | 3 | 對方銀行代號 |
| `oppositeAccount` | string | 16 | 對方銀行帳號 |

Pagination: continue while returned rows == 500 and code == 910000.

---

## 5. API #4 — FX Pre-Check (Validation)

**Endpoint**: `POST /payment/validateFxPaymentInstruction`

### 5.1 Request `rqData`

| Field | Type | Req | Len | Description |
|---|---|---|---|---|
| `formatType` | string | Y | 1 | `2`=json |
| `paymentType` | string | Y | 1 | `1`=one debit one credit, `2`=one debit many credits |
| `content` | string | Y | — | Base64(JSON of `foreignTxList`) |

### 5.2 Key `foreignTxList` fields (subset)

| Field | Type | Req | Len | Description |
|---|---|---|---|---|
| `txSeq` | string | Y | 22 | ERP txSeq (unique) |
| `txDate` | string | Y | 8 | YYYYMMDD (today or future) |
| `payerId` | string | Y | 8–15 | 付款統編 |
| `payerAccountNo` | string | Y | 12–17 | 付款帳號 |
| `payerCurrency` | string | Y | 3 | 付款幣別 |
| `payerAmount` | decimal | N | 15.2 | 付款金額 (擇一) |
| `payeeAccountNo` | string | Y | ≤34 | 受款帳號 (IBAN for Europe) |
| `payeeCurrency` | string | Y | 3 | 受款幣別 |
| `payeeAmount` | decimal | N | 15.2 | 受款金額 (擇一) |
| `payeeName` | string | Y* | ≤140 | 受款人名稱 (FX cross-border required) |
| `payeeBankSwiftCode` | string | N | 8/11 | SWIFT (or payeeBankName) |
| `payeeBankAddressCountry` | string | Y | 2 | 受款銀行國別 |
| `remittanceType` | string | Y | 1 | `1`=一般, `2`=全額到行, `3`=受益人負擔 |
| `feeType` | string | Y | 1 | `0`=外收, `1`=內含 |
| `feeDebitAccountNo` | string | Y* | 12–17 | 費用付款帳號 (cross-border) |
| `feeDebitCurrency` | string | Y* | 3 | 費用付款幣別 |
| `notificationType` | string | N | 1 | `0/1`=不通知, `2`=Email |
| `notificationInfo` | string | N | 250 | Email(s) |

Full field list (addresses, declaration codes, invoice detailList) per V2.8 §4.(四).

### 5.3 Response

Returns `status`: `3000` (檢核成功) or `3001` (檢核失敗) plus computed `listingRate`, `dealingRate`, `feeDebitAmount`, `transactionReferenceNo`, and `sysErrorDesc` on failure. `moreInfo` carries per-tx errors as `txSeq|reason;...`.

---

## 6. API #5 — FX Payment Release

**Endpoint**: `POST /payment/createFxPaymentInstruction`

Same request/response shape as API #4 (pre-check + release). Additional `verificationType` (1/2) and `flowInfo`. Response includes bank-computed rates/fees and `transactionReferenceNo`.

---

## 7. API #6 — FX Transaction Status Query

**Endpoint**: `POST /payment/retrieveFxPaymentInstructionResult`

### 7.1 Request

| Field | Type | Req | Len |
|---|---|---|---|
| `searchList[].payerId` | string | Y | 8–10 |
| `searchList[].txSeq` | string | Y | 22 |

### 7.2 Response

References FX pre-check response shape; key status fields:

| Field | Type | Len | Description |
|---|---|---|---|
| `status` | string | 4 | Transaction status code |
| `statusDesc` | string | 60 | Description |
| `transactionReferenceNo` | string | 16 | Bank transaction number |
| `listingRate` / `dealingRate` | decimal | 13 | FX rates |
| `feeDebitAmount` | decimal | 13 | Fee |
| `valueDate` | string | 8 | Value date (on send) |
| `actualTxDate` | string | 8 | Actual tx date |
| `sysErrorDesc` | string | 120 | Error detail |

---

## 8. API #7 — Account Balance Query

**Endpoint**: `POST /account/retrieveAccountBalance`

### 8.1 Request `rqData`

| Field | Type | Req | Len | Description |
|---|---|---|---|---|
| `companyId` | string | Y | 8–10 | 帳號所屬公司統編 |
| `currType` | string | Y | 1 | `1`=TWD, `2`=FX |
| `demandDeptAcc` | string | N | 13 | 玉山帳號 (blank = all) |

### 8.2 Response `rsData.deptBlncDtlList[]`

| Field | Type | Len | Description |
|---|---|---|---|
| `demandDeptAcc` | string | 13 | 玉山帳號 |
| `curr` | string | 3 | 幣別 |
| `accType` | string | 1 | 1=活存, 2=支存, 3=綜存 |
| `actBalance` | string | 18.2 | 實質餘額 |
| `avlBalance` | string | 18.2 | 可用餘額 |
| `todayExgAmt` | string | 18.2 | 今交票金額 |
| `nextExgAmt` | string | 18.2 | 次交票金額 |

---

## 9. API #8 — TWD Payment Result Print

**Endpoint**: `POST /payment/retrieveTwdPaymentInstructionPrintResult`

### 9.1 Request `rqData`

| Field | Type | Req | Len | Description |
|---|---|---|---|---|
| `searchList` | list | Y | ≤5 | Query list |
| `searchList[].payerIxd` | string | Y | 8–10 | 顧客統編 |
| `searchList[].txSeq` | string | Y | 22 | ERP txSeq |

### 9.2 Response `rsData.printResultList[]`

| Field | Type | Len | Description |
|---|---|---|---|
| `payerIxd` | string | 15 | 付款統編 |
| `txSeq` | string | 22 | |
| `content` | string | X | PDF binary as Base64 |

---

## 10. Currency Codes

USD, CNY, HKD, JPY, EUR, AUD, CAD, GBP, ZAR, NZD, CHF, SEK, SGD, MXN, THB.

## 11. Status & Result Code Reference

See `05-security-and-audit.md` §4 (result codes) and `02-business-workflow.md` §5 (transaction status).
