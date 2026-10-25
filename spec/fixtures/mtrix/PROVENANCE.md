# MTRIX 測試樣本出處（Fixture Provenance）

> 文件編號：KNB-DAT-001｜狀態：Approved｜版本：v1.0｜日期：2026-10-20
> 對應目錄：`Wellfit/spec/fixtures/mtrix/`

## 1. 分類原則

本目錄中的樣本分為兩類，**永不混用**：

| 類別 | 目錄 | 性質 | 用途 |
|---|---|---|---|
| **交付樣本（delivered）** | `delivered/` | 對接方實際交付之位元組，檔名與內容均未修改 | 介接驗收基準（ADR-0001 R-B） |
| **衍生案例（derived）** | `derived/` | 由交付樣本**逐位元組衍生**，每組僅注入一個缺陷 | 逐條驗證 VR 規則 |

**合成（虛構）資料一律不得進入本目錄。** 專案禁止將自行捏造的資料當作實體資料呈現（絕對反模擬律）。

## 2. 交付樣本（delivered）

- **來源**：`2026\哈瓦士串接專案\API_Integration_Samples\`（對接系統樣本正式儲存位置）
- **取得方式**：`spec\fixtures\mtrix\generate-derived.ps1` 逐檔複製，未做任何位元組修改
- **機敏等級**：內部機敏（含客戶代碼、產品碼、價格、數量）；僅限內部版控，不得推上公開遠端

| 檔案 | 位元組 | SHA-256 | 對應角色 |
|---|---|---|---|
| `ALTHCI30062020153005130 2 (3).csv` | 492 | `56608440DEC2F79A2A2E2A84A6A5CEB40D421B518967B4F13D54D255F05B99E3` | Customer / PoS（16 欄） |
| `ALTHES30062020153005130 2 (3).csv` | 336 | `C4FC9EF8611AA04187EBC16E6480735EBA24306ADD04C09B908F77239F978C8B` | Inventory（6 欄） |
| `ALTHFV30062020153005130 2 (3).csv` | 319 | `457019CCAECF37EBD61020A6DDC7F49CC78865C50A45386359A6839B9AE2A673` | Sales Force（10 欄） |
| `ALTHPD30062020153005130 2 (3).csv` | 524 | `EAF23C2DE466A60F9D1865A7496CB2990176EEE47C91C2C9D6F5BC64AAC14CE4` | Product（12 欄） |
| `ALTHVD30062020153005130 5 (4).csv` | 892 | `CFEC9717E66DBD9F48F3AB29BFC6E48E3EBB27458E1AC54CD82963F46F976842` | Sales / Sellout（12 欄） |

雜湊與來源目錄逐一比對一致，確認為位元組相同之副本。

## 3. 已知的交付資料偏差（不得靜默改寫）

下列差異為「交付實況 vs 原始規格」之落差，引擎一律**記錄偏差並保留原值**，僅在輸出時使用技術別名：

| 檔案 | 序 | 交付欄名 | 原始規格欄名 | 處理 |
|---|---|---|---|---|
| ALTHCI | 5 | `ADRESS` | `ADDRESS` | ANOM-02，輸出用 `ADDRESS` |
| ALTHCI | 7 | `ZIP_CODE` | `ZIP CODE` | ANOM-03，輸出用 `ZIP_CODE` |
| ALTHCI | 12 | `CUSTOMER_ID` | `CUSTOMER_ID` | MTRIX-009，兩欄皆保留（`CUSTOMER_ID` / `CUSTOMER_ID_2`），不合併 |
| ALTHFV | 3 | `CUSTOMER_ID` | `CUSTOMER ID` | ANOM-03 |
| ALTHFV | 4 | `MANAGER_CODE` | `MANAGER_ID` | ANOM-03 |
| ALTHFV | 8 | `SALESPERSON_CODE` | `SALESPERSON_ID` | ANOM-03 |
| ALTHFV | 10 | `COUNTRY` | `COUNTRY_ISO` | ANOM-03 |
| ALTHVD | 11 | `ZIP_CODE` | `ZIP CODE` | ANOM-03 |
| ALTHVD | 12 | `COUNTRY_ISO` | `COUNTRY ISO` | ANOM-03 |

其他已觀察到的實況差異（皆由既有修復規則處理，詳見 `Knowledge/specs/mtrix-anomaly-catalogue.md`）：
- ALTHES 之 `VENDOR_ID` 帶 3 字母國碼前綴（原始規格為 2 字母）→ ANOM-05，前綴與廠商碼分離保存。
- ALTHPD 之 `DIST_ID` 含點遮罩 → ANOM-06 還原為無遮罩 RUC。
- ALTHPD 之 `BARCODE` 帶尾端空白 → ANOM-09/ANOM-07 清理。
- ALTHCI 第 5 列 `STORE_TYPE` 為 `Onlinne` → ANOM-04 正規化為 `Online`。
- ALTHVD 全部列之 `DOCUMENT_TYPE` 與 `SALESPERSON_CODE` 為空 → 必填性未定義（MTRIX-003），記 Warning 不隔離。

## 4. 衍生案例（derived）

由 `generate-derived.ps1` 從交付樣本生成，每組為完整五檔，**僅一個檔案帶一個缺陷**：

| 案例 | 注入缺陷 | 預期規則 | 預期結果 |
|---|---|---|---|
| `case-a-value-violations` | 庫存數量 -12；包裝型別 7；日期 20240230（不存在的日）；文件類型 X | VR-06 / VR-07 / VR-03 / VR-04 | 4 列隔離（值違規依原文為「隔離錯誤列」） |
| `case-b-duplicate-transaction` | 同一交易同一商品重複一列，數量拆為兩筆 | VR-05 | 彙總為單筆 + Warning |
| `case-c-header-rejected` | 銷售檔標題 `BATCH_CODE` 改名為 `ITEM_CODE` | VR-02 | 該檔拒絕，其餘四檔照常處理，狀態 `partial-accepted` |
| `case-d-salesforce-orphan-customer` | 業務檔客戶代碼 `999UNKNOWN` 不存在於客戶主檔 | VR-14 | 列出該鍵（跨檔規則為「列出」而非隔離） |

重新產生方式（由交付樣本重建，確保衍生關係可重現）：

```powershell
powershell -ExecutionPolicy Bypass -File spec\fixtures\mtrix\generate-derived.ps1
```

## 5. 移除的舊樣本

原 `data\raw_samples\*`、`data\ALTH*.csv` 與 `data\repaired_outputs\*` 為早期自創版面之合成資料，違反絕對反模擬律，已移除。`audit` 子命令不再自動產生任何樣本：缺檔即明確回報 `not-evaluated`。
