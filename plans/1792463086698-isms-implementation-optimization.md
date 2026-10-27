# ISMS 實施最佳化 — 完整執行計畫

> 文件編號：KNB-CMP-PLAN-001｜狀態：Executable｜版本：v1.0｜日期：2026-10-23
> 本計畫為「下一位代理可直接執行」的檔案級行動清單；不預填 Q1–Q5 未決事實。
> 所有變更後必須重跑 `verify-compliance.ps1`，結果須為 PASS>0 / WARN=0 / FAIL=0。

## 1. 現況與基線（已驗證）

| 項目 | 狀態 | 證據 |
|---|---|---|
| ISO/IEC 27001:2022 合規套件（10 主文、23 範本、15 表單、9 工具） | 已建立 | `Knowledge/compliance/iso-27001/README.md` |
| `verify-compliance.ps1` | PASS=614 / WARN=0 / FAIL=0 | `compliance/iso-27001/verification/verify-report-*.md` |
| `check-scope-consistency.ps1` | fail=0 / warn=0 | 執行後報告於同目錄 |
| `build-control-trace.ps1` | controls=93 covered=93 uncovered=0 | 產出 `control-trace-matrix.csv` |
| `collect-evidence.ps1` | evidence_rows=101 collected=0 (dry_run) | 產出 `evidence-manifest.csv` |
| `build-access-review-packet.ps1` | population=13 strata=4 sampled=13 | 產出 `access-review-pending.csv` |
| `build-workpackage-sheet.ps1` | work_packages=13 work_groups=6 | 產出 `workpackage-sheet.md` |
| `build-compliance-xlsx.ps1` | 26 CSV → XLSX 全部 OK | 參數需：`-CsvDir .../templates -OutDir ...` |
| `new-isms-artifact.ps1` | 需 `-Path -Number -Title` 必填參數 | 未執行（參數缺失導致報錯，非腳本錯誤） |
| 單一控制庫 / 單一 SoA / 單一證據索引 | ADR-0010 已定案 | `Knowledge/adr/ADR-0010-*.md` |
| IMS / 知識庫 權威分工 | ADR-0012 已定案 | `Knowledge/adr/ADR-0012-isms-authority-precedence.md` |

### 既有反模式與防線（已定案）
- 不預填 Q1–Q5（組織法人、據點數、系統清單、AI 使用、客戶/監管要求）→ 以 `待確認`/`待判定` 佔位
- 合成行以 `EXAMPLE` 標記，不當真實資料
- 不宣稱「符合/已通過/認證」除非可回溯至有效證據 ID 與範圍
- 零 `unsafe`、Clippy 零警告、零浮點金額、禁止序位退讓（Dev/Wellfit 不變量，不得為合規修改）

## 2. 端到端工作流程（10 步，每步可機械驗證）

| # | 工作流程 | 輸入 | 輸出 | 驗證關卡 | 執行腳本 |
|---:|---|---|---|---|---|
| 1 | 範圍定義 | Q1/Q2 待確認 | `scope-exclusion-register.csv` + `system-depth-classification.csv` | V-02/V-06/V-07 | `check-scope-consistency.ps1` |
| 2 | 利害關係人與需求 | Q5 待確認 | `context-interested-parties.csv` + `requirements-register.csv` | V-04/V-05 | `verify-compliance.ps1` |
| 3 | 資產與深度分級 | Q2 待確認 | `asset-inventory.csv` | V-06 | `check-scope-consistency.ps1` |
| 4 | 風險評估 | 1–3 | `risk-register.csv` | V-07/V-09 | `verify-compliance.ps1` |
| 5 | 控制庫（SoA） | 4 | `soa-93-controls.csv` + `control-library.csv` | V-04/V-08 | `build-control-trace.ps1` |
| 6 | 權限生命週期 | 5 | `access-register.csv` + `access-snapshot-log.csv` + `automation-access-log.csv` | V-10/V-11/V-17 | `build-access-review-packet.ps1` |
| 7 | 流程簽核 | 5 | `process-control-signoff-matrix.csv` | V-08/V-12/V-18 | `verify-compliance.ps1` |
| 8 | 證據收集 | 6/7 | `evidence-index.csv` + `collect-evidence.ps1` 產出 | V-13/V-14 | `collect-evidence.ps1` |
| 9 | 差距與 WBS | 4–8 | `gap-assessment.csv` + `wbs-workpackages.csv` + `wg-workgroups.csv` | V-15/V-16 | `build-workpackage-sheet.ps1` |
| 10 | 稽核與管理評審 | 9 | `internal-audit-program.csv` + `control-test-plan.csv` + `document-control-list.csv` | V-17–V-20 | `verify-compliance.ps1` |

## 3. 智慧化控管點（自動檢查，每步不可跳過）

| 代號 | 控管內容 | 檢查方式 | 阻擋條件 |
|---:|---|---|---|
| G-01 | 欄位契約：所有 CSV 以名稱解析，禁止序位退讓 | `*-schema.md` 欄位名一致 | 欄數不一致 → V-02 FAIL |
| G-02 | 值域：所有列舉值存在於 `vocab/*.csv`；新增值域先改 vocab | `verify-compliance.ps1` V-05/V-11/V-12/V-15 | 非法值 → FAIL |
| G-03 | 跨向一致性：範圍↔排除↔流程↔資產↔控制 四向一致 | `check-scope-consistency.ps1` | 孤立節點 → fail |
| G-04 | 追溯：控制→流程→風險→證據→測試 五向追溯 | `build-control-trace.ps1` | uncovered > 0 → 修正 |
| G-05 | 證據雜湊：證據檔 SHA-256 登錄 `evidence-index.csv`；來源唯讀 | `collect-evidence.ps1` | 回寫來源 → 禁止 |
| G-06 | 文件管制：新增受控文件用 `new-isms-artifact.ps1`，自動更新 `document-control-list.csv` | `new-isms-artifact.ps1` | 缺少文件管制標頭 → V-03 FAIL |

## 4. 優先序與行動清單（下一位代理直接執行）

### P0（阻擋稽核，立即執行）
| # | 動作 | 檔案/腳本 | 完成標準 |
|---:|---|---|---|
| P0-1 | 執行基線驗證，確認 0 FAIL | `tools/compliance/verify-compliance.ps1` | PASS>0 / WARN=0 / FAIL=0 |
| P0-2 | 執行四向一致性檢查 | `tools/compliance/check-scope-consistency.ps1` | fail=0 / warn=0 |
| P0-3 | 產出五向追溯矩陣；孤立控制標記為差距 | `tools/compliance/build-control-trace.ps1` | covered=93 uncovered=0 |
| P0-4 | 產出權限複審工作包；待決清單逐項簽核 | `tools/compliance/build-access-review-packet.ps1` | pending 清單有核准/駁回決定 |

### P1（提升專業度，本週）
| # | 動作 | 檔案/腳本 | 完成標準 |
|---:|---|---|---|
| P1-1 | 對齊 `control-self-assessment.csv` 與 `gap-assessment.csv`；缺口自動生成 WBS 工作包 | `templates/control-self-assessment.csv`、`templates/gap-assessment.csv`、`build-workpackage-sheet.ps1` | 每個缺口有對應工作包 |
| P1-2 | 同步 `governance-calendar.csv` 與 `internal-audit-program.csv`；稽核計畫覆蓋全部控制 | `templates/governance-calendar.csv`、`templates/internal-audit-program.csv` | 內稽覆蓋條款 4–10 及全部適用控制 |
| P1-3 | 產出格式化 XLSX 供審閱 | `tools/compliance/build-compliance-xlsx.ps1 -CsvDir templates -OutDir $env:TEMP/compliance-xlsx` | 26 CSV 全部轉換 OK |
| P1-4 | 新增受控文件時用 `new-isms-artifact.ps1` 更新管制清單 | `tools/compliance/new-isms-artifact.ps1 -Path -Number -Title` | `document-control-list.csv` 有新項 |

### P2（細節完善，持續）
| # | 動作 | 檔案/腳本 | 完成標準 |
|---:|---|---|---|
| P2-1 | 每次變更後重跑 `verify-compliance.ps1`，報告存於 `verification/` | `verify-compliance.ps1` | FAIL=0 |
| P2-2 | 多標準接軌：依 `multi-standard-evolution-roadmap.md` 順序導入 ISO 27701/27017/27018 | `Knowledge/compliance/iso-27001/multi-standard-evolution-roadmap.md` | 對照表 `crosswalk.csv` 新增對應 |
| P2-3 | Q1–Q5 填入後，所有 `待確認`/`待判定` 佔位更新為實值並附來源 | 各模板 CSV | 禁用待確認佔位於正式登錄 |

## 5. 工具參數速查（可直接貼上執行）

| 工具 | 完整指令（範例） |
|---|---|
| verify | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/compliance/verify-compliance.ps1` |
| check-scope-consistency | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/compliance/check-scope-consistency.ps1` |
| build-control-trace | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/compliance/build-control-trace.ps1` |
| build-access-review-packet | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/compliance/build-access-review-packet.ps1` |
| collect-evidence | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/compliance/collect-evidence.ps1` |
| build-workpackage-sheet | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/compliance/build-workpackage-sheet.ps1` |
| build-compliance-xlsx | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/compliance/build-compliance-xlsx.ps1 -CsvDir compliance/iso-27001/templates -OutDir $env:TEMP/compliance-xlsx` |
| new-isms-artifact | `powershell -NoProfile -ExecutionPolicy Bypass -File tools/compliance/new-isms-artifact.ps1 -Path <doc.md> -Number <KNB-CMP-###> -Title <標題>` |

## 6. 完成定義（DoD）

- `verify-compliance.ps1`：PASS>0 / WARN=0 / FAIL=0
- `check-scope-consistency.ps1`：fail=0 / warn=0，無孤立節點
- `build-control-trace.ps1`：93 項控制皆有流程/風險/證據對應，uncovered=0
- 所有文件狀態由 `Draft` → `Review` → `Approved` 逐級簽核，無跳級
- Q1–Q5 填入後，不得留 `待確認`/`待判定` 於正式登錄
- 每一份受控 Markdown 具備文件管制標頭（文件編號／狀態／版本／日期／驗證標記）

## 7. 風險與回滾

| 風險 | 影響 | 緩降措施 | 回滾方式 |
|---|---|---|---|
| 範圍未核准（U-01★） | 所有後續工作邊界不明 | G-0 關卡前不推進 P1–P2 | 保留 `待確認` 佔位，不填實值 |
| 高風險無處置或有效接受 | G-3/G-4 阻擋 | 風險擁有者具名核准 | 風險登錄回退至上一版本 |
| 證據來源不可確認 | V-10 WARN | E3 證據索引 + SHA-256 | 刪除無來源證據行 |
| 宣稱與證據脫節 | 對外合規宣稱無效 | 禁止「符合/已通過」措辭 | 移除對外宣稱 |
| 值域腳本硬編碼 | 維護成本高 | 所有值域僅於 `vocab/*.csv` | 恢復資料驅動值域 |

## 8. 禁止事項（硬性）

- 不得將合成示範列（EXAMPLE）當作組織現況或控制有效性證據。
- 不得預填 Q1–Q5 未決問題。
- 不得複製受版權保護之國際標準附錄 A 完整控制文字；僅記載控制編號、英文短標題與本地化作業規格。
- 不得對外宣稱「符合」「已通過」「認證」，除非可回溯至有效證據識別碼與認證範圍。
- 不得為合規而修改 Dev/Wellfit 既有不變量（零 unsafe、Clippy 零警告、零浮點金額、禁止序位退讓、合成資料不得呈現為真實）。
- 不得建立平行控制編號、平行狀態欄或平行門檻（ADR-0010/0012）。

## 9. 開放決策（需專案負責人裁示）

| # | 決策事項 | 建議答案 | 責任角色 |
|---|---|---|---|
| D-1 | ISMS 範圍邊界（場所、系統、流程） | 以 35 條流程與既有專案（Dev/Wellfit、哈瓦士串接、蝦皮、RPA）為初始範圍，超出者寫排除理由 | 執行贊助人／最高管理層 |
| D-2 | 認證目標時程 | 先不對外承諾時程；G-3 確定後再對外 | 執行贊助人／最高管理層 |
| D-3 | PIPL 適用性 | 法遵/外部顧問意見後再判定（U-03★） | 法務／法遵／隱私職能 |
| D-4 | 汽車客戶（Alpargatas）資安要求 | 索取客戶要求文件後納入 P-OPS-03/P-OPS-07 | 法務／法遵／隱私職能 |

## 10. 驗證指令（執行完畢後必跑）

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File "Knowledge/tools/compliance/verify-compliance.ps1"
powershell -NoProfile -ExecutionPolicy Bypass -File "Knowledge/tools/compliance/check-scope-consistency.ps1"
powershell -NoProfile -ExecutionPolicy Bypass -File "Knowledge/tools/compliance/build-control-trace.ps1"
```
