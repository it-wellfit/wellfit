# ISO/IEC 27001:2022 ISMS 最佳化導入實施計畫（v1.0）

> 計畫路徑：`C:\Users\stephanie.huang\.local\share\kilo\plans\1792632365810-isms-optimization-implementation.md`
> 目標樹：`C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Knowledge\compliance\iso-27001\`
> 工具樹：`C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Knowledge\tools\compliance\`
> 基線：驗證腳本 `verify-compliance.ps1` 執行結果 PASS=612 / WARN=0 / FAIL=0（2026-10-23T11:29）
> 執行方式：由具文件/腳本修改權限的代理逐條執行；本計畫不自行變更原始碼或文件。

---

## 1. 計畫目標（SMART 與驗證標準）

| 目標 | 可驗證的完成標準 | 參考 |
|---|---|---|
| 10 個主文件完整、內容專業、能直接支撐認證 | 每份 Markdown 含文件管制標頭（編號/狀態/版本/日期/驗證標記）、§1 定位、§最終章「相關文件與待決清單」；V-03 全過 | §2–§12 |
| 權限管理縱切落實所有 35 流程 | `process-control-signoff-matrix.csv` 35 條；每條流程至少關聯 1 個控制；雙軌治理（直營自管／專櫃第三方）明確 | §5, §7 |
| 自動化身分與時間點快照追溯無遺漏 | `automation-access-log.csv`、`access-snapshot-log.csv`；V-24/V-31/V-32 全過 | §5, §8 |
| 3 個主文件建立（KNB-CMP-007/008/009） | 檔案存在且通過 V-03；與現有 7 份主文件一致 | §6 |
| ADR-0012 權威分工完成 | 檔案存在；與 `ims-reconciliation-map.md` 不重複衝突 | §6 |
| 7 個工具腳本可執行 | 每支 .ps1 可在 PowerShell 5.1 下執行；ASCII 主體、中文值域置於 vocab | §8 |
| XLSX 匯出可執行 | `build-compliance-xlsx.ps1` 在本環境可產出可開啟之 XLSX（不依賴 Excel COM 為唯一路徑） | §8 |
| 索引與知識庫連結完整 | `compliance/iso-27001/README.md`、`Knowledge/README.md` v1.2、`Dev/Wellfit/docs/knowledge-index.md` 互相指向且無斷鏈 | §9 |
| 最終驗證全綠 | `verify-compliance.ps1`（含 -Strict）PASS=0 WARN=0 FAIL=0；結束時執行一次並存報告 | §10 |

---

## 2. 設計決策（實作前必須共識，不在執行中變更）

| 決策 | 內容 | 影響範圍 |
|---|---|---|
| **D-A 單一權威** | 關卡 G0–G7、9 種角色、值域以 `tools/compliance/vocab/*.csv` 為唯一來源；腳本與文件不得硬編碼中文值域 | 所有 .ps1 與 Markdown |
| **D-B 雙軌治理** | 直營門市＝內部流程（`our_employee`／`dispatched`）；百貨專櫃＝第三方/租戶介面（`subject_employer_type`，不進權限母體，僅登 TPA 矩陣） | access-register、third-party-access-matrix、F-18、§17 零售專章 |
| **D-C 四維控制狀態** | 設計／實施／運作有效性／證據充分性；不得跳級；D1 系統須有年度測試規劃 | soa-93-controls、control-library、V-21 |
| **D-D 深度分級 D1/D2/D3** | 以 D1 深度驗證／D2 標準控制／D3 登記控管取代規模排除；100% 在範圍內，比例於深度軸 | system-depth-classification、V-13 |
| **D-E ASCII .ps1 主體** | Windows PowerShell 5.1 以系統 ANSI 碼頁解析無 BOM .ps1；中文值域與標題置於 UTF-8 資料檔，腳本以 `Get-Content -Encoding UTF8` 讀取 | 所有 .ps1 |
| **D-F UTF-8 BOM CSV** | 所有 CSV 含 BOM；中文敘述內使用全形逗號「，」避免與分隔符混淆 | 所有 CSV |
| **D-G 不預填組織事實** | 人名、真實日期、門檻值、系統數量、員工數一律標「待確認」；不得對外宣稱已認證或 100% 符合 | 所有 Markdown 與 CSV |
| **D-H 不複製標準原文** | 不複製 ISO Annex A 完整控制文字；條款引用以 ✔/△/未驗證 標記 | 所有 Markdown |

---

## 3. 現況已完成（執行時勿重複建立）

| 項目 | 現況 | 驗證 |
|---|---|---|
| 主文件 10 份 | KNB-CMP-001~006 已存在，含完整目錄與內容 | V-01/V-03 全過 |
| 範本 CSV 28 份 + schema 28 份 | 包含 soa-93、risk-register、access-register、evidence-index、requirements、asset、site、data-flow、cross-border、third-party、automation-access-log、access-snapshot-log、ai、BIA、incident、supplier、context、control-library、gap、crosswalk、CSA、access-review-findings、governance-calendar、operational-calendar、measurement、competence、sod、training、process-control-signoff-matrix、process-step、wbs、wg | V-01/V-02/V-14 全過 |
| 表單 CSV 19 份 | f-01~f-19 全存在 | V-01/V-20 全過 |
| vocab 11 份 | access-types、control-status、decisions、depth-tiers、process-areas、risk-levels、roles、subject-types、taxonomies、treatment-options、verify-marks | CFG00 全過 |
| 工具 9 支 | build-compliance-xlsx、verify-compliance、new-isms-artifact、check-scope-consistency、build-control-trace、build-access-review-packet、collect-evidence、build-workpackage-sheet、check-personal-data-flow | V-01 工具清單全過 |
| 逗號值域與 V 檢查 | V-01~V-32 全過；V-24/V-31/V-32 已補入 verify-compliance.ps1 | 驗證報告 20261023T11:29 |

---

## 4. 實施任務清單（順序不可交換，除非標註獨立）

### W1：主文件內容最佳化（10 份既有文件檢閱與補充）

| # | 任務 | 目標檔案 | 具體內容 | 依賴 | 驗收 |
|---|---|---|---|---|---|
| W1-01 | 檢閱並補充 KNB-CMP-001 | iso-27001-master-plan.md | 補充「所有工作流程的標準作業步驟」章節：申請→核准→授予→複審→撤銷每一步驟的責任、時限、證據；補充 D-C 四維狀態與 D-D 深度分級於所有流程的適用性對照 | — | 內容完整；V-03 全過 |
| W1-02 | 檢閱並補充 KNB-CMP-002 | iso-27001-access-management-sop.md | 明確「直營門市內部流程」與「百貨專櫃第三方介面」的分離機制；補充 automation_identity 四元關係（執行人↔排程↔目標系統↔憑證）的具體作業步驟；補充 MFA 決策矩陣與 break-glass 流程圖 | W1-01 | 內容完整；V-03 全過 |
| W1-03 | 檢閱並補充 KNB-CMP-003 | iso-27001-risk-assessment-method.md | 補充四種處置（避免／減少／轉移／接受）的具體執行步驟；風險接受權限門檻與到期重審機制；深度級別 D1/D2/D3 對風險評估的比例性影響 | W1-01 | 內容完整；V-03 全過 |
| W1-04 | 檢閱並補充 KNB-CMP-004 | iso-27001-audit-signoff-playbook.md | 對齊 G0–G7 關卡；稽核獨立性機制；10.2 四要素（根因／同樣發生檢查／有效性驗證／ISMS 變更）；SME 職能分離限制 F-19 | W1-01 | 內容完整；V-03 全過 |
| W1-05 | 檢閱並補充 KNB-CMP-005 | iso-27001-evidence-and-document-control.md | vocab 驅動證據索引；SHA-256 雜湊 manifest；證據有效性時效窗；CR/LF 相容；證據唯讀與不回寫來源 | W1-01 | 內容完整；V-03 全過 |
| W1-06 | 檢閱並補充 KNB-CMP-006 | multi-standard-evolution-roadmap.md | 補充多標準接軌時間表（SOC2/TISAX/PIPL/42001）；跨境個資（巴西 Alpargatas）列為最高優先 REQ-001；接軌順序與排除留痕規則 | W1-01 | 內容完整；V-03 全過 |

### W2：新建主文件（KNB-CMP-007 ~ KNB-CMP-010）

| # | 任務 | 目標檔案 | 具體內容 | 依賴 | 驗收 |
|---|---|---|---|---|---|
| W2-01 | 建立 KNB-CMP-007 | iso-27001-scope-and-exclusions.md | 範圍聲明（4.3）；排除四要素（理由／影響／補償控制／證據／核准／日期／重審觸發）；D1 深度分級模型；直營／專櫃雙軌治理說明；據點清冊 | W1-01 | 檔案存在；V-01/V-03/V-06/V-07 全過 |
| W2-02 | 建立 KNB-CMP-008 | iso-27001-control-library-method.md | 控制庫編列方法；93 項附錄 A 控制與 5 項組織附加控制；深度級別對照；程序、擁有人、頻率、證據、測試、例外路徑 | W1-01 | 檔案存在；V-03 全過 |
| W2-03 | 建立 KNB-CMP-009 | iso-27001-implementation-roadmap.md | WG-A~F 表單；WP-00~WP-12 工作包；依賴圖；關卡對應（G0-G7）；實施順序表 | W1-01, W2-01 | 檔案存在；V-03 全過 |
| W2-04 | 建立 KNB-CMP-010 | ims-reconciliation-map.md | 與 `Projects/國際標準稽核` 套件的權威分工；§1 優先規則；禁止平行副本 | W2-01 | 檔案存在；V-03 全過 |

### W3：決策紀錄 ADR-0012

| # | 任務 | 目標檔案 | 具體內容 | 依賴 | 驗收 |
|---|---|---|---|---|---|
| W3-01 | 撰寫 ADR-0012 | adr/ADR-0012-isms-authority-precedence.md | 關卡定義、角色詞彙、七模組方法、WBS 骨架、PM-01~15、事件/供應商/BIA/AI schema、附錄 A 93 項清單、其餘欄位、證據模型之權威來源與引用規則 | W2-04 | 檔案存在；V-03 全過 |

### W4：工具腳本建立與修復

| # | 任務 | 目標檔案 | 具體內容 | 依賴 | 驗收 |
|---|---|---|---|---|---|
| W4-01 | 建立 new-isms-artifact.ps1 | tools/compliance/new-isms-artifact.ps1 | 建立受控文件骨架（文件管制標頭、編號、版本）；ASCII 主體 | W1-01 | 可執行；產出正確標頭 |
| W4-02 | 建立 check-scope-consistency.ps1 | tools/compliance/check-scope-consistency.ps1 | 範圍/排除/據點/流程/資產/控制/深度六方一致性；差異報告 | W2-01 | 可執行；可偵測斷鏈 |
| W4-03 | 建立 build-control-trace.ps1 | tools/compliance/build-control-trace.ps1 | 控制→流程→風險→證據→測試五向追溯矩陣；輸出 CSV | W2-02, W2-03 | 可執行；矩陣無斷鏈 |
| W4-04 | 建立 build-access-review-packet.ps1 | tools/compliance/build-access-review-packet.ps1 | 季度權限複審工作包（access-register 母體＋抽樣框＋待決清單） | W1-02 | 可執行；工作包含完整欄位 |
| W4-05 | 建立 collect-evidence.ps1 | tools/compliance/collect-evidence.ps1 | 證據擷取、SHA-256、manifest；唯讀；CR/LF 皆可；不修改來源 | W1-05 | 可執行；manifest 正確 |
| W4-06 | 建立 build-workpackage-sheet.ps1 | tools/compliance/build-workpackage-sheet.ps1 | WBS → 工作包列印表（A/R、DoD、依賴、關卡） | W2-03 | 可執行；表可開啟 |
| W4-07 | 修復 check-personal-data-flow.ps1 | tools/compliance/check-personal-data-flow.ps1 | 資料流 × 個人資料 × 跨境 × 權限遮罩交叉檢查；確認可執行且不壞掉既有檔 | W2-01 | 可執行；報告可產出 |
| W4-08 | 修復 build-compliance-xlsx.ps1 | tools/compliance/build-compliance-xlsx.ps1 | 解決 Excel COM 路徑問題；提供 CSV 中繼或絕對路徑寫入方式，確保可在本機執行並產出可開啟 XLSX | W1-05 | 可執行；XLSX 可由 Excel 開啟且中文正常 |

### W5：索引與知識庫連結

| # | 任務 | 目標檔案 | 具體內容 | 依賴 | 驗收 |
|---|---|---|---|---|---|
| W5-01 | 建立合規總索引 | compliance/iso-27001/README.md | 文件管制清單（全部文件編號、檔名、標題、類別、狀態、版本、日期、擁有者、核准者、驗證標記、附註） | W1~W4 全部 | 索引所指檔案皆存在且編號無重複 |
| W5-02 | 更新 Knowledge/README.md | Knowledge/README.md v1.2 | 目錄地圖新增 compliance/iso-27001；§3 命名例外註記；§7 接手程序新增步驟；§8 專案端對應表 | W5-01 | 連結正確；V-03 全過 |
| W5-03 | 更新 Dev/Wellfit/docs/knowledge-index.md | Dev/Wellfit/docs/knowledge-index.md | 快速入口、目錄樹、ADR 索引（ADR-0009/0010/0011/0012） | W5-01 | 連結正確 |

### W6：最終驗證與收尾

| # | 任務 | 目標檔案 | 具體內容 | 依賴 | 驗收 |
|---|---|---|---|---|---|
| W6-01 | 執行驗證器 | verification/ | `powershell -File tools/compliance/verify-compliance.ps1 -Strict`；PASS=0 WARN=0 FAIL=0；存報告 | W1~W5 全部 | 退出碼 0；報告無 FAIL/WARN |
| W6-02 | 執行 XLSX 匯出 | verification/xlsx/ | 產出 XLSX；可由 Excel 開啟且中文正常 | W4-08 | XLSX 存在且可開啟 |
| W6-03 | 最終內容覆檢 | 所有新建/修改文件 | 檢查是否有預填組織事實、是否複製標準原文、是否有斷鏈、是否有未補「待確認」 | W6-01 | 無違規；清單完整 |

---

## 5. 權限管理縱切（重點落實）

針對使用者強調的「清楚每個工作流程與步驟、智慧化控管、專業實際幫助到所有業務」，執行時需確保：

- **申請→核准→授予→複審→撤銷** 五階段於 KNB-CMP-002 §5 各有責任角色、時限、證據欄位（W1-02）。
- **直營門市**（內部流程）：`our_employee` / `dispatched`，權限與內部控制同一套（W2-01）。
- **百貨專櫃**（第三方/租戶介面）：櫃位人員帳號不進權限母體，須登「介接存取」與「共享資料範圍」，以 `third-party-access-matrix.csv` + F-18 管理（W1-02）。
- **自動化身分**：以人員桌面身分執行之排程，須登記「執行人↔排程↔目標系統↔憑證」四元關係；存於 `automation-access-log.csv`；V-24/V-31 檢查（W1-02）。
- **三波時序**（08:30 蝦皮／10:00 庫存報表／17:00 MTRIX）：權限登錄支援 point-in-time 權限還原；`access-snapshot-log.csv` 記錄三波快照；V-32 檢查（W1-02）。
- **跨境個資**：MTRIX 5 檔含個人資料（ADDRESS/SALES_CONTACT/PHONE_NUMBERS/MANAGER_NAME 等）輸出至巴西 Alpargatas，列為 REQ-001 最高優先；跨境評估 `cross-border-transfer-assessment.csv` + F-17（W2-01）。
- **職能分離**：WG-B 權限授予者不得為核准者；WG-D 稽核者不得稽核其本人設計或執行之控制；WG-C 風險接受者不得為控制執行者；SME 無法分離時登 `sod-limitation-register.csv`（W1-02/W1-04）。

---

## 6. 風險與防護

| 風險 | 防護 |
|---|---|
| 與平行代理同時編輯同一檔案 | 實作前先 `ls` 比對目錄；若檔案已存在則擴充不覆寫（RK-7）；編輯失敗時重新讀取最新內容再重試 |
| .ps1 主體含非 ASCII 導致 PowerShell 5.1 解析失敗 | 所有 .ps1 主體僅 ASCII；中文置於 UTF-8 vocab 資料檔；執行前以 `verify-compliance.ps1` 自身驗證 |
| CSV 欄數不一致或主鍵重複 | V-02 自動檢查；每支新建 CSV 須先在腳本外以 `Import-Csv` 測試 |
| 排除理由為「不需要」或「成本過高」 | V-07 擋下；排除項須四要素齊備（W2-01） |
| 對外宣稱已認證或 100% 符合 | 所有 Markdown 僅以 ✔/△/未驗證 標記；V-16 掃描禁則宣稱 |

---

## 7. 驗收定義（DoD）

- 所有 W1~W6 任務標記完成。
- `verify-compliance.ps1 -Strict` 退出碼 0，PASS=0 WARN=0 FAIL=0。
- `build-compliance-xlsx.ps1` 可產出可開啟之 XLSX。
- 所有新建 Markdown 含文件管制標頭；所有 CSV 為 UTF-8 BOM、欄數一致、主鍵唯一。
- 索引所指向之檔案皆存在且編號無重複。
- 未預填任何組織事實；未複製受版權保護之 ISO Annex A 完整控制文字；未撰寫合規結論或認證宣稱。

---

## 8. 下一動作

此計畫已達「implementation-ready」。執行代理應從 W1-01 開始依序完成，並在 W6-01 執行最終驗證後，向使用者回報 PASS/WARN/FAIL 結果與任何 W6-03 發現的內容覆檢問題。
