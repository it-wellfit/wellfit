# 知識庫索引（Knowledge index）

> 文件編號：WFLT-DOC-KNW-001｜狀態：Draft｜版本：v1.2｜日期：2026-10-22
> 撰寫：Wellfit 知識庫維護者｜審閱：未審閱｜核准：未核准
> 變更摘要：v1.2。同步知識庫合規子樹之全面盤點與操作程序文件新增：主文件 6→10 份、範本 7→23 份＋23 份欄位說明、
> 新增 15 份受控表單、值域資料檔（`tools/compliance/vocab/`）與 8 支合規工具；新增 ADR-0012。
> v1.0。初版。索引 [`../../Knowledge/`](../../../Knowledge/) 之全部文件，並標明哪些是「唯一真相來源」。
> 驗證標記：✔ 已對原始文件逐字查證｜△ 依標準條文或通行做法推導｜未驗證 本次無法確認

## 0. 知識庫在哪裡

| 項目 | 內容 |
|---|---|
| 路徑 | [`../../Knowledge/`](../../../Knowledge/)（`2026/Knowledge`） |
| 性質 | **跨專案**共享，不屬於 `Dev/Wellfit`；哈瓦士串接、蝦皮與 ERP 自動化、RPA 導入、國際標準稽核共用 |
| 治理規則 | [`../../Knowledge/README.md`](../../../Knowledge/README.md) |
| 為何獨立 | 規格屬於「跨專案的介接事實」，不應綁死在單一實作專案的版控裡（`adr/ADR-0004`） |
| 引用方向 | **單向**：`Dev/Wellfit` 引用 `Knowledge`；`Knowledge` 不引用任何專案的實作程式碼作為契約事實 |

## 1. 快速入口（依「我要找什麼」）

| 我想知道 | 去這裡 |
|---|---|
| 這份契約從哪來？ | [`sources/source-register.md`](../../../Knowledge/sources/source-register.md) |
| 某個欄位到底是什麼？ | [`reconciliation/field-diff.md`](../../../Knowledge/reconciliation/field-diff.md)（56 欄七欄對照） |
| 56 欄的機讀版 | [`specs/mtrix-field-dictionary.md`](../../../Knowledge/specs/mtrix-field-dictionary.md)（程式碼生成） |
| 驗證規則怎麼運作？ | [`specs/mtrix-vr-rules.md`](../../../Knowledge/specs/mtrix-vr-rules.md) |
| 引擎會改我的資料嗎？ | [`specs/mtrix-anomaly-catalogue.md`](../../../Knowledge/specs/mtrix-anomaly-catalogue.md) |
| 蝦皮那邊到底驗了什麼？ | [`specs/shopee-contract.md`](../../../Knowledge/specs/shopee-contract.md) |
| 輔翼 ERP 那邊呢？ | [`specs/erp-report-contract.md`](../../../Knowledge/specs/erp-report-contract.md) |
| 還有哪些沒解決？ | [`issues/mtx-issue-register.md`](../../../Knowledge/issues/mtx-issue-register.md)（87 項） |
| 這個數字有來源嗎？ | [`traceability/claims-vs-evidence.md`](../../../Knowledge/traceability/claims-vs-evidence.md) |
| 欄位對應到哪段程式碼？ | [`traceability/field-to-code.md`](../../../Knowledge/traceability/field-to-code.md) |
| 哪條規則由哪個測試覆蓋？ | [`traceability/vr-to-test.md`](../../../Knowledge/traceability/vr-to-test.md) |
| UAT 跑到哪了？ | [`uat/UAT-REPORT.md`](../../../Knowledge/uat/UAT-REPORT.md) |
| 測試資料從哪來？ | [`data/provenance.md`](../../../Knowledge/data/provenance.md) |
| 每天怎麼跑？ | [`ops/RUNBOOK.md`](../../../Knowledge/ops/RUNBOOK.md) |
| 示場怎麼走？ | [`ops/DEMO.md`](../../../Knowledge/ops/DEMO.md) |
| 命令怎麼用？ | [`ops/CLI.md`](../../../Knowledge/ops/CLI.md) |
| 為什麼是這樣決定？ | [`adr/`](../../../Knowledge/adr/) |
| 上網查了什麼？ | [`sources/web-research.md`](../../../Knowledge/sources/web-research.md) |
| 國際標準稽核（ISO 27001）怎麼做？ | [`compliance/iso-27001/`](../../../Knowledge/compliance/iso-27001/)（總體計畫、權限管理、風險評估、稽核與簽核、證據模型、多標準接軌、範圍與排除、控制庫方法、實施路線圖、IMS 權威對照） |
| 合規套件的總索引在哪？ | [`compliance/iso-27001/README.md`](../../../Knowledge/compliance/iso-27001/README.md) |
| SoA／風險／權限／稽核範本在哪？ | [`compliance/iso-27001/templates/`](../../../Knowledge/compliance/iso-27001/templates/)（23 份 CSV ＋ 23 份 `-schema.md` ＋ 填寫規約） |
| 受控表單在哪？ | [`compliance/iso-27001/forms/`](../../../Knowledge/compliance/iso-27001/forms/)（F-01～F-15） |
| 值域（角色、狀態、領域、深度級別…）從哪來？ | [`tools/compliance/vocab/`](../../../Knowledge/tools/compliance/vocab/)（11 份 CSV；**唯一值域來源**，值域變更不需改腳本） |
| 合規產物有沒有機械驗證？ | `powershell -ExecutionPolicy Bypass -File ..\..\Knowledge\tools\compliance\verify-compliance.ps1`（V-01～V-20；報告在 `compliance/iso-27001/verification/`） |
| 知識庫與 IMS 套件誰是權威？ | [`adr/ADR-0012`](../../../Knowledge/adr/ADR-0012-isms-authority-precedence.md) ＋ [`ims-reconciliation-map.md`](../../../Knowledge/compliance/iso-27001/ims-reconciliation-map.md) |
| 規則可以怎麼用？ | [`sources/research-protocol.md`](../../../Knowledge/sources/research-protocol.md) |

## 2. 目錄結構

```
2026/Knowledge/
├── README.md                      治理規則、目錄地圖、命名規則、驗證標記
├── adr/                           決策紀錄（KNB-ADR-001..012）
│   ├── ADR-0001-source-hierarchy-and-conflict-arbitration.md
│   ├── ADR-0002-per-file-rejection-partial-acceptance.md
│   ├── ADR-0003-fixture-provenance-strategy.md
│   ├── ADR-0004-knowledge-base-location.md
│   ├── ADR-0005-mandatory-columns-fail-hard.md
│   ├── ADR-0006-fixed-point-decimal-arithmetic.md
│   ├── ADR-0007-unverifiable-sources-are-marked.md
│   ├── ADR-0008-work-continuation-model.md          ← 工作延續模型
│   ├── ADR-0009-spec-derivation-vs-arbitration-separation.md
│   ├── ADR-0010-iso-27001-isms-baseline-single-control-library.md   ← ISMS 單一控制庫
│   ├── ADR-0011-compliance-naming-and-crosswalk-schema.md          ← 合規命名與對照表 schema
│   └── ADR-0012-isms-authority-precedence.md                        ← 與 IMS 套件之權威分工
├── sources/
│   ├── source-register.json        權威機讀登錄（9 份來源）
│   ├── source-register.md          人可讀檢視（含附錄 A/B）
│   ├── research-protocol.md        研究規約（上網邊界）
│   ├── web-research.md             工程研究落地紀錄
│   └── snapshots/<doc-id>/         不可變快照（text.md / pages.json / extract.log）
├── specs/
│   ├── mtrix-field-dictionary.md   56 欄欄位字典（程式碼生成）
│   ├── mtrix-vr-rules.md           VR-01..VR-14 目錄
│   ├── mtrix-anomaly-catalogue.md  ANOM-01..ANOM-12 目錄
│   ├── shopee-contract.md          蝦皮契約與缺口
│   └── erp-report-contract.md      輔翼 ERP 契約與缺口
├── reconciliation/
│   ├── field-diff.md               56 欄對齊與裁決表（程式碼生成）
│   └── layer-evidence.json         L1／L2／L3 逐欄證據（人工維護）
├── issues/mtx-issue-register.md    87 項議題（MTRIX-001..030、ISS-001..031、LOCAL-001..026）
├── traceability/
│   ├── field-to-code.md            欄位 → 程式碼 → 規則 → 測試
│   ├── vr-to-test.md               規則 → 測試
│   └── claims-vs-evidence.md       宣稱 vs 證據（含撤回清單）
├── uat/UAT-REPORT.md               UAT-01..07 執行結果
├── data/provenance.md              fixture 溯源與雜湊
├── ops/
│   ├── RUNBOOK.md                  每日維運
│   ├── DEMO.md                     示場流程
│   └── CLI.md                      命令參考
├── tools/
│   ├── dev-env/build.ps1           可重現建置入口
│   ├── spec-extract/               抽取管線與欄位字典產生器
│   ├── reconciliation/             對照表產生器
│   └── compliance/                 合規工具（8 支 .ps1，主體 ASCII only）
│       ├── verify-compliance.ps1   機械式驗證器（V-01～V-20，資料驅動值域）
│       ├── build-compliance-xlsx.ps1
│       ├── new-isms-artifact.ps1   受控文件骨架與文件管制清單
│       ├── check-scope-consistency.ps1
│       ├── build-control-trace.ps1
│       ├── build-access-review-packet.ps1
│       ├── collect-evidence.ps1
│       ├── build-workpackage-sheet.ps1
│       └── vocab/                  值域資料檔（11 份 CSV，唯一值域來源）
└── compliance/iso-27001/           ISO/IEC 27001:2022 ISMS 導入（跨專案）
    ├── README.md                            套件總索引
    ├── iso-27001-master-plan.md              KNB-CMP-001 總體演進計畫
    ├── iso-27001-access-management-sop.md    KNB-CMP-002 權限管理實務指南
    ├── iso-27001-risk-assessment-method.md   KNB-CMP-003 風險評估與 SoA 規則
    ├── iso-27001-audit-signoff-playbook.md   KNB-CMP-004 稽核與簽核手冊
    ├── iso-27001-evidence-and-document-control.md  KNB-CMP-005 證據與文件管制
    ├── multi-standard-evolution-roadmap.md    KNB-CMP-006 多標準接軌
    ├── iso-27001-scope-and-exclusions.md     KNB-CMP-007 範圍與排除規則
    ├── iso-27001-control-library-method.md   KNB-CMP-008 控制庫方法
    ├── iso-27001-implementation-roadmap.md   KNB-CMP-009 實施路線圖
    ├── ims-reconciliation-map.md             KNB-CMP-010 與 IMS 套件之權威對照
    ├── templates/                   23 份 CSV ＋ 23 份 -schema.md ＋ README.md
    ├── forms/                       15 份受控表單 ＋ README.md
    └── verification/                 驗證報告、JSON 報告與 SHA-256 雜湊清單
```

## 3. 來源快照索引

| doc-id | 層級 | 快照 |
|---|---|---|
| `manual-alpargatas` | L1 原始規格 | [`text.md`](../../../Knowledge/sources/snapshots/manual-alpargatas/text.md) |
| `mtrix-bi-spec-v2` | L2 內部推薦版 | [`text.md`](../../../Knowledge/sources/snapshots/mtrix-bi-spec-v2/text.md) |
| `mtrix-field-dict-v2` | L2 內部推薦版 | **unavailable**（抽取失敗，見來源登錄 §3） |
| `mtrix-mapping-plan` | L2 內部推薦版 | **unavailable**（抽取失敗，見來源登錄 §3） |
| `alpargatas-v6-docx` | L3 近期整合版 | [`text.md`](../../../Knowledge/sources/snapshots/alpargatas-v6-docx/text.md) |
| `alpargatas-v6-pdf` | L3 近期整合版 | [`text.md`](../../../Knowledge/sources/snapshots/alpargatas-v6-pdf/text.md) |
| `shopee-requirements-v10` | L2 內部推薦版 | [`text.md`](../../../Knowledge/sources/snapshots/shopee-requirements-v10/text.md) |
| `shopee-requirements-v21` | L2 內部推薦版 | [`text.md`](../../../Knowledge/sources/snapshots/shopee-requirements-v21/text.md) |
| `ims-template-set` | aux 輔助 | [`text.md`](../../../Knowledge/sources/snapshots/ims-template-set/text.md) |

> 快照內的 `[pN]` 是 **Word 重排頁碼**，與原 PDF **印刷頁碼**不同。
> 引用時必須註明頁碼種類（詳見 `reconciliation/field-diff.md` §4）。

## 4. 決策紀錄索引

| ADR | 主題 | 對本專案的影響 |
|---|---|---|
| [ADR-0001](../../../Knowledge/adr/ADR-0001-source-hierarchy-and-conflict-arbitration.md) | 來源層級與衝突裁決 | 語意採 L1、介接採交付位元組、不一致登議題 |
| [ADR-0002](../../../Knowledge/adr/ADR-0002-per-file-rejection-partial-acceptance.md) | 逐檔拒絕與部分接受 | 一檔壞不影響其餘四檔 |
| [ADR-0003](../../../Knowledge/adr/ADR-0003-fixture-provenance-strategy.md) | Fixture 溯源策略 | 真實／衍生／合成三分，不得混用 |
| [ADR-0004](../../../Knowledge/adr/ADR-0004-knowledge-base-location.md) | 知識庫位置 | 跨專案 `2026/Knowledge` |
| [ADR-0005](../../../Knowledge/adr/ADR-0005-mandatory-columns-fail-hard.md) | 必填欄硬性失敗、無序位退讓 | `wellfit_core::contract` |
| [ADR-0006](../../../Knowledge/adr/ADR-0006-fixed-point-decimal-arithmetic.md) | 定點十進位、零浮點 | `wellfit_core::scaled`、蝦皮整數分 |
| [ADR-0007](../../../Knowledge/adr/ADR-0007-unverifiable-sources-are-marked.md) | 不可驗證一律標記 | `VerifyStatus` 五態 |
| [ADR-0008](../../../Knowledge/adr/ADR-0008-work-continuation-model.md) | 工作延續模型 | 本次工作的目標與邊界 |
| [ADR-0009](../../../Knowledge/adr/ADR-0009-spec-derivation-vs-arbitration-separation.md) | 規格推導與裁決分離 | 禁止以程式碼反推原文 |
| [ADR-0010](../../../Knowledge/adr/ADR-0010-iso-27001-isms-baseline-single-control-library.md) | ISO 27001 為 ISMS 基線 | 單一控制庫、單一 SoA、單一證據索引 |
| [ADR-0011](../../../Knowledge/adr/ADR-0011-compliance-naming-and-crosswalk-schema.md) | 合規命名與對照表 schema | 檔名例外、crosswalk 與 SoA 欄位一對一映射 |
| [ADR-0012](../../../Knowledge/adr/ADR-0012-isms-authority-precedence.md) | 與 IMS 套件之權威分工 | 關卡／角色／WBS 以 IMS 為權威；控制編號／欄位／值域／證據以合規套件為權威；不得複本 |

## 5. 維護規則（摘要）

1. **文件管制**：檔名 `KNB-<類別>-<序號>_<名稱>_v<主>.<次>`；狀態限 `Draft`／`In Review`／`Approved`／`Superseded`；
   標頭含文件編號、狀態、版本、日期、作者、審閱、核准、變更摘要與驗證標記。
2. **驗證標記**：✔ 已對原始文件逐字查證｜△ 依標準條文或通行做法推導｜未驗證 本次無法確認。
3. **不可追溯的數字不得寫入**（ADR-0007）。
4. **產生的文件不得手改**：改 `spec.rs` 或 `layer-evidence.json` 後重跑產生腳本。
5. **快照不可變**：重新抽取產生新目錄並更新來源登錄。
6. **抽取腳本之 `.ps1` 在 Windows PowerShell 5.1 下不得含非 ASCII 字面量**；
   非 ASCII 內容置於 UTF-8 模板／資料檔，由腳本以顯式編碼讀取。
7. **合規產物變更後必須通過機械驗證**：
   `powershell -ExecutionPolicy Bypass -File ..\..\Knowledge\tools\compliance\verify-compliance.ps1`
   須為 FAIL=0；值域變更只動 `Knowledge/tools/compliance/vocab/`，**不得**為了新增值域去改驗證腳本。
8. **不預填未知事實**：門市數、櫃位數、員工數、系統數、權限母體數、風險門檻與具名人員一律填
   「待確認／待指派／待核准」；法律適用性未取得正式來源前一律「待判定」。
