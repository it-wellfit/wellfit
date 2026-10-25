# Wellfit DEMO 示場流程

> 文件編號：WFLT-DOC-DEMO-001｜狀態：Draft｜版本：v1.0｜日期：2026-10-20
> 撰寫：Wellfit 知識庫維護者｜審閱：未審閱｜核准：未核准
> 變更摘要：v1.0。全部輸出改為真實實測值；明確標示 MTRIX 為真實交付位元組、蝦皮與輔翼 ERP 為合成示範資料。
> 驗證標記：✔ 已對原始文件逐字查證｜△ 依標準條文或通行做法推導｜未驗證 本次無法確認

> 知識庫層的完整版（含更多問答）見 [`../../Knowledge/ops/DEMO.md`](../../../Knowledge/ops/DEMO.md)。

## 0. 示場前必讀

| # | 事實 | 後果 |
|---:|---|---|
| 1 | MTRIX 使用**真實交付位元組**（`spec/fixtures/mtrix/delivered/`，SHA-256 與對接方樣本相同） | MTRIX 的數字可被對方覆核 |
| 2 | 蝦皮與輔翼 ERP 使用**合成示範資料**（`data/shopee_orders.csv`、`data/fuyi_inventory_report.csv`） | **不得**宣稱已對接真實蝦皮／ERP 資料 |
| 3 | 對方尚未提供：蝦皮真實匯出欄位（OI-003）、ERP 匯入範本（OI-006）、MTRIX 逐欄必填清單（MTRIX-003） | 被問到時據實回答「尚未取得」 |

## 1. 環境準備

```powershell
cd "C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit"

powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "build -p wellfit_cli" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target-demo"

$wf = "$env:LOCALAPPDATA\Temp\wellfit-target-demo\debug\wellfit.exe"
```

## 2. 示場流程

### 2.1 契約面（2 分鐘）

```powershell
& $wf spec --format markdown
```

**要講的話**：

> 這份契約不是照抄規格書。每一欄都帶著來源層級、驗證狀態、原文頁碼與對應議題編號。
> 五個檔、五十六欄，一欄不多一欄不少。

**追問應對**：任選一欄 → 對照知識庫 `reconciliation/field-diff.md` 的
「L1 原文／L2 推薦／L3 V6.0／交付觀察值／裁決／狀態／議題」。

### 2.2 真實交付批次（4 分鐘）

```powershell
& $wf mtrix --dir spec/fixtures/mtrix/delivered `
            --output-dir "$env:TEMP\demo-repaired" `
            --report "$env:TEMP\demo-report.json" `
            --batch DEMO-20261020 `
            --db "$env:TEMP\demo-ledger.sqlite"
```

```
wave 3 mtrix: status=partial-accepted files=5 errors=22 warnings=24 repairs=38
  - ALTHCI rows 4/4 accepted, 0 isolated, 5 repairs
  - ALTHES rows 4/4 accepted, 0 isolated, 5 repairs
  - ALTHFV rows 4/4 accepted, 0 isolated, 12 repairs
  - ALTHPD rows 4/4 accepted, 0 isolated, 5 repairs
  - ALTHVD rows 9/9 accepted, 0 isolated, 11 repairs
  - VR-01 warning ALTHCI line 0 …: file name carries trailing residue ' 2 (3).csv' after the timestamp
  - VR-08 error ALTHVD… line 2 7AC: customer '7AC' does not exist in the customer master (ALTHCI)
  - VR-09 error ALTHVD… line 2 7893249911649: item '7893249911649' is not present in both the inventory file (false) and the product master (false)
  - VR-10 warning ALTHVD… line 2 : SALESPERSON_CODE is empty; mandatory-ness is undefined by the manual (MTRIX-003)
  - VR-04 warning ALTHVD… line 2 : DOCUMENT_TYPE is empty; mandatory-ness is undefined by the manual (MTRIX-003)
  specification     : 5 files, 56 fields, 14 rules (12 from the field dictionary, 2 from the original manual)
```

**要講的話**：

> 這五個檔是對接方的真實交付位元組，SHA-256 與來源目錄逐一相同。
> 結果是 `partial-accepted` 而**不是** `accepted`——因為有 22 筆 error。
> 重點是：**系統不會為了讓批次看起來漂亮而把問題吞掉**。
> 客戶 `7AC` 不在客戶主檔裡，這是對接方要回答的問題，不是我們要猜的問題。

**追問應對**：

- 為什麼 `errors=22` 而隔離列數是 0？
  → 跨檔規則的結果語意是**「列出鍵值、保留列」**（原文 p.14 要求列出的不是丟棄），
  但嚴重度仍是 error，所以閘門關閉、狀態是 `partial-accepted`。
- 為什麼 `repairs=38`？
  → 每筆修復都有編號（ANOM-01..12）與授權來源，明細在帳本裡，可逐筆還原。

### 2.3 標題偏差與修復輸出（3 分鐘）

```powershell
Get-Content "$env:TEMP\demo-repaired\ALTHCI30062020153005130 2 (3)_repaired.csv" -TotalCount 1
Get-Content "$env:TEMP\demo-repaired\ALTHFV30062020153005130 2 (3)_repaired.csv" -TotalCount 1

$report = Get-Content "$env:TEMP\demo-report.json" -Raw | ConvertFrom-Json
$report.deviations | Format-Table prefix, seq, source_name, spec_name, canonical_name, issue_id, manual_page
```

**要講的話**：

> 交付方寫的是 `ADRESS`、`ZIP_CODE`、`MANAGER_CODE`；原始手冊寫的是 `ADDRESS`、`ZIP CODE`、`MANAGER_ID`。
> 我們**不改對方的檔案**，而是把 8 處偏差記在報告裡（每處帶議題編號與原文頁碼），
> 輸出給下游 BI 的檔案才使用正規化名。語意以原文為準、介接以實際位元組為準，兩者不一致時留下紀錄。

### 2.4 負向案例（3 分鐘）

```powershell
# 標題不符：只有那一個檔被拒，其餘四檔照常
& $wf mtrix --dir spec/fixtures/mtrix/derived/case-c-header-rejected `
            --report "$env:TEMP\demo-case-c.json" --batch DEMO-C --db "$env:TEMP\demo-ledger.sqlite"

# 業務檔孤島客戶：VR-14 列出該鍵
& $wf mtrix --dir spec/fixtures/mtrix/derived/case-d-salesforce-orphan-customer `
            --report "$env:TEMP\demo-case-d.json" --batch DEMO-D --db "$env:TEMP\demo-ledger.sqlite"
```

**要講的話**：

> 這四組負向案例由真實交付位元組**逐位元組衍生**，每組只注入一個缺陷，產生腳本是決定性的、可重跑。
> 目的是讓每一條規則都能被單獨驗證，而不必等真實資料裡剛好出現這些錯誤。

### 2.5 帳本（2 分鐘）

```powershell
& $wf status --db "$env:TEMP\demo-ledger.sqlite"
```

```
Wellfit system status
  ledger            : <帳本路徑>
  isolated records  : 0
  validation errors : 22
  validation warnings: 24
```

**要講的話**：

> 每筆隔離都保留完整原值；每筆 finding 都有規則編號、嚴重度、行號、鍵值與原因。
> 報告裡的每一筆 finding 都同時寫進帳本，所以 `status` 的數字與波次輸出的
> `errors`／`warnings` **必然一致**——不會有「報告看得到、帳本查不到」的情形。
> 正式擷取以批次 ID 加 SHA-256 去重，同一批資料重送不會被重複處理；
> `audit` 是**驗證**而非擷取，因此以「每次執行代次」認領，同一份檔案可以重複稽核。

### 2.6 蝦皮與 ERP（合成資料，必須說明）（1 分鐘）

```powershell
& $wf shopee --input data/shopee_orders.csv --output "$env:TEMP\erp.csv" --db "$env:TEMP\demo-ledger.sqlite"
& $wf reports --input data/fuyi_inventory_report.csv --recipient finance@dachen.com.tw --db "$env:TEMP\demo-ledger.sqlite"
```

```
wave 1 shopee: 2 order(s) accepted, 0 record(s) isolated
wave 2 reports: 4 record(s) accepted, 0 record(s) isolated
```

**要講的話**：

> 這兩段的輸入是我們自己的**合成示範資料**，不是蝦皮或輔翼 ERP 的真實匯出檔——
> 因為對方還沒提供真實樣本（OI-003 與 OI-006）。
> 我們做到的是：8 個必填欄位以名稱解析、缺欄即整批拒收、金額以整數分檢查 EX-06 守恆律、
> 違規列隔離並保留原值。真實欄位對應要等對方提供樣本才能簽核。

## 3. 常見問答

| 問題 | 答案 | 證據 |
|---|---|---|
| 「56 欄」有沒有漏？ | 沒有。16／6／10／12／12 = 56，程式碼與產出文件三處一致 | `spec.rs::test_total_field_count_is_56` |
| 規則幾條？ | 15 條：欄位字典 13 條 + 原始規格衍生 2 條（VR-13 水位對齊、VR-14 業務客戶存在性） | `validators.rs::VrRule::all()` |
| 「27 項格式異常」是什麼？ | 舊說法，**沒有來源**，已撤回。實際是 12 條編號化規則 ANOM-01..12 | 知識庫 `specs/mtrix-anomaly-catalogue.md` |
| 蝦皮 58 欄在哪？ | 58 是內部需求文件的**候選 Mapping 列數**（47 欄待確認、11 欄推薦），不是實際欄位；實作契約為 8 欄 | 知識庫 `specs/shopee-contract.md` |
| 資料是真的嗎？ | MTRIX 是真實交付位元組；蝦皮與 ERP 是合成示範資料 | 知識庫 `data/provenance.md` |
| 跑過測試嗎？ | 121 個測試全數通過（2026-10-20 06:33 UTC 實測） | 知識庫 `uat/UAT-REPORT.md` |
| 對方還沒回答什麼？ | 15 題，已整理成可直接寄送的清單 | 知識庫 `issues/mtx-issue-register.md` §5 |
| 為什麼整批是 partial-accepted？ | 跨檔參照有 error。跨檔問題只「列出鍵、保留列」是原文要求，但嚴重度仍是 error | ADR-0002 §3 |
| 這個專案跑了多久？ | **未確認**——本知識庫未記載專案起訖日期；若需此數字，請由專案窗口提供 | — |

## 4. 示場後待辦

1. 對方承諾提供的資料（尤其蝦皮真實匯出樣本、ERP 匯入範本、MTRIX 必填清單）登記到知識庫議題登錄，
   指派負責人與期限。
2. 若現場確認了新的裁決，更新 `Knowledge/reconciliation/layer-evidence.json` 後重跑 `build-field-diff.ps1`。
3. 若現場使用了未登錄的資料檔，補登 SHA-256 後才可納入 fixture。
