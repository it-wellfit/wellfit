# 影子擷取規格（S1-1）

**文件編號：** SHADOW-CAPTURE-SPEC-001  
**版本：** 1.0  
**日期：** 2026-11-04  
**狀態：** 生效（實作與測試已到位；僅待真實 FlapsErp 匯出檔執行）  
**依據：** `docs/operations/PRODUCTION_AUTHORIZATION_DECISION.md` Stage 1 前置條件 S1-1~S1-4  
**實作：** `crates/wellfit_assist/src/capture.rs`、CLI `wellfit shadow`  
**證據格式：** `wellfit-shadow-evidence/v1`

---

## 1. 目的與範圍

Stage 1（內部影子試行）的前提，是把「原廠輸出 → 我方獨立驗證 → 差異證據」的流程固定成可重跑、可稽核、零寫入的機制。本規格定義該機制的：

- **S1-1**：擷取程序（對象、步驟、比較方法論、證據格式）
- **S1-2**：唯讀保證（不得觸碰來源、上限、無網路）
- **S1-3**：產出物存放與去識別規則
- **S1-4**：回退方式

**範圍外**：任何對 FlapsErp 的寫入、任何已達 M1 門檻以上的自動化（那是後續階段的授權範圍）。

## 2. 擷取對象（第一版）

| 對象 | 管線 | 比較方法論 | 狀態 |
|---|---|---|---|
| Fuyi ERP 庫存報表匯出（CSV） | `reports` | **L1 檔內獨立重算**：對每列重算 `End = Begin + In - Out + Adj`（checked 運算、溢位即標記），與供應商自報值逐列比對 | ✔ 已實作（`wellfit shadow --pipeline reports`） |
| Shopee 訂單匯出（CSV） | `shopee` | L1 檔內檢核：每訂單 `sum(LINE_AMOUNT) == ORDER_TOTAL`（EX-06） | 規劃中（規格未定，見 §8） |
| MTRIX 五檔 drop | `mtrix` | L1 檔內：既有 15 條 VR 規則重跑 | 規劃中 |
| ERP 匯入結果（原廠端） | `reports`/`shopee` | **L2 輸出比對**：原廠處理結果 vs 我方產出 | 需 FlapsErp 環境，規格另行訂定 |

**比較層級定義**：
- **L1（檔內獨立重算）**：不依賴原廠環境；對原廠自報數字做獨立算術驗證。任何 L1 差異＝原廠輸出內部不一致或我方解析歧義，必須人工裁決。
- **L2（輸出比對）**：同一輸入分別經原廠與我方處理，比對結果。需原廠環境。
- **L3（結果比對）**：匯入後 ERP 狀態 vs 我方帳本。需完整整合。

## 3. 程序步驟（機械化，CLI 實作）

```
wellfit shadow --pipeline reports --input <vendor-export.csv> --artifacts <dir> --batch <id>
```

| # | 步驟 | 實作行為 |
|---|---|---|
| 1 | 讀取（唯讀） | `wellfit_core::text::read_capped`，上限 100 MiB；UTF-8（允許 UTF-8 BOM） |
| 2 | 契約解析（名稱優先） | `fuyi_inventory_report_contract`：8 欄全部必要；缺任一欄→**零列讀取**、錯誤回報；禁止序位退讓 |
| 3 | 逐列獨立重算 | checked add/sub 鏈；`Some(v)`／`None`（溢位）；無法解析的儲存格→`unparsable` |
| 4 | 雙側正規化與摘要 | 供應商值序列與我方重算序列各自 SHA-256（`vendor_digest`／`our_digest`） |
| 5 | 判定 | `row_level_match = (digests 相等) && (mismatches 為空)` |
| 6 | 證據寫入 | `<artifacts>/shadow-mode/<unix>-<batch>.json`（schema 見 §5） |

**退出碼**：0＝全部一致；2＝有差異列（差異本身是**證據**，不是失敗）；1＝錯誤。

## 4. S1-2 唯讀保證

| 保證 | 落地機制 |
|---|---|
| 不寫來源 | 實作僅以 `read_capped` 讀取輸入檔；無任何對輸入路徑的寫入呼叫 |
| 不寫他處 | 唯一寫入＝`<artifacts>/shadow-mode/` 下證據 JSON（`create_dir_all` + `write`） |
| 輸入上限 | 100 MiB 單檔（與工作區既有擷取上限一致；沿用既有測試覆蓋） |
| 無網路 | 無任何 HTTP/TCP 呼叫；純本機檔案處理 |
| 無背景狀態 | 命令為一次性執行；不註冊服務、不排程、無殘留鎖 |

**執行前置（人工）**：Stage 1 授權書面紀錄、產出目錄選定（建議 `artifacts/shadow`，已被 `.gitignore` 之產物規則覆蓋——待確認入庫政策）。

## 5. 證據格式（`wellfit-shadow-evidence/v1`）

```json
{
  "schema": "wellfit-shadow-evidence/v1",
  "captured_at_unix": 1793787393,
  "pipeline": "reports",
  "batch_id": "DEMO-SYNTHETIC-001",
  "input_path": "data/fuyi_inventory_report.csv",
  "input_sha256": "604390ef...",
  "input_bytes": 252,
  "records_total": 4,
  "records_verified": 4,
  "records_mismatched": 0,
  "row_level_match": true,
  "vendor_digest": "cc7c162e...",
  "our_digest": "cc7c162e...",
  "mismatches": []
}
```

差異列（`mismatches[]`）欄位：`line`（1-based 含標題）、`item_no`、`warehouse_code`、`vendor_end_inv`、`our_expected_end`、`note`（`value mismatch` | `overflow` | `unparsable: <欄位>`）。

## 6. S1-3 產出物存放與去識別

| 規則 | 內容 |
|---|---|
| 存放 | `<artifacts>/shadow-mode/`；檔名 `<unix秒>-<batch>.json`（batch 經檔名安全化） |
| 版控 | 證據檔屬產物，**不進版控**（`.gitignore` 產物規則）；真實資料之證據嚴禁提交 |
| 去識別 | 證據不含原始儲存格全文；僅含品號、倉別、兩側數值與原因。若輸入含客戶/個資欄，本管線（reports）不攝取；其他管線開通前須先完成去識別欄位白名單 |
| 保存期 | 待 Stage 1 授權時指定（建議 ≥ 90 天，供 M1 差異清單引用） |
| 存取 | 限執行者與裁決者 |

## 7. S1-4 回退

- **回退動作**：停止執行 `wellfit shadow` 命令即可。無常駐程序、無註冊、無對外副作用。
- **已產證據**：保留（作為歷史證據），或依保存期政策整目錄刪除；不得選擇性刪除單筆以修飾差異率。
- **罕見失敗處置**：輸入不可讀→僅回報錯誤；證據寫入失敗→回報錯誤（退出碼 1），不重試、不寫半份檔案之外的殘留（`write` 為單次寫入）。

## 8. 已驗證狀態與後續擴充

### 8.1 已驗證（2026-11-04，本機實測）

| 驗證 | 證據 |
|---|---|
| 單元測試 8 項（MATCH、value mismatch、overflow、unparsable、缺欄零列、亂序表頭、BOM、輸入缺失） | `cargo test -p wellfit_assist` → 36 passed（含既有 28） |
| CLI MATCH 實跑 | `data/fuyi_inventory_report.csv`（合成示範）→ 4/4 驗證、`row_level_match=true`、退出碼 0、證據已寫入 |
| CLI DIFF 實跑 | 注入單列錯誤（650→999）→ 偵測 `line 2: vendor=999, ours=650 (value mismatch)`、退出碼 2 |
| 證據格式 | `wellfit-shadow-evidence/v1` JSON 實檔核對無誤 |

以上以**合成示範資料**執行，屬機制驗證；**尚未**對真實 FlapsErp 匯出執行（那是 Stage 1 授權後的執行項）。

### 8.2 擴充計畫（規格凍結前不動工）

| 擴充 | 觸發條件 |
|---|---|
| `shopee` L1（EX-06 逐單檢核） | Stage 1 首次執行後，依實務需要 |
| `mtrix` L1（VR 規則重跑） | 同上 |
| L2/L3 輸出比對 | FlapsErp 環境與 OQ-F1 回覆到位後另訂規格 |
| Windmill 排程化 | Stage 1 手動期穩定後（每日一次） |

## 9. 檢討條件

首次真實資料擷取後（差異分布未知）；或 Stage 1 授權附帶新條件；或原廠文件（OQ-F1/F2）改變匯出格式定義時。
