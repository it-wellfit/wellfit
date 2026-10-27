# ISO/IEC 27001:2022 全範圍 ISMS 導入計畫（權限管理縱切）

> 計畫版本：v2.0（取代 v1.0）｜日期：2026-10-22｜狀態：待核准
> 目標工作區：`C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Knowledge\compliance\iso-27001\`
> 對齊套件：`C:\Users\stephanie.huang\Desktop\新人訓練\Projects\國際標準稽核\`（IMS 套件，57 檔）
> 工具工作區：`C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Knowledge\tools\compliance\`
> 本版新增：§0.3 已知組織事實（12 項已查證）、§2 D13–D16、§17 人事時地物四維完整性稽核（25 項缺口）

---

## 0. 目標與範圍

### 0.1 已確認的範圍決定（使用者裁決 2026-10-22）

**全組織納入 ISMS 範圍**：財務／物流／電商／百貨專櫃／直營門市／全流程作業／人力／資訊／實體，
且**大型、中型、小型核心與周邊系統全部納入**。

### 0.2 已確認的場域歸屬（決定雙軌治理）

| 場域 | 歸屬 | 治理路徑 |
|---|---|---|
| **直營門市** | 我方自管（人力、POS、門市網路皆為我方） | **內部流程**：主體類型 `our_employee` / `dispatched`，權限登錄與內部控制同一套 |
| **百貨專櫃** | **品牌方人力**（系統多由百貨方或品牌方提供） | **第三方/租戶介面**：我方管契約、報表、介接存取、資料流；櫃位人員帳號不進我方權限母體，但須登錄「介接存取」與「共享資料範圍」 |

> 此分流直接決定：`access-register.csv` 增 `subject_employer_type`；新增 `third-party-access-matrix.csv`；
> 專櫃櫃位人員帳號以「第三方存取」登錄，而非我方員工帳號。

### 0.3 已知組織事實（✔ 已對來源文件逐字查證，非推測）

| # | 事實 | 來源 | 對 ISMS 的直接影響 |
|---|---|---|---|
| F-01 | 大前實業有限公司，統一編號 **28467935** | `wellfit-blueprint-v1` [p1] | 範圍聲明之法人主體（唯一已確認法人） |
| F-02 | 核心交易系統＝**Windows 桌面版「輔翼 ERP」**（非 Web/非 IdP） | 同上 [p1] | 權限控制點在 **Windows 帳號/群組 + ERP 內部角色**；不得設計 SSO/MFA 假設 |
| F-03 | 三大專案分屬**營運與管理部門、電商營運團隊、品牌經銷與 BI 團隊** | 同上 [p2] | 跨部門發起 → ISMS 負責人須有跨部門授權；治理歸屬不清為首要風險 |
| F-04 | 輔翼 ERP 為品號、**通路部門（SHOP_CODE）**、**倉庫代碼**之驗證基準；含**跨倉移轉**與**折讓**規則 | 同上 [p3] | 資料分類與權限範圍必須含**倉庫維度**；跨倉移轉是權限與稽核軌跡的天然斷點 |
| F-05 | 專案一為桌面自動化擷取，採 **UI Automation／剪貼簿／原生匯出**，明示「不修改 ERP、不繞過權限」 | 同上 [p3] | 自動化**以人員桌面身分執行** → 須新增 `automation_identity` 主體類型（D15） |
| F-06 | 專案三輸出 5 檔 CSV 至**巴西 Alpargatas International**；欄位含 `ADDRESS`（完整地址）、`SALES_CONTACT`（PoS 聯絡人姓名）、`PHONE_NUMBERS`、`MANAGER_NAME`/`SUPERVISOR_NAME`/`SALESPERSON_NAME` | `mtrix-field-dict-v2` 快照（逐字） | **構成境外提供個人資料** → 個資法境外提供義務（D14，REQ-001 最高優先） |
| F-07 | 蝦皮匯入含收件人、電話、地址、買家識別；規格已載「須訂遮罩、權限與保存期限」，且「個資與匿名化規則待定」「個資／憑證治理未定」 | `shopee-requirements-v21` [p13][p15][p17] | 個資遮罩/權限/保存為**已知未解事項**，可作為風險登錄真實初始條目 |
| F-08 | 外部 RPA 供應商**百林商智（EMILY）**，有正式報價單與信函 | `source-register.md` 附錄 A；`wellfit-blueprint-v1` [p1] | 第三方服務人員帳號 → A.5.19–A.5.23 + 第三方存取矩陣 |
| F-09 | 三波時序：**08:30 蝦皮 / 10:00 庫存報表 / 17:00 MTRIX** | `Dev/GEMINI.md` | 排程型身分；稽核軌跡須支援 **point-in-time 權限還原** |
| F-10 | 冪等帳本 SQLite（append-only）、`ReleaseGate` 白名單網域 `dachen.com.tw`/`wellfit.tw`/`alpargatas.com` | `Knowledge/README.md` B-5/B-8 | 既有可稽核軌跡 → A.8.15/A.8.16 證據來源（勿重造） |
| F-11 | 介接 CSV 為 **CR only** 換行（無 LF） | `source-register.md` §6 | 證據擷取與雜湊工具**不得**假設 LF 分行；需以 bytes 層級處理 |
| F-12 | 直營門市自管；百貨專櫃為品牌方人力 | 使用者裁決 2026-10-22 | 雙軌治理（§0.2） |

> **未查證者不得推測**：`F-01`~`F-11` 為 ✔；其餘組織事實（員工數、門市數、櫃位數、倉庫數、供應商數、系統清單、
> AI 使用情形、支付卡架構）**一律標「待盤點」**，並列入 §14 未決問題。

### 0.4 排除項規則（強制）

排除僅能透過 `templates/scope-exclusion-register.csv` 登錄，且須齊備：
**組織情境層面之可辯護理由 ＋ 對風險與體系完整性之影響評估 ＋ 補償控制 ＋ 證據引用 ＋ 具名核准角色 ＋ 核准日 ＋ 重審觸發**。
無理由或無核准之排除 → 驗證腳本 **FAIL**。理由不得為「不需要」「成本過高」（V-07）。

### 0.5 本計畫不做

- 不採購 GRC 平台（Phase 1 以既有 `build.ps1` + CSV 產物鏈支撐）
- 不預填任何組織事實（人名、真實日期、門檻值、系統數量、員工數）
- 不修改 `Dev/Wellfit` 程式碼（僅引用既有 `IdempotencyLedger`、`wellfit audit` 為證據來源）
- 不主張任何認證結果或法規符合性結論
- 不複製受版權保護之 ISO Annex A 完整控制文字

---

## 1. 權威性與優先規則（消除與 IMS 套件的分歧）

| 產物類型 | 權威來源 | 另一側角色 |
|---|---|---|
| 關卡定義 G0–G7、決策權、阻擋條件 | **IMS** `49_Phase0-1_驗收關卡.csv` | KB 以 ID 引用，不得另立 |
| 角色詞彙（9 種） | **IMS** `41_…手冊.md` §2 | KB 沿用；領域差異以 `process_area` 承載，不新增角色 |
| 七模組方法框架 | **IMS** `47_…執行藍圖.md` §3 | KB 以此為模組骨架落地 |
| WBS / 工作包骨架 | **IMS** `48_Phase0-1_WBS.csv`、`46_Phase1_WBS與關卡追蹤.csv` | KB 產出細化與縱切版本 |
| 專案管理表單 PM-01..PM-15 | **IMS** `21`–`35` | KB 僅產出 ISO 27001 專屬 F-01..F-19 |
| **事件登錄 schema** | **IMS** `20_事件與通報義務登錄範本.csv` | KB `incident-register.csv` 採**同欄位**並註明 `source_schema_ref` |
| **供應商登錄 schema** | **IMS** `18_供應商風險與退出登錄範本.csv` | KB `supplier-register.csv` 同上 |
| **BIA/復原 schema** | **IMS** `19_BIA與復原測試登錄範本.csv` | KB `bia-service-continuity.csv` 同上 |
| **AI 清冊 schema** | **IMS** `17_AI系統清冊範本.csv` | KB `ai-system-inventory.csv` 同上 |
| 附錄 A 93 項清單與 SoA 母表 | **KB** `templates/soa-93-controls.csv` | IMS 側 `37_IMS_控制矩陣.csv` 引用 `control_id` |
| 其餘登錄表欄位定義與值域 | **KB** `templates/*-schema.md` | IMS 側 CSV 引用同一 schema |
| 證據模型與文件管制 | **KB** KNB-CMP-005 | IMS 側引用 EVD-ID 規則 |

**規則**：同一事實只允許一個權威來源；另一側以 ID + 連結引用，**禁止平行副本**。
新增此優先規則於 `adr/ADR-0012-isms-authority-precedence.md`。

---

## 2. 設計決策

| # | 決策 | 理由 | 影響 |
|---|---|---|---|
| **D1** | **控制深度三級制**取代「大小系統排除」：D1 深度驗證／D2 標準控制／D3 登記控管 | ISO 4.3 要求控制與風險相稱；用深度軸可在 100% 在範圍內滿足比例原則 | `system-depth-classification.csv`；SoA 增 `depth_tier` |
| **D2** | **SoA 狀態改 4 維獨立量尺**：設計／實施／運作有效性／證據充分性 | IMS `15_MasterControlMatrix` 已用 4 維；單欄無法區分「部分實作」與「已有效」 | `soa-93-controls.csv` 增 4 欄取代 1 欄 |
| **D3** | **角色 9 種不新增**；新增 `process_area` 11 類 | 避免核准權衝突；領域差異由流程領域承載 | 全登錄表增 `process_area` |
| **D4** | **值域語言＝繁體中文為正典**；驗證腳本改為**從外部 UTF-8 資料檔載入值域** | 現有 7 CSV 全為中文值；KB §6 規則 4 明令 .ps1 不得含非 ASCII 字面量。**此為腳本失效根因，單改 ASCII 無效** | `tools/compliance/vocab/*.csv`；重寫 `verify-compliance.ps1` |
| **D5** | **流程清單 18 → 35 筆**，既有 `process_id` **保持穩定**（P-OPS-* 不改名） | ID 穩定避免追溯斷裂 | `process-control-signoff-matrix.csv` |
| **D6** | **風險處置對齊 IMS**：避免／減少／轉移／接受 | IMS WP-05 §4 | `risk-register.csv` 值域 |
| **D7** | **`subject_type` 與 `access_type` 分離** | 現 schema 中 `privileged` 同時出現在兩者，語意混淆 | `access-register.csv` 重整 |
| **D8** | **關卡沿用 IMS G0–G7**；原 KB G-0..G-4 降級為別名 | 消除雙軌 | KNB-CMP-001 §4.3 |
| **D9** | **門檻值與指派人數一律不預填**，改「方法已定義、值待核准」欄位 | ADR-0007 | 全部登錄表 |
| **D10** | **排除項四要素強制**（§0.4） | 6.1.3 d) ＋ 4.3 | `scope-exclusion-register.csv` |
| **D11** | **PIPL／GDPR／DORA／NIS2／EU AI Act 為「候選要求」**，不得直接判定適用或不適用 | IMS `02_Phase0` §5 決策規則 | `requirements-register.csv` |
| **D12** | **不新增資料庫或新服務**；狀態存於 CSV + 檔案版本 | 與既有產物鏈一致 | 全部 |
| **D13** | **混合場域雙軌治理**：直營門市＝內部流程；百貨專櫃＝第三方/租戶介面 | 使用者裁決 §0.2；櫃位人員非我方員工 | `access-register.csv` 增 `subject_employer_type`；新增 `third-party-access-matrix.csv`；F-18 |
| **D14** | **跨境個人資料列為 REQ-001 最高優先**：MTRIX 5 檔含個人資料（F-06）→ 個資法境外提供評估（告知＋同意，或依施行細則第 26 條情形）；蝦皮匯入為 REQ-002 | F-06/F-07 為 ✔ 事實，非推測 | `cross-border-transfer-assessment.csv`；F-17；V-22 |
| **D15** | **`automation_identity` 為 access-register 一級主體類型** | F-05：自動化以人員桌面身分執行。若不登錄「執行人↔排程↔目標系統↔憑證」四元關係，權限無法追溯至自然人（違反 A.5.16 唯一識別） | `access-register.csv` 增類型；V-24 |
| **D16** | **季節性營運節奏對齊治理** | 零售/電商旺季會偏誤母體與稽核抽樣。權限複審排於旺季前；稽核抽樣須跨旺季或明示 coverage limitation | `operational-calendar.csv`；V-28 |

---

## 3. 交付物目錄樹（目標狀態）

```
Knowledge/compliance/iso-27001/
├── README.md                                   # 索引 + 文件管制清單
├── iso-27001-master-plan.md            (KNB-CMP-001 v2.0)
├── iso-27001-scope-and-exclusions.md   (KNB-CMP-007 v1.0)  # 範圍/排除/深度分級
├── iso-27001-access-management-sop.md  (KNB-CMP-002 v1.1)  # 零售/專櫃/automation_identity
├── iso-27001-risk-assessment-method.md (KNB-CMP-003 v1.1)
├── iso-27001-audit-signoff-playbook.md (KNB-CMP-004 v1.1)
├── iso-27001-evidence-and-document-control.md (KNB-CMP-005 v1.1)
├── multi-standard-evolution-roadmap.md (KNB-CMP-006 v1.1)
├── iso-27001-control-library-method.md (KNB-CMP-008 v1.0)
├── iso-27001-implementation-roadmap.md (KNB-CMP-009 v1.0)  # WBS + 工作組
├── ims-reconciliation-map.md            (KNB-CMP-010 v1.0)
├── templates/   (32 CSV + 32 schema.md)
├── forms/       (19 CSV + forms/README.md)
└── verification/(自動產出)

Knowledge/tools/compliance/
├── build-compliance-xlsx.ps1     (既有，擴充 vocab 標題)
├── verify-compliance.ps1         (重寫：資料驅動值域，V-01..V-30)
└── vocab/  (11 CSV)

Knowledge/adr/ADR-0012-isms-authority-precedence.md   (新)
```

---

## 4. 模組 M1–M7 × 權限管理縱切

| 模組 | KB 產物 | 權限管理縱切 |
|---|---|---|
| M1 Requirements Register | `requirements-register.csv` | 個資法境外提供（REQ-001）、蝦皮個資（REQ-002）、汽車客戶資安條款、PCI |
| M2 Gap Assessment | `gap-assessment.csv` | 每控制×每流程四段式缺口（文件缺／設計缺／實施缺／運作缺） |
| M3 Control Library | `control-library.csv`（93+ 附加控制） | 含深度級別、程序、擁有人、頻率、證據、測試、例外路徑 |
| M4 Policy & Procedure | KNB-CMP-002/003/004/005 ＋ F-01..F-19 | 權限治理原則、申請/授予/複審/撤銷、例外與風險接受、跨境提供程序 |
| M5 Implementation Roadmap | KNB-CMP-009 ＋ `wbs-workpackages.csv` ＋ `wg-workgroups.csv` | WP-07 身分與權限生命週期為主線 |
| M6 Evidence & Testing | `evidence-index.csv`、`control-test-plan.csv`、`test-results.csv` | 權限母體完整性（多來源對帳）、離職員時效、抽查與複測 |
| M7 Continuous Improvement | `nonconformity-register.csv`、`governance-calendar.csv`、`metrics-register.csv` | 複審發現閉環、季度 KPI、管理評審輸入 |

---

## 5. 工作組（WG）

| WG | 名稱 | Accountable | 覆蓋 process_area | 主要交付物 | 關卡 |
|---|---|---|---|---|---|
| **WG-A** | 治理・範圍・政策 | 執行贊助人／最高管理層 | GOV 全域 | 章程、範圍聲明、排除登錄、政策、目標、角色任命 | G0 G1 G2 |
| **WG-B** | 身分與權限（★主線） | ISMS 負責人 | FIN LOG ECS RET SVC HR IT | 權限原則、角色矩陣、權限登錄、生命週期、複審發現、職能分離限制 | G1 G4 G5 |
| **WG-C** | 風險與控制設計 | 風險授權人 | 全域 | 風險方法、風險登錄、SoA、控制庫、深度分級、處置計畫 | G2 G3 |
| **WG-D** | 證據・測試・稽核 | 內部稽核／獨立覆核者 | 全域 | 證據索引、測試計畫、稽核計畫、內稽報告、CAPA | G4 G5 G6 |
| **WG-E** | 事件・營運持續・實體 | 資安／IT／工程 | IT PHY SUP | 事件登錄（IMS 20 schema）、BIA/復原（IMS 19 schema）、實體安全 | G4 G5 |
| **WG-F** | 供應商・第三方・個資 | 法務／法遵／隱私職能 | SUP MKT SVC | 供應商登錄（IMS 18 schema）、第三方存取矩陣、個資處理活動、跨境評估、AI 清冊 | G2 G4 |

**職能分離紅線（違反＝FAIL）**：WG-B 權限授予者不得為核准者；WG-D 稽核者不得稽核其本人設計或執行之控制；WG-C 風險接受者不得為控制執行者；同一自然人不得同時為同一控制的執行者與稽核者。
**SME 無法分離時** → 登錄於 `sod-limitation-register.csv`（F-19），記錄限制、替代覆核者、殘餘風險、到期日。

---

## 6. 工作包 × 關卡

| WP | 名稱 | 關鍵輸出 | 關卡 | 主責 WG | 前置 |
|---|---|---|---|---|---|
| WP-00 | 專案治理啟動 | 章程、角色任命、決策權、受控工作區 | G0 | A | — |
| WP-01 | 組織情境・利害關係人・範圍・排除 | 情境登錄、範圍聲明、排除登錄、據點清冊 | G1 | A | WP-00 |
| WP-02 | 需求與法規適用性 | 需求登錄（法律/契約/客戶/標準）、跨境評估、未決法律清單 | G1 | F | WP-01 |
| WP-03 | 資產盤點與深度分級 | 資產清冊（資訊/服務/端點/雲端/身分/供應商/實體/紙本）、D1/D2/D3 分級、來源對帳 | G2 | B/C | WP-01 |
| **WP-03A** | **資料流・個資・支付卡盤點** | 資料流圖（資料主體→蒐集→目的→系統→接收者→跨境→保存→刪除）、個資處理活動、支付卡範圍假說 | G2 | F | WP-01, WP-03 |
| **WP-03B** | **AI・自動化・第三方存取盤點** | AI 清冊（IMS 17 schema）、自動化身分清冊、第三方存取矩陣、供應商母體 | G2 | B/F | WP-03 |
| WP-04 | 風險方法與風險評估 | 風險方法核准、風險登錄 | G3 | C | WP-02,03,03A,03B |
| WP-05 | SoA 與 Annex A 對帳 | 93/93 判定、排除理由 | G3 | C | WP-04 |
| WP-06 | 控制庫與作業規格 | control-library.csv、程序草案 | G4 | C/B | WP-05 |
| **WP-07** | **身分與權限生命週期** | 權限原則、角色矩陣、權限登錄、申請/授予/複審/撤銷、MFA 清單、職能分離矩陣、automation_identity 對應 | G4 | **B** | WP-03,03B,WP-06 |
| WP-08 | 政策與作業程序 | 最小足夠政策集、程序集（含跨境提供程序、第三方存取程序） | G4 | A/B/C/F | WP-06 |
| WP-09 | 證據架構與 KPI | 證據索引（含時效窗）、指標定義與基線方法 | G4 | D | WP-06 |
| WP-10 | 測試與內部稽核 | 測試計畫、第一輪測試、內稽報告 | G5 G6 | D | WP-07,WP-09 |
| WP-11 | 管理評審與改善 | 評審紀錄、決議追蹤 | G6 | A | WP-10 |
| WP-12 | 維運移交 | BAU 交接、未結風險處置 | G7 | A | WP-11 |

---

## 7. 登錄範本集（templates/，32 CSV）

### 7.1 既有 9 檔之變更

| 檔案 | 動作 |
|---|---|
| `soa-93-controls.csv` | **改欄位**：`implementation_status` → `design_status`／`implementation_state`／`operating_effectiveness`／`evidence_sufficiency`；增 `depth_tier`、`process_area`、`linked_risk_ids`、`requirement_ids`、`soa_basis`、`exclusion_justification` |
| `risk-register.csv` | `treatment_option` 改 4 選項；增 `process_area`、`risk_acceptance_by`、`acceptance_expiry`、`residual_risk_basis`、`reassessment_trigger`、`evidence_ref`（接入 F-06/F-07 已知未解事項為真實初始條目） |
| `access-register.csv` | `subject_type`／`access_type` 分離；增 `subject_employer_type`（D13）、`automation_identity`（D15）、`process_area`、`warehouse_or_shop_scope`、`mfa_enabled`、`last_used_at`、`justification_seed`、`privileged_jit_flag`、`review_cycle`、`source_of_record`、`point_in_time_snapshot_ref` |
| `evidence-index.csv` | 增 `process_area`、`population_basis`、`coverage_period`、`evidence_validity_window`（時效窗）、`integrity_method`、`custodian_role`、`access_restriction`、`retention_basis`、`timezone_basis` |
| `internal-audit-program.csv` | 對齊 G0–G7；增 `independence_check`、`sample_rationale`、`seasonality_covered`、`retest_due` |
| `nonconformity-register.csv` | 增 `root_cause`、`same_occurrence_check`、`effectiveness_verification`、`isms_change_required`（10.2 四要素） |
| `process-control-signoff-matrix.csv` | **18 → 35 筆**；既有 ID 保留、重新歸類 `process_category`；增 `depth_tier`、`criticality`、`seasonality_peak`、`disaster_recovery_tier` |
| `templates/README.md` | 重寫：清單、值域|ID 命名、遷移說明、IMS schema 對應 |

### 7.2 新增 23 檔 CSV

| 檔案 | 前綴 | 對應條款 | 用途 | schema 來源 |
|---|---|---|---|---|
| `scope-exclusion-register.csv` | EXC | 4.3 / 6.1.3 d) | 排除項登錄（D10 四要素） | 新 |
| `system-depth-classification.csv` | SYS | 4.3 | D1/D2/D3 深度分級 | 新 |
| `requirements-register.csv` | REQ | 4.3 | 法律、契約、客戶、標準候選要求 | 新 |
| `asset-inventory.csv` | AST | A.5.9 | 資訊/服務/端點/雲端/身分/供應商/實體/**紙本紀錄**/行動裝置/儲存媒體 | 新 |
| `site-register.csv` | SITE | 4.3 | 據點/場所：門市、專櫃所在百貨、倉庫、辦公室、機房 | 新 |
| `data-flow-map.csv` | DFT | A.5.12/A.5.14/A.5.34 | 資料主體→蒐集→目的→系統→接收者→跨境→保存→刪除 | 新 |
| `cross-border-transfer-assessment.csv` | XBT | A.5.34 / 個資法 | 境外提供個人資料評估（D14） | 新 |
| `third-party-access-matrix.csv` | TPA | A.5.19–A.5.23/A.8.2 | 第三方/品牌方/供應商對我方系統之存取 | 新 |
| `ai-system-inventory.csv` | AIS | A.5.9 | AI 系統清冊 | **IMS 17** |
| `bia-service-continuity.csv` | SVC | A.5.29–A.5.30 | BIA、RTO/RPO、復原測試 | **IMS 19** |
| `context-interested-parties.csv` | CTX | 4.1 / 4.2 | 內外部議題、利害關係人、氣候相關性 | peer 版擴充 |
| `control-library.csv` | CTL | 6.1.3 | 93+ 組織附加控制 | 新 |
| `gap-assessment.csv` | GAP | M2 | 四段式缺口 | 新 |
| `crosswalk.csv` | XW | M1 | ISO27001↔CSF2↔CIS v8.1↔NIST 800-53↔SOC2↔PIPL | ADR-0011 |
| `control-self-assessment.csv` | CSA | 9.1 | 控制自評卡（4 維狀態） | peer 版擴充 |
| `access-review-findings.csv` | ARV | A.5.18 / A.5.3 | 複審發現與處置 | peer 版擴充 |
| `training-awareness-register.csv` | TRN | A.6.3 | 訓練與有效性 | 新 |
| `competence-matrix.csv` | CMP | A.6.3 | 角色×必要能力×控制 | 新 |
| `sod-limitation-register.csv` | SOD | A.5.3 | 職能分離限制與替代覆核 | 新 |
| `incident-register.csv` | INC | A.5.24–A.5.28 | 事件時間軸、分級、通報義務判定 | **IMS 20** |
| `supplier-register.csv` | VEN | A.5.19–A.5.23 | 供應商風險分級、條款、退出 | **IMS 18** |
| `governance-calendar.csv` | GC | 9.1/9.2/9.3 | 日/週/月/季/半年/年 治理節奏 | peer 版擴充 |
| `operational-calendar.csv` | OPL | 9.1 | 營運節奏與季節性（D16） | 新 |
| `metrics-register.csv` | KPI | 9.1 | 指標定義：公式、分子分母、基線、目標、核准 | 新 |

> **peer 檔名處理**：`access-review-findings` / `control-self-assessment` / `governance-calendar` / `context-interested-parties` 四檔由 peer 代理宣告建立。實作時**先列目錄比對**：磁碟上已存在則**擴充**（補 `process_area`、對齊 vocab、改 4 維狀態），不得覆寫。

### 7.3 每份 CSV 附 `-schema.md`

欄位定義、型別、允許值、必填性、驗證規則、值域來源（vocab）、合成示範（`EXAMPLE` 前綴）、引用限制、**IMS schema 對應**（若採 IMS 來源）。

---

## 8. 受控表單集（forms/，19 份）

IMS `21`–`35` PM 表單已存在；本套只做 ISO 27001 專屬之 19 份（CSV 單列空白表 ＋ `forms/README.md` 填寫規約）：

| ID | 表單 | 對應控制 | 關鍵欄位 |
|---|---|---|---|
| F-01 | ISMS 範圍聲明（含排除附錄） | 4.3 | 法人/場所/系統/流程、排除摘要、核准 |
| F-02 | 資訊安全政策核准 | A.5.1 | 版本、適用範圍、核准、發布、複審日 |
| F-03 | 權限申請單 | A.5.15/A.5.18 | 系統、資料範圍、權限類型、理由、期限、核准者 |
| F-04 | 權限授予/變更/撤銷作業單 | A.5.16/A.5.18 | 三段時間戳、執行者、驗證方式 |
| F-05 | 特權／緊急存取臨時授權單 | A.8.2 | 到期、事由、緊急作業後檢視 |
| F-06 | 進用/異動/離職權限作業單 | A.5.5/A.6.5 | 生效日、撤銷完成日、資產回收 |
| F-07 | 權限複審結論表 | A.5.18 | 母體範圍、保留/撤銷/降階、證據 |
| F-08 | 風險接受／例外核准單 | 6.1.3 f) | 理由、到期、重審、核准權限 |
| F-09 | 內部稽核報告 | 9.2 | 準則、範圍、母體、樣本理由、結論、發現 |
| F-10 | 管理評審會議紀錄 | 9.3 | 9.3.2 a)–g) 輸入、決議、資源、責任、期限 |
| F-11 | 資安事件通報單 | A.5.24–A.5.27 | 時間軸、分級、法定通報時限判定、通知截止與來源 |
| F-12 | 供應商資安評估表 | A.5.19–A.5.23 | 存取權限、DPA/條款、退出、複核日 |
| F-13 | 控制自評卡 | 9.1 | 4 維狀態、證據、限制、待辦 |
| F-14 | 證據收集清單 | 7.5.3 | 來源、期間、母體、完整性、保管、保留、時效窗 |
| F-15 | 存取權限差異表（登錄 vs 系統） | A.5.16/A.8.2 | 登錄側/系統側比對、差異、處置 |
| **F-16** | **據點與場所盤點表** | 4.3 / A.7.1 | 場所類型（門市/專櫃/倉庫/辦公室/機房）、管理主體、門禁、網路、實體負責角色 |
| **F-17** | **境外提供個人資料評估表** | A.5.34 | 資料欄位、接收國、法域、告知方式、同意狀態或施行細則第 26 條情形、再保險/轉任同意、重評時點 |
| **F-18** | **第三方存取核准單** | A.5.19–A.5.23/A.8.2 | 第三方類別（品牌方/百貨方/RPA 供應商/物流/雲端）、存取系統、資料範圍、期限、監控、撤銷條件 |
| **F-19** | **職能分離限制說明與替代覆核安排** | A.5.3 | 無法分離之控制、限制、替代覆核者、殘餘風險、到期 |

---

## 9. 系統軟體器具（tools/compliance/）

全部 **.ps1 主體僅 ASCII**；非 ASCII 置於 UTF-8 資料檔，以 `Get-Content -Encoding UTF8` 讀取（KB §6 規則 4）。

| 工具 | 用途 | 輸出 |
|---|---|---|
| `verify-compliance.ps1`（重寫） | 機械式符合性驗證 V-01..V-30 | `verification/verify-report-<ts>.md/.json` + log；exit 0/1/2 |
| `build-compliance-xlsx.ps1`（擴充） | CSV→XLSX（含 vocab 欄位標題） | `verification/xlsx/` + 雜湊日誌 |
| `new-isms-artifact.ps1` | 建立受控文件骨架（文件管制標頭、編號、版本） | 新檔 + 文件管制清單列 |
| `check-scope-consistency.ps1` | 範圍/排除/據點/流程/資產/控制/深度 六方一致性 | 差異報告 |
| `build-control-trace.ps1` | 控制→流程→風險→證據→測試 五向追溯矩陣 | `verification/control-trace-matrix.csv` |
| `build-access-review-packet.ps1` | 季度權限複審工作包（母體＋抽樣框＋待決清單） | `verification/access-review-<ts>/` |
| `collect-evidence.ps1` | 證據擷取、SHA-256、manifest（唯讀；**CR/LF 皆可**；不修改來源） | `evidence/EVD-*/` + manifest CSV |
| `build-workpackage-sheet.ps1` | WBS → 工作包列印表（A/R、DoD、依賴、關卡） | Markdown/CSV 表 |
| `check-personal-data-flow.ps1` | 資料流 × 個人資料 × 跨境 × 權限遮罩 交叉檢查 | 報告 |

### vocab 資料檔（11 檔，UTF-8 BOM）

`roles.csv`、`decisions.csv`、`control-status.csv`、`risk-levels.csv`、`treatment-options.csv`、
`verify-marks.csv`、`process-areas.csv`、`depth-tiers.csv`、**`subject-types.csv`**（含 `automation_identity`、`subject_employer_type`）、
**`access-types.csv`**、**`evidence-types.csv`**。

每檔欄位：`vocab_set,code,label_zh,label_en,notes`。驗證腳本以此為**唯一值域來源** → 值域變更不需改腳本（D4 核心）。

---

## 10. 資料流與追溯鏈

```
① 情境/利害關係人 (CTX) ─┐
② 需求/法規 (REQ) ───────┼─→ ⑤ SoA (93) ──→ ⑥ 控制庫 (CTL)
③ 資產 (AST) + 深度 (SYS) ┤      ↑                    │
④ 據點 (SITE) ────────────┘      │                    │
                                        │                    ▼
⑨ 第三方存取 (TPA) ──→ ⑧ 權限登錄 (AR) ──→ ⑩ 複審發現 (ARV)
                                        │              │
⑪ 資料流 (DFT) ──→ ⑫ 跨境評估 (XBT)      │              │
⑬ AI 清冊 (AIS) ────────────────────────┤              │
⑭ BIA (SVC) ────────────────────────────┤              ▼
⑮ 供應商 (VEN) ────────────────────────►│      ⑯ 證據 (EVD) ──→ ⑰ 測試 (TST)
⑱ 能力 (CMP)/職能分離限制 (SOD) ─────────┘              │
                                                          ▼
                                          ⑲ 不符合 (NCR) ──→ ⑳ 管理評審 (MR)
```

**不變量（驗證腳本逐項強制）**
1. SoA 的 93 個 `control_id` 為精確集合，無缺漏無重複。
2. 每個 `phase1_decision=適用` 之控制 → 至少被 1 個流程引用（V-08）。
3. 每個風險之 `linked_control_ids` → 已知且非「不適用」（V-09）。
4. 每個控制 → 至少 1 筆證據規劃（V-10）。
5. 每個排除項 → 理由＋影響＋補償控制＋證據＋核准＋日期齊備（V-06）。
6. 角色值必屬 9 種詞彙；`process_area` 必屬 11 種；`subject_type`／`access_type`／`subject_employer_type` 必屬 vocab。
7. 所有 CSV：UTF-8 BOM、欄數一致、主鍵唯一、表頭無空值。
8. 所有 Markdown：含「文件編號」「驗證標記」「狀態」「版本」「日期」管制標頭。
9. 資料流中含個人資料者 → 必須連結 `cross-border-transfer-assessment.csv` 之評估（V-22）。
10. 第三方存取矩陣之供應商皆存在於供應商登錄（V-27）。

---

## 11. 失敗模式與防護

| # | 失敗模式 | 徵兆 | 防護 |
|---|---|---|---|
| FM-1 | 範圍無限大無法執行 | 清單持續膨脹無收斂 | D1 深度分級 ＋ 排除四要素 ＋ `check-scope-consistency.ps1` |
| FM-2 | SoA 淪為勾選填表 | 理由空泛、證據欄空 | V-02、V-10 |
| FM-3 | 狀態跳級 | 直接填「已有效」 | 4 維獨立狀態 ＋ 禁止跳級（V-05、V-21） |
| FM-4 | 腳本與資料脫節 | 值域改了腳本沒改 | vocab 資料驅動（D4） |
| FM-5 | 權限複審形式化 | 100% 保留零撤銷 | 須產實際變更紀錄；撤銷率為觀察指標非目標 |
| FM-6 | 職能分離被破 | 同一人授予+核准+稽核 | V-09 交叉檢查 ＋ `sod-limitation-register.csv` |
| FM-7 | 證據散落／宣稱脫節 | 稽核找不到紀錄 | 證據索引 ＋ `collect-evidence.ps1` manifest ＋ 禁則宣稱掃描 |
| FM-8 | 個資／法規被過早結案 | REQ 直接填「不適用」 | D11 ＋ D14（跨境為最高優先且已有 ✔ 事實依據） |
| FM-9 | 雙套文件分歧 | KB 與 IMS 說法不同 | §1 優先規則 ＋ KNB-CMP-010 ＋ 交叉引用檢查 |
| FM-10 | 服務帳號被當人使用 | 權限登錄出現共用帳號 | D7 ＋ 服務帳號不得對應 `employee`/`contractor` |
| **FM-11** | **專櫃人員被誤登為我方員工** | access-register 出現品牌方員工 | D13 `subject_employer_type` ＋ 專櫃一律走 TPA（V-23） |
| **FM-12** | **自動化身分無法追溯至自然人** | 批次帳號無法對應執行人 | D15 `automation_identity` 四元關係（V-24） |
| **FM-13** | **跨境個資無合法基礎** | MTRIX 5 檔直送巴西無評估 | D14 ＋ F-17 ＋ `check-personal-data-flow.ps1`（V-22、V-29） |
| **FM-14** | **季節性偏誤** | 稽核抽樣全在淡季 | D16 營運行事曆 ＋ `seasonality_covered` 欄 |
| **FM-15** | **證據跨期重用** | 去年母體用於今年稽核 | `evidence_validity_window` ＋ 覆蓋期間必填 |
| **FM-16** | **SME 無法分離職能而無人登錄** | 同一自然人同時擔任多角 | F-19 ＋ `sod-limitation-register.csv` 強制登錄 |

---

## 12. 遷移／導入路徑

| 波次 | 內容 | 產出 | 驗收 |
|---|---|---|---|
| **W1 工具修復** | vocab 11 檔 ＋ `verify-compliance.ps1` 重寫 ＋ 舊欄位值域對齊 | 可執行驗證器 | 對現況產出基線報告 |
| **W2 範圍與結構** | 排除、深度分級、需求、據點、資產、情境、營運行事曆 | 範圍文件 ＋ 7 份新登錄 | 排除四要素 100%；流程 35 筆全有 owner 與控制 |
| **W3 資料流與法規** | 資料流、跨境評估、AI 清冊、BIA、第三方存取、供應商、事件 | 7 份新登錄 ＋ F-17/F-18 | 含個人資料之資料流 100% 有跨境評估連結 |
| **W4 控制與證據** | SoA 4 維改欄、控制庫、缺口評估、證據擴充、權限登錄重整、風險擴充 | 控制庫與證據模型 | 93/93 四維有值；每控制 ≥1 證據規劃 |
| **W5 文件與表單** | KNB-CMP-001..010 撰寫、forms 19 份 | 文件集 | 每 Markdown 含管制標頭；每 CSV 可解析 |
| **W6 整合與索引** | `Knowledge/README.md` v1.2、`Dev/Wellfit/docs/knowledge-index.md`、ADR-0012、KNB-CMP-010 | 索引一致 | 索引所指檔案皆存在且編號無重複 |
| **W7 總驗證** | 執行驗證器 ＋ XLSX 產出 | 報告 ＋ XLSX | FAIL=0；XLSX 可由 Excel 開啟且中文正常 |

---

## 13. 驗證計畫（V-01..V-30）

| ID | 檢查 | 層級 |
|---|---|---|
| V-01 | 必要檔案存在（主文件 10 ＋ 範本 32 ＋ 表單 19 ＋ vocab 11 ＋ 工具 9） | FAIL |
| V-02 | CSV：UTF-8 BOM、欄數一致、表頭非空、主鍵唯一 | FAIL |
| V-03 | Markdown：含「文件編號」「驗證標記」「狀態」「版本」「日期」 | FAIL |
| V-04 | SoA 93 項 `control_id` 精確集合（A.5.1–37 / A.6.1–8 / A.7.1–14 / A.8.1–34） | FAIL |
| V-05 | SoA 四維狀態值皆屬 vocab `control-status.csv` | FAIL |
| V-06 | 排除項四要素齊備（理由/影響/補償控制/證據/核准/日期） | FAIL |
| V-07 | 排除理由不得為「不需要」「成本過高」「無影響」 | FAIL |
| V-08 | 適用控制 → 至少 1 流程引用 | WARN |
| V-09 | 風險 → linked_control_ids 已知且非不適用 | FAIL |
| V-10 | 控制 → 至少 1 筆證據規劃 | WARN |
| V-11 | 角色值皆屬 9 種詞彙 | FAIL |
| V-12 | `process_area` 皆屬 11 種領域 | FAIL |
| V-13 | 深度級別合法；D1 系統須有年度測試規劃 | WARN |
| V-14 | 合成示範列一律含 `EXAMPLE` 標記 | FAIL |
| V-15 | 不得預填人名（角色欄不得出現非 vocab 值） | FAIL |
| V-16 | 禁則宣稱掃描（「已取得認證」「100% 符合」「符合 SOC 2」「已通過 TISAX」） | WARN |
| V-17 | 服務帳號不得標為 employee/contractor | FAIL |
| V-18 | 流程清單 ≥ 35 筆且既有 18 個 `process_id` 全部保留 | FAIL |
| V-19 | 交叉引用 ID 皆可解析（process_id / control_id / EVD-ID / WP-ID / G-ID / SITE-ID / VEN-ID） | WARN |
| V-20 | 表單 19 份皆可被 Import-Csv 解析且欄位數一致 | FAIL |
| **V-21** | **SoA 四維狀態不得跳級**（未設計不得為已實施；未實施不得為已有效） | FAIL |
| **V-22** | **含個人資料之資料流必連結跨境評估** | FAIL |
| **V-23** | **`access-register` 每列 `subject_employer_type` 合法；專櫃類不得標為我方員工** | FAIL |
| **V-24** | **`automation_identity` 列必具備執行人對應欄位且不得為空** | FAIL |
| **V-25** | **流程矩陣之 `site` 引用皆存在於據點清冊** | FAIL |
| **V-26** | **BIA 服務清單與流程矩陣交叉；D1 服務須有 BIA 列** | WARN |
| **V-27** | **第三方存取矩陣之供應商皆存在於供應商登錄** | FAIL |
| **V-28** | **營運行事曆之旺季標記與流程矩陣 `seasonality_peak` 一致** | WARN |
| **V-29** | **跨境評估之法定依據與告知方式欄位不得空白** | FAIL |
| **V-30** | **權限登錄之系統須存在於深度分級表；深度與複審頻率一致** | FAIL |

**執行**：`powershell -File tools/compliance/verify-compliance.ps1 -Root <compliance\iso-27001>`
退出碼：0=無 FAIL｜1=有 FAIL｜2=`-Strict` 下有 WARN。

---

## 14. 風險與未決事項

| # | 項目 | 影響 | 處置 |
|---|---|---|---|
| RK-1 | 全範圍導致 Phase 1 工作量失控 | 高 | D1 深度分級切分；G1 允許條件式放行但排除項與未評估項須逐項列示 |
| RK-2 | 專櫃／直營門市實際人力、POS、網路現況無記錄 | 高 | WP-01/03 產出據點與主體盤點；無法取得之母體記為 coverage limitation |
| RK-3 | 巴西跨境個人資料（D14）之合法基礎未確認 | **最高** | F-17 ＋ REQ-001；法務未決前不得擴大欄位或變更頻率 |
| RK-4 | 個資法 2025-11-11 修正部分條文未生效 | 中 | REQ 標註施行日待定與覆核人；不預判適用性 |
| RK-5 | ISO 授權文本未取得 | 中 | 條款引用標 ✔/△/未驗證；Phase 1 內取得授權版後複核（IMS P1-02） |
| RK-6 | 認證機構與顧問檔期未知 | 中 | 不承諾認證時程；G3 決策記錄為唯一對外依據 |
| RK-7 | peer 代理可能已寫入同路徑檔 | 低 | 實作前先列目錄，比對後**擴充**既有檔 |
| RK-8 | 自動化以人員桌面身分執行，權限歸屬模糊 | 高 | D15 四元關係登錄；A.8.2 特權邊界納入 WP-07 |
| RK-9 | SME 容量不足，無法同時滿足 93 控制 × 35 流程 | 高 | 深度分級 ＋ 外部稽核支援條款 ＋ F-19 職能分離限制 |

### 未決問題（需管理層／責任人回覆，非代理可自行決定）

| # | 問題 | 影響 | 決策者 | 阻擋 |
|---|---|---|---|---|
| Q1 | 風險接受門檻與風險胃納（可能性/影響尺度、升級門檻、接受權限） | 風險登錄與處置計畫無法定案 | 執行贊助人（WP-04） | 否 |
| Q2 | 9 種角色之具名人員與代理人 | 核准欄位無法填實 | 執行贊助人（WP-00） | 否 |
| Q3 | 直營門市數、專櫃數、倉庫數、現場人數、共用 POS 帳號數 | D1/D2 分級與權限母體規模 | 零售營運責任人（WP-03） | 否 |
| Q4 | 物流倉儲地點數與承運商數 | 物流資產與供應商母體 | 物流責任人（WP-03） | 否 |
| Q5 | **巴西跨境個人資料之告知/同意或施行細則第 26 條情形** | 專案三欄位與頻率能否維持 | 法務／法遵（WP-02 / F-17） | **是**（限 REQ-001） |
| Q6 | 支付卡是否進入我方環境（POS/電商）與 PCI 角色 | 支付類控制與 REQ | 支付業務責任人（WP-03A） | 否 |
| Q7 | 是否存在 AI（客服/推薦/文案/防詐） | AI 清冊與 ISO 42001 接軌 | AI 治理責任人（WP-03B） | 否 |
| Q8 | 認證目標時程與是否採外部顧問 | Phase 3 規劃 | 執行贊助人（WP-12 前） | 否 |
| Q9 | 內部稽核由內部、跨部門或外部支援 | 9.2.2 b 獨立性 | 執行贊助人（WP-10） | 否 |

> 以上 9 項**不阻擋**本計畫所列文件、範本、表單、工具與驗證工作；僅阻擋「填入組織事實」。唯一實質阻擋者為 Q5（且僅限 REQ-001 之結論）。

---

## 15. 執行順序（Implementation Task List）

| # | 任務 | 目標檔案 | 依賴 |
|---|---|---|---|
| T-01 | 建 `tools/compliance/vocab/` 11 份 vocab CSV（UTF-8 BOM） | vocab/*.csv | — |
| T-02 | 重寫 `verify-compliance.ps1`：資料驅動值域、繁中相容、V-01..V-30 | verify-compliance.ps1 | T-01 |
| T-03 | 執行驗證器取得基線報告並修正所有 FAIL | verification/ | T-02 |
| T-04 | 新增 `scope-exclusion-register.csv` + schema | templates/ | T-01 |
| T-05 | 新增 `system-depth-classification.csv` + schema | templates/ | T-01 |
| T-06 | 新增 `requirements-register.csv` + schema（REQ-001 跨境置頂） | templates/ | T-01 |
| T-07 | 新增 `site-register.csv` + schema | templates/ | T-01 |
| T-08 | 新增 `asset-inventory.csv` + schema（含紙本/行動/媒體） | templates/ | T-07 |
| T-09 | 新增 `data-flow-map.csv` + schema | templates/ | T-08 |
| T-10 | 新增 `cross-border-transfer-assessment.csv` + schema | templates/ | T-09 |
| T-11 | 新增 `third-party-access-matrix.csv` + schema | templates/ | T-01 |
| T-12 | 新增 `ai-system-inventory.csv`（採 IMS 17 schema）+ schema | templates/ | T-01 |
| T-13 | 新增 `bia-service-continuity.csv`（採 IMS 19 schema）+ schema | templates/ | T-08 |
| T-14 | 新增 `incident-register.csv`（採 IMS 20 schema）+ schema | templates/ | T-01 |
| T-15 | 新增 `supplier-register.csv`（採 IMS 18 schema）+ schema | templates/ | T-01 |
| T-16 | 新增 `competence-matrix.csv`、`sod-limitation-register.csv` + schema | templates/ | T-01 |
| T-17 | 新增 `operational-calendar.csv`、`metrics-register.csv` + schema | templates/ | T-01 |
| T-18 | 新增 `control-library.csv`（93+ 附加控制）+ schema | templates/ | T-22 |
| T-19 | 新增 `gap-assessment.csv` + schema | templates/ | T-18 |
| T-20 | 新增 `crosswalk.csv`（18 欄，ADR-0011） | templates/ | T-18 |
| T-21 | 改寫 `soa-93-controls.csv`：4 維狀態 + depth_tier + process_area + 理由欄 | templates/ | T-01 |
| T-22 | 重整 `access-register.csv`：subject/access 分離、`subject_employer_type`、`automation_identity`、MFA、倉庫維度、point-in-time | templates/ | T-01 |
| T-23 | 擴充 `risk-register.csv`：處置 4 選項、接受權限與到期；接入 F-06/F-07 真實初始條目 | templates/ | T-21, T-09 |
| T-24 | 擴充 `evidence-index.csv`：母體/期間/時效窗/完整性/保管/時區 | templates/ | T-21 |
| T-25 | 擴充 `nonconformity-register.csv`：10.2 四要素 | templates/ | T-01 |
| T-26 | 擴充 `internal-audit-program.csv`：G0–G7、獨立性、抽樣理由、季節性 | templates/ | T-01 |
| T-27 | 重建 `process-control-signoff-matrix.csv` 18→35 筆（ID 穩定、加 site/深度/季節性） | templates/ | T-07, T-21 |
| T-28 | 整併 peer 4 份登錄 ＋ schema（擴充，不覆寫） | templates/ | T-01 |
| T-29 | 重寫 `templates/README.md`（清單、值域|ID 命名、遷移、IMS schema 對應） | templates/README.md | T-04..T-28 |
| T-30 | 建立 `forms/` 19 份 CSV + `forms/README.md` | forms/ | T-01 |
| T-31 | 撰寫 KNB-CMP-007 範圍與排除（含 D1 深度模型、專櫃/門市雙軌） | 主文件 | T-04, T-05, T-07 |
| T-32 | 撰寫 KNB-CMP-008 控制庫方法 | 主文件 | T-18, T-19 |
| T-33 | 撰寫 KNB-CMP-009 實施路線圖（WG 表、WP 表、依賴圖、關卡對應） | 主文件 | T-27, T-18 |
| T-34 | 撰寫 KNB-CMP-010 IMS 對照與優先規則 | 主文件 | T-02 |
| T-35 | 改寫 KNB-CMP-001 v2.0（範圍、G0–G7、4 維狀態、七模組、25 項缺口對照） | 主文件 | T-31..T-34 |
| T-36 | 改寫 KNB-CMP-002 v1.1（automation_identity、專櫃/門市、倉庫維度、MFA 決策矩陣、break-glass） | 主文件 | T-22, T-11 |
| T-37 | 改寫 KNB-CMP-003 v1.1（4 處置、接受權限、深度分級、季節性） | 主文件 | T-23 |
| T-38 | 改寫 KNB-CMP-004 v1.1（G0–G7、獨立性、10.2 四要素、SME 職能分離限制） | 主文件 | T-25, T-16 |
| T-39 | 改寫 KNB-CMP-005 v1.1（vocab 化、雜湊 manifest、時效窗、保管與保留、CR/LF 相容） | 主文件 | T-24 |
| T-40 | 改寫 KNB-CMP-006 v1.1（接軌時序、跨境優先） | 主文件 | T-20, T-10 |
| T-41 | 撰寫 `adr/ADR-0012-isms-authority-precedence.md` | adr/ | T-34 |
| T-42 | 新增工具 7 支（new-isms-artifact / check-scope-consistency / build-control-trace / build-access-review-packet / collect-evidence / build-workpackage-sheet / check-personal-data-flow） | tools/ | T-01, T-02 |
| T-43 | 更新 `Knowledge/README.md` v1.2 | Knowledge/ | T-29..T-41 |
| T-44 | 更新 `Dev/Wellfit/docs/knowledge-index.md` | Dev/ | T-43 |
| T-45 | 執行驗證器 ＋ XLSX 產出，FAIL=0 收斂 | verification/ | T-02..T-44 |
| T-46 | 產出 `compliance/iso-27001/README.md` 總索引（文件管制清單） | compliance/ | T-45 |

---

## 16. 執行注意事項

1. **.ps1 一律 ASCII-only**；中文值域與標題置於 UTF-8 資料檔。
2. **CSV 一律 UTF-8 BOM**；中文敘述內使用全形逗號「，」避免與分隔符混淆。
3. **不得預填組織事實**（人名、真實日期、門檻值、系統數量、員工數）。
4. **不得複製受版權保護之 ISO Annex A 完整控制文字**。
5. **不得撰寫合規結論或認證宣稱**；一律以 ✔/△/未驗證 標記。
6. **既有 `Dev/Wellfit` 程式碼與 `Knowledge/` 其他子樹不得破壞性修改**。
7. **每完成一波次即執行一次 `verify-compliance.ps1`**，保持 FAIL=0 遞增。
8. **產物一經建立即不得回寫來源**；證據擷取須唯讀且支援 CR/LF 兩種換行（F-11）。

---

## 17. 人事時地物四維完整性稽核

本節記錄 v1.0 → v2.0 的缺口分析結果。每一項均已於本版獲得關閉機制。

### 17.1 人（People／組織與能力）

| # | 缺口 | 關閉機制 | 產物 |
|---|---|---|---|
| P-01 | SME 無法分離職能時無登錄機制（IMS 41 §2 要求記錄限制、替代覆核者與殘餘風險） | F-19 ＋ 強制登錄 | `sod-limitation-register.csv`、F-19、V-06 |
| P-02 | 角色×必要能力矩陣缺失，A.6.3 無法判定 | 能力矩陣連結控制 | `competence-matrix.csv` |
| P-03 | 內部稽核獨立性無替代機制（9.2.2 b / 9.2.3） | Q9 未決問題 ＋ WP-10 外部支援條款 | `internal-audit-program.csv.independence_check` |
| P-04 | 內部人風險與職務分離偵測無設計 | A.6.1 進用審查納入 WP-07 生命週期 | `access-register.csv` ＋ F-06 |
| P-05 | **百貨專櫃人員（品牌方人力）歸屬未定** → D13 | `subject_employer_type` ＋ 專櫃走 TPA | V-23、F-18 |
| P-06 | 關鍵人依賴（ISMS owner／ERP 管理員／CI 維護者休假） | 角色任命須含代理（WP-00） | 章程、KNB-CMP-009 |
| P-07 | 外部 RPA 供應商（百林商智）人員帳號未涵蓋 | 第三方存取矩陣 ＋ 供應商登錄 | `third-party-access-matrix.csv`、VEN |
| P-08 | 訓練「有效性」只記出席 | 訓練後測驗與觀察欄位 | `training-awareness-register.csv` |

### 17.2 時（Time／節奏與時效）

| # | 缺口 | 關閉機制 | 產物 |
|---|---|---|---|
| T-01 | **季節性偏誤**：零售旺季使母體與抽樣失真 | 營運行事曆 ＋ 稽核須標 seasonality_covered | `operational-calendar.csv`、V-28 |
| T-02 | 控制上線→可測試之觀察期未在 WBS 相依化 | WP-07→WP-10 明確相依；不足者標「待累積」 | KNB-CMP-009、IMS G5 規則 |
| T-03 | 證據跨期重用（去年母體用於今年稽核） | `evidence_validity_window` ＋ 覆蓋期間必填 | `evidence-index.csv`、V-15 |
| T-04 | 法規生效待辦（個資法部分條文未施行） | REQ 標施行日待定與覆核時點 | `requirements-register.csv` |
| T-05 | 週期性活動逾期無遞延機制 | 治理行事曆含「逾期處置／遞延核准」 | `governance-calendar.csv` |
| T-06 | 跨國時區（台灣作業 vs 巴西申報） | 證據 `timezone_basis` 必填；作業窗口定義 | `evidence-index.csv`、WP-07 |
| T-07 | 三波排程（08:30/10:00/17:00）與權限狀態之時間點還原 | 權限登錄 `point_in_time_snapshot_ref` | `access-register.csv` |
| T-08 | 排程型身分權限隨作業結束未回收 | 服務帳號到期日與作業綁定 | `access-register.csv`（D7） |

### 17.3 地（Place／據點與法域）

| # | 缺口 | 關閉機制 | 產物 |
|---|---|---|---|
| PL-01 | **據點／場所清冊不存在**（4.3 強制要求場所） | SITE 登錄；流程關聯據點 | `site-register.csv`、V-25 |
| PL-02 | 共管場所（百貨商場內專櫃）邊界與責任歸屬未定義 | SITE 記錄管理主體與責任分工 | `site-register.csv`、F-16 |
| PL-03 | **跨境資料流（台灣→巴西）無評估** | 跨境評估：告知/同意或施行細則第 26 條情形 | `cross-border-transfer-assessment.csv`、F-17、V-22、V-29 |
| PL-04 | 法域地圖與多國轉換法未盤點 | REQ 登錄含法域欄；IMS 47 §9 來源清單 | `requirements-register.csv` |
| PL-05 | 遠端／居家作業（A.6.7）未納入 | 權限登錄含裝置與位置欄；A.6.7 於 SoA | `access-register.csv` |
| PL-06 | 物流倉儲自有或 3PL 未定（影響場所歸屬） | Q4 未決；SITE 記錄營運主體 | `site-register.csv` |
| PL-07 | 門市網路分區與斷線作業（A.8.22）未評估 | 據點網段納入資產與架構盤點 | `asset-inventory.csv`、WP-03 |

### 17.4 物（Things／資產與資料）

| # | 缺口 | 關閉機制 | 產物 |
|---|---|---|---|
| TH-01 | **CCTV／影像監控未列為資產**（可識別個人 → 個資法適用；A.7.4） | AST 增 `surveillance_cameras` 類別 ＋ 個資標記 | `asset-inventory.csv` |
| TH-02 | 紙本紀錄（收據、簽核、合約）未涵蓋（A.5.33） | AST 增 `paper_record` 類別 ＋ 保存／銷毀 | `asset-inventory.csv` |
| TH-03 | 行動／自助裝置與 BYOD 未涵蓋（A.8.1） | AST 增 `mobile_device` 類別 | `asset-inventory.csv` |
| TH-04 | 儲存媒體與外接（USB、實體報表）未涵蓋（A.8.10/A.8.13） | AST 增 `removable_media` 類別 ＋ A.7.14 對應 | `asset-inventory.csv` |
| TH-05 | **本機匯出檔為主要外洩管道**（輔翼 ERP 桌面版匯出） | 資料流圖納入匯出節點 ＋ 檔案權限與保存 | `data-flow-map.csv` |
| TH-06 | **資料流圖不存在**（個資處理與跨境判定之必要輸入） | 資料流登錄（主體→蒐集→目的→系統→接收者→跨境→保存→刪除） | `data-flow-map.csv` |
| TH-07 | 支付卡資料流與 PCI 範圍未評估 | WP-03A 產出範圍假說；不得以「外包/tokenization」單獨斷言不適用 | WP-03A、REQ |
| TH-08 | AI 系統清冊不存在（零售 2026 幾必有 AI） | 採 IMS 17 schema；「未發現」須標明檢索來源與期間 | `ai-system-inventory.csv` |
| TH-09 | **自動化身分未登錄**（以人員桌面身分執行 → 無法追溯自然人） | D15 `automation_identity` 四元關係 | `access-register.csv`、V-24 |
| TH-10 | 測試／示範資料可能含真實個資 | 沿用既有「合成資料」慣例；測試環境不得含真實 PAN/SAD | 既有慣例 ＋ WP-06 |
| TH-11 | 資料生命週期（最小化、保留、刪除）未連結資產 | AST 增保留與處置欄位；A.8.10 對應 | `asset-inventory.csv` |

### 17.5 稽核結論

四維稽核共辨識 **35 項缺口**（人 8、時 8、地 7、物 11、跨維度 1），
其中 **19 項為 v1.0 計畫未涵蓋**（P-01、P-05、P-07、T-01、T-03、T-07、PL-01、PL-03、PL-05、TH-01、TH-02、TH-03、TH-04、TH-05、TH-06、TH-07、TH-08、TH-09、TH-11），
本版已全部納入 §7 新增產物、§8 新增表單、§13 新增檢查（V-21..V-30）與 §6 新增工作包（WP-03A、WP-03B）。
