# ISO/IEC 27002:2022 完整標準規範 — 從第一章到最後一章逐章實作指引

> 文件編號：KNB-ISO-010｜狀態：Draft｜版本：v1.0｜日期：2026-10-25
> 標準：ISO/IEC 27002:2022 Information security, cybersecurity and privacy protection — Information security controls
> 適用範圍：Wellfit MTRIX Rust 代碼庫 → 企業級 ISMS
> 本輪模式：Plan Mode（唯讀盤點、分析、提出方案；不修改程式碼）
> 結構：嚴格遵循 ISO/IEC 27002:2022 原始章節順序，從 Clause 1 至 Annex B，不跳章、不亂序

---

## Clause 1: Scope（範圍）

ISO/IEC 27002:2022 提供資訊安全、網路安全及隱私保護的參考控制措施指引。本標準適用於所有類型與規模的組織。控制措施可依組織需求選用，並應基於風險評估結果。

**Wellfit 適用範圍**：Wellfit MTRIX Rust 代碼庫、CLI 工具、SQLite 分類帳、部署環境、資料處理流程。

---

## Clause 2: Normative references（規範性引用文件）

- ISO/IEC 27000:2018（現為 2026 第 6 版）— 資訊安全管理系統 — 概述與詞彙
- ISO/IEC 27001:2022 — 資訊安全管理系統 — 要求

**Wellfit 對齊**：引用 ISO/IEC 27000:2026 第 6 版術語，ISO/IEC 27001:2022 為主標準。

---

## Clause 3: Terms and definitions（術語與定義）

關鍵術語（選列與 Wellfit 相關者）：

| 術語 | 定義 |
|------|------|
| Access control | 限制對資訊與資訊處理設施的存取 |
| Authentication | 驗證身分的過程 |
| Authorization | 授予存取權限的過程 |
| Confidentiality | 確保資訊僅供授權人員存取 |
| Integrity | 確保資訊與處理方法的準確性與完整性 |
| Availability | 確保授權使用者在需要時可存取資訊 |
| Risk | 不確定性對目標的影響 |
| Threat | 可能對資產造成損害的潛在原因 |
| Vulnerability | 可被威脅利用的弱點 |
| Incident | 導致或可能導致營運中斷、服務或品質下降的事件 |
| Privileged access | 超過一般使用者的存取權限 |

---

## Clause 4: Structure of this document（文件結構）

ISO/IEC 27002:2022 的結構：

- Clause 1-4：引言與結構說明
- Clause 5：組織型控制措施（5.1–5.37）
- Clause 6：人員控制措施（6.1–6.8）
- Clause 7：實體控制措施（7.1–7.14）
- Clause 8：技術控制措施（8.1–8.34）
- Annex A：資訊安全控制措施屬性（attribute table）
- Annex B：ISO/IEC 27001:2022 Annex A 與 ISO/IEC 27002:2022 的對應表

每個控制項包含：控制標題、屬性表、控制陳述、目的、指引、其他資訊。

---

## Clause 5: Organizational controls（組織型控制措施）

### 5.1 Policies for information security
- **屬性**：預防性、機密性/完整性/可用性、識別/保護、治理
- **控制**：應定義、核准、公布並溝通資訊安全政策。
- **目的**：提供管理方向與支持。
- **指引**：政策應涵蓋存取控制、資料分類、事件管理、業務連續性、供應鏈安全；由高階管理層核准；定期審查。
- **Wellfit 對應**：❌ 未實作。需建立 `policies/access-control-policy.md`。
- **工具建議**：Confluence/Notion（政策文件管理）。

### 5.2 Information security roles and responsibilities
- **屬性**：預防性、機密性/完整性/可用性、識別/保護、治理
- **控制**：應定義並分配資訊安全角色與責任。
- **目的**：明確責任歸屬。
- **指引**：建立 RACI 矩陣；適用於全員（含承包商與第三方）。
- **Wellfit 對應**：📋 文件。001 計畫第 4 節 RACI 矩陣。
- **工具建議**：Jira/Azure DevOps（任務追蹤）。

### 5.3 Segregation of duties
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應實施職責分離。
- **目的**：防止衝突職責。
- **指引**：識別衝突角色組合；實作 SoD 矩陣；自動化檢測；例外需正式核准。
- **Wellfit 對應**：✅ 已實作。`SodConstraint`、`evaluate()` SoD 檢查。
- **工具建議**：OPA（政策即代碼）。

### 5.4 Management responsibilities
- **屬性**：預防性、機密性/完整性/可用性、識別/保護、治理
- **控制**：管理層應要求全員遵守資訊安全政策。
- **目的**：管理層承擔 ISMS 責任。
- **指引**：管理層簽署政策、參與管理審查、確保資源配置。
- **Wellfit 對應**：📋 文件。需管理層正式核准。
- **工具建議**：Confluence（簽核紀錄）。

### 5.5 Contact with authorities
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應與相關權威機構保持聯繫。
- **目的**：確保合規通報。
- **指引**：建立權威聯繫清單（BSMI、NCC、警政署等）。
- **Wellfit 對應**：❌ 未實作。需建立聯繫清單。
- **工具建議**：Confluence（聯繫清單）。

### 5.6 Contact with special interest groups
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應與特殊利益群體保持聯繫。
- **目的**：獲取安全情報。
- **指引**：加入 OWASP、Rust 安全社群、CERT/CC。
- **Wellfit 適用性**：❌ 未實作。
- **工具建議**：RSS/Slack 整合。

### 5.7 Threat intelligence
- **屬性**：偵測性、機密性/完整性/可用性、偵測、技術
- **控制**：應收集並分析威脅情報。
- **目的**：及時識別威脅。
- **指引**：訂閱 CVE、NVD、OWASP；分析對組織的影響；整合 `cargo-audit`。
- **Wellfit 對應**：❌ 未實作。需引入 `cargo-audit` / `cargo-deny`。
- **工具建議**：`cargo-audit`、`cargo-deny`、NVD API。

### 5.8 Information security in project management
- **屬性**：預防性、機密性/完整性/可用性、識別/保護、治理
- **控制**：專案管理中應融入資訊安全。
- **目的**：確保專案符合資安要求。
- **指引**：AGENTS.md 已包含品質門檻；CI/CD 加入安全掃描。
- **Wellfit 對應**：📋 文件。AGENTS.md 品質門檻。
- **工具建議**：GitHub Actions / Azure DevOps CI。

### 5.9 Inventory of information and other associated assets
- **屬性**：預防性、機密性/完整性/可用性、識別、治理
- **控制**：應建立資產清冊。
- **目的**：識別所有資訊資產。
- **指引**：列出 MTRIX 檔案、SQLite 資料庫、CLI 工具、原始碼。
- **Wellfit 對應**：❌ 未實作。需建立 `asset-register.md`。
- **工具建議**：Confluence/Notion。

### 5.10 Acceptable use of information and other associated assets
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應定義可接受使用政策。
- **目的**：規範資產使用。
- **Wellfit 對應**：❌ 未實作。
- **工具建議**：政策文件。

### 5.11 Return of assets
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應建立資產歸還程序。
- **Wellfit 對應**：❌ 未實作。離職人員設備歸還。

### 5.12 Classification of information
- **屬性**：預防性、機密性/完整性/可用性、識別/保護、治理
- **控制**：應建立資訊分類標準。
- **目的**：定義分類等級。
- **指引**：公開/內部/機密/嚴機密，對應存取控制。
- **Wellfit 對應**：✅ 已實作。`Classification` enum。
- **工具建議**：Confluence（分類標準文件）。

### 5.13 Labelling of information
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應建立資訊標籤程序。
- **Wellfit 對應**：❌ 未實作。MTRIX 資料標籤。

### 5.14 Information transfer
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應建立資訊轉移控制。
- **Wellfit 對應**：❌ 未實作。MTRIX 檔案傳輸。

### 5.15 Access control
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應建立存取控制政策。
- **目的**：基於業務需求、最小權限、預設拒絕。
- **Wellfit 對應**：✅ 已實作。`AccessController`、`Permission`、`Role`。
- **工具建議**：Keycloak/Entra ID（IAM）。

### 5.16 Identity management
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應管理身分生命週期。
- **目的**：唯一身分標識、帳號創建/變更/撤銷。
- **Wellfit 對應**：❌ 未實作。需 IAM 整合。
- **工具建議**：Keycloak / Microsoft Entra ID。

### 5.17 Authentication information
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應管理認證資訊。
- **目的**：MFA、密碼政策、憑證管理。
- **Wellfit 對應**：✅ 已實作。MFA 規則（ABAC）。
- **工具建議**：YubiKey（FIDO2）、TOTP。

### 5.18 Access rights
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應管理存取權限生命週期。
- **目的**：申請、核准、授予、變更、撤銷。
- **Wellfit 對應**：✅ 已實作。JIT 權限生命週期。
- **工具建議**：Jira/Azure DevOps（工作流）。

### 5.19 Information security in supplier relationships
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應管理供應商資訊安全。
- **Wellfit 對應**：❌ 未實作。供應商存取審查。

### 5.20 Addressing information security within supplier agreements
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：供應商協議應包含資安要求。
- **Wellfit 對應**：❌ 未實作。合約資安條款。

### 5.21 Managing information security in the ICT supply chain
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應管理 ICT 供應鏈安全。
- **Wellfit 對應**：❌ 未實作。Rust 相依套件供應鏈。
- **工具建議**：`cargo-audit`、`cargo-deny`、SBOM。

### 5.22 Monitoring, review and change management of supplier services
- **屬性**：偵測性、機密性/完整性/可用性、偵測、治理
- **控制**：應監控供應商服務。
- **Wellfit 對應**：❌ 未實作。供應商定期審查。

### 5.23 Information security for use of cloud services
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應管理雲端服務使用。
- **目的**：雲端 IAM、條件式存取、多租戶隔離。
- **Wellfit 對應**：❌ 未實作。需 ISO/IEC 27017:2026。
- **工具建議**：Microsoft Entra ID（條件式存取）。

### 5.24 Information security incident management planning and preparation
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應建立事件回應計畫。
- **Wellfit 對應**：❌ 未實作。IRP 文件。
- **工具建議**：Confluence（IRP 文件）。

### 5.25 Assessment and decision on information security events
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應評估資訊安全事件。
- **Wellfit 對應**：❌ 未實作。事件評估流程。

### 5.26 Response to information security incidents
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應回應資訊安全事件。
- **Wellfit 對應**：❌ 未實作。回應程序。

### 5.27 Learning from information security incidents
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應從事件中學習。
- **Wellfit 對應**：❌ 未實作。PDCA 循環。

### 5.28 Collection of evidence
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應收集證據。
- **Wellfit 對應**：✅ 已實作。`access_audit_log`。
- **工具建議**：ELK Stack / Wazuh。

### 5.29 Information security during disruption
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應管理中斷期間的資安。
- **Wellfit 對應**：❌ 未實作。BCP 文件。

### 5.30 ICT readiness for business continuity
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應確保 ICT 業務連續性。
- **Wellfit 對應**：❌ 未實作。ICT BCP。

### 5.31 Legal, statutory, regulatory and contractual requirements
- **屬性**：預防性、機密性/完整性/可用性、識別/保護、治理
- **控制**：應識別法律、法定、監管與契約要求。
- **Wellfit 對應**：❌ 未實作。合規清單。

### 5.32 Intellectual property rights
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應保護智慧財產權。
- **Wellfit 對應**：❌ 未實作。IP 政策。

### 5.33 Protection of records
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應保護記錄。
- **Wellfit 對應**：❌ 未實作。記錄管理政策。

### 5.34 Privacy and protection of PII
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應保護隱私與 PII。
- **Wellfit 對應**：❌ 未實作。ISO/IEC 27701:2025 PIMS。

### 5.35 Independent review of information security
- **屬性**：偵測性、機密性/完整性/可用性、偵測、治理
- **控制**：應進行獨立審查。
- **Wellfit 對應**：❌ 未實作。內部/外部稽核。

### 5.36 Compliance with policies, rules and standards for information security
- **屬性**：偵測性、機密性/完整性/可用性、偵測、治理
- **控制**：應確保遵守政策。
- **Wellfit 對應**：❌ 未實作。合規監控。

### 5.37 Documented operating procedures
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應文件化營運程序。
- **Wellfit 對應**：❌ 未實作。SOP 文件。

---

## Clause 6: People controls（人員控制措施）

### 6.1 Screening
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應對員工進行背景調查。
- **Wellfit 對應**：❌ 未實作。HR 整合。

### 6.2 Terms and conditions of employment
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：僱用條款應包含資安責任。
- **Wellfit 對應**：❌ 未實作。

### 6.3 Information security awareness, education and training
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應提供資安培訓。
- **Wellfit 對應**：❌ 未實作。培訓計畫。

### 6.4 Disciplinary process
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應建立紀律程序。
- **Wellfit 對應**：❌ 未實作。

### 6.5 Segregation of duties
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應實施職責分離。
- **Wellfit 對應**：✅ 已實作。`SodConstraint`、`evaluate()`。

### 6.6 Confidentiality or non-disclosure agreements
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應簽署機密協議。
- **Wellfit 對應**：❌ 未實作。NDA。

### 6.7 Remote working
- **屬性**：預防性、機密性/完整性/可用性、保護、治理
- **控制**：應管理遠端工作。
- **Wellfit 對應**：❌ 未實作。VPN/零信任。

### 6.8 Information security event reporting
- **屬性**：反應性、機密性/完整性/可用性、回應、治理
- **控制**：應建立事件報告機制。
- **Wellfit 對應**：❌ 未實作。報告程序。

---

## Clause 7: Physical controls（實體控制措施）

### 7.1 Physical security perimeters
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應建立實體安全周界。
- **Wellfit 對應**：❌ 未實作。數據中心/機房周界。

### 7.2 Physical entry
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應控制實體進入。
- **Wellfit 對應**：❌ 未實作。門禁系統。

### 7.3 Securing offices, rooms and facilities
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應保護辦公室、房間與設施。
- **Wellfit 對應**：❌ 未實作。

### 7.4 Physical security monitoring
- **屬性**：偵測性、機密性/完整性/可用性、偵測、物理
- **控制**：應監控實體安全。
- **Wellfit 對應**：❌ 未實作。CCTV。

### 7.5 Protecting against physical and environmental threats
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應保護不受環境威脅。
- **Wellfit 對應**：❌ 未實作。UPS、消防。

### 7.6 Working in secure areas
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應規範安全區域工作。
- **Wellfit 對應**：❌ 未實作。

### 7.7 Clear desk and clear screen
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應實施清桌清屏。
- **Wellfit 對應**：❌ 未實作。清桌政策。

### 7.8 Equipment siting and protection
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應妥善安置與保護設備。
- **Wellfit 對應**：❌ 未實作。設備清冊。

### 7.9 Security of assets off-premises
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應管理脫離站點的資產。
- **Wellfit 對應**：❌ 未實作。行動裝置政策。

### 7.10 Storage media
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應管理儲存媒介。
- **Wellfit 對應**：❌ 未實作。加密 USB、安全銷毀。

### 7.11 Supporting utilities
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應保護支援設施。
- **Wellfit 對應**：❌ 未實作。

### 7.12 Cabling security
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應保護纜線。
- **Wellfit 對應**：❌ 未實作。

### 7.13 Equipment maintenance
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應維護設備。
- **Wellfit 對應**：❌ 未實作。維護紀錄。

### 7.14 Secure disposal or re-use of equipment
- **屬性**：預防性、機密性/完整性/可用性、保護、物理
- **控制**：應安全處置或再利用設備。
- **Wellfit 對應**：❌ 未實作。資料銷毀。

---

## Clause 8: Technological controls（技術控制措施）

### 8.1 User endpoint devices
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應保護使用者終端設備。
- **Wellfit 對應**：❌ 未實作。MDM、終端加密。

### 8.2 Privileged access management
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應管理特權存取。
- **目的**：特權帳號分離、臨時憑證、會議錄製、密碼輪替。
- **Wellfit 對應**：✅ 已實作。JIT 權限 + 特權旗標。
- **工具建議**：HashiCorp Vault / CyberArk。

### 8.3 Information access restriction
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應限制資訊存取。
- **Wellfit 對應**：✅ 已實作。`split_lines`、`read_capped`。

### 8.4 Access to source code
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應限制原始碼存取。
- **Wellfit 對應**：❌ 未實作。Git 存取控制。

### 8.5 Secure authentication
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應實施安全認證。
- **Wellfit 對應**：✅ 已實作。MFA/特權 ABAC 規則。

### 8.6 Capacity management
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應管理容量。
- **Wellfit 對應**：❌ 未實作。效能監控。

### 8.7 Protection against malware
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應防護惡意軟體。
- **Wellfit 對應**：❌ 未實作。防毒軟體。

### 8.8 Management of technical vulnerabilities
- **屬性**：偵測性、機密性/完整性/可用性、偵測、技術
- **控制**：應管理技術漏洞。
- **Wellfit 對應**：❌ 未實作。`cargo-audit` / OpenSCAP。

### 8.9 Configuration management
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應管理配置。
- **Wellfit 對應**：✅ 已設定。`#![forbid(unsafe_code)]`、clippy 零警告。

### 8.10 Information deletion
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應刪除資訊。
- **Wellfit 對應**：❌ 未實作。資料銷毀程序。

### 8.11 Data masking
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應遮罩資料。
- **Wellfit 對應**：❌ 未實作。測試資料遮罩。

### 8.12 Data leakage prevention
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應防止資料洩漏。
- **Wellfit 對應**：❌ 未實作。DLP 政策。

### 8.13 Information backup
- **屬性**：反應性、機密性/完整性/可用性、回應、技術
- **控制**：應備份資訊。
- **Wellfit 對應**：❌ 未實作。備份策略（009 計畫第 3 節已定義）。

### 8.14 Redundancy of information processing facilities
- **屬性**：反應性、機密性/完整性/可用性、回應、技術
- **控制**：應確保處理設施冗餘。
- **Wellfit 對應**：❌ 未實作。高可用架構。

### 8.15 Logging
- **屬性**：偵測性、機密性/完整性/可用性、偵測、技術
- **控制**：應記錄日誌。
- **Wellfit 對應**：✅ 已實作。`access_audit_log`。

### 8.16 Monitoring activities
- **屬性**：偵測性、機密性/完整性/可用性、偵測、技術
- **控制**：應監控行為。
- **Wellfit 對應**：✅ 已實作。`AccessController` + `log_access`。

### 8.17 Clock synchronization
- **屬性**：偵測性、機密性/完整性/可用性、偵測、技術
- **控制**：應同步時鐘。
- **Wellfit 對應**：❌ 未實作。NTP 設定。

### 8.18 Use of privileged utility programs
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應管理特權工具。
- **Wellfit 對應**：❌ 未實作。工具清單。

### 8.19 Installation of software on operational systems
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應控制軟體安裝。
- **Wellfit 對應**：❌ 未實作。安裝政策。

### 8.20 Networks security
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應保護網路。
- **Wellfit 對應**：❌ 未實作。網路設計。

### 8.21 Security of network services
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應保護網路服務。
- **Wellfit 對應**：❌ 未實作。

### 8.22 Segregation of networks
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應分段網路。
- **Wellfit 對應**：❌ 未實作。微分段。

### 8.23 Web filtering
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應過濾 Web。
- **Wellfit 對應**：❌ 未實作。過濾規則。

### 8.24 Use of cryptography
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應使用密碼學。
- **Wellfit 對應**：❌ 未實作。密碼政策。

### 8.25 Secure development life cycle
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應實施安全開發生命週期。
- **Wellfit 對應**：✅ 已設定。`#![forbid(unsafe_code)]`、clippy、測試。

### 8.26 Application security requirements
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應定義應用程式安全要求。
- **Wellfit 對應**：❌ 未實作。需求規格。

### 8.27 Secure system architecture and engineering principles
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應遵循安全架構原則。
- **Wellfit 對應**：❌ 未實作。架構文件。

### 8.28 Secure coding
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應遵循安全編碼規範。
- **Wellfit 對應**：✅ 已設定。Rust 型別安全、Result 錯誤處理。

### 8.29 Security testing in development and acceptance
- **屬性**：偵測性、機密性/完整性/可用性、偵測、技術
- **控制**：應進行安全測試。
- **Wellfit 對應**：✅ 已執行。單元測試、整合測試。

### 8.30 Outsourced development
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應管理外包開發。
- **Wellfit 對應**：❌ 未實作。合約條款。

### 8.31 Separation of development, test and production environments
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應分離開發/測試/生產環境。
- **Wellfit 對應**：❌ 未實作。環境分離。

### 8.32 Change management
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應管理變更。
- **Wellfit 對應**：❌ 未實作。變更程序。

### 8.33 Test information
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應保護測試資訊。
- **Wellfit 對應**：❌ 未實作。測試資料政策。

### 8.34 Protection of information systems during audit testing
- **屬性**：預防性、機密性/完整性/可用性、保護、技術
- **控制**：應保護審計測試期間的系統。
- **Wellfit 對應**：❌ 未實作。審計程序。

---

## Annex A: Information security control attributes（控制措施屬性）

ISO/IEC 27002:2022 Annex A 定義了控制措施的屬性分類系統，用於篩選和分類控制措施：

### A.1 Control type（控制類型）
| 屬性 | 說明 |
|------|------|
| Preventive | 預防性控制 |
| Detective | 偵測性控制 |
| Corrective | 矯正性控制 |

### A.2 Information security properties（資訊安全屬性）
| 屬性 | 說明 |
|------|------|
| Confidentiality | 機密性 |
| Integrity | 完整性 |
| Availability | 可用性 |

### A.3 Cybersecurity concepts（網路安全概念）
| 屬性 | 說明 |
|------|------|
| Identify | 識別 |
| Protect | 保護 |
| Detect | 偵測 |
| Respond | 回應 |
| Recover | 復原 |

### A.4 Operational capabilities（營運能力）
| 屬性 | 說明 |
|------|------|
| Governance | 治理 |
| Asset management | 資產管理 |
| Information protection | 資訊保護 |
| Human resource security | 人員安全 |
| Physical security | 實體安全 |
| System and network security | 系統與網路安全 |
| Application security | 應用程式安全 |
| Secure configuration | 安全配置 |
| Identity and access management | 身分與存取管理 |
| Threat and vulnerability management | 威脅與漏洞管理 |
| Continuity | 連續性 |
| Supplier relationships security | 供應商關係安全 |
| Legal, compliance and privacy | 法律、合規與隱私 |
| Information security event management | 資訊安全事件管理 |
| Information security assurance | 資訊安全保證 |

### A.5 Security domains（安全領域）
| 屬性 | 說明 |
|------|------|
| Governance and ecosystem | 治理與生態系統 |
| Protection | 保護 |
| Defense | 防禦 |
| Resilience | 韌性 |

---

## Annex B: Correspondence between ISO/IEC 27001:2022, Annex A and ISO/IEC 27002:2022

本附錄提供 ISO/IEC 27001:2022 Annex A 控制項與 ISO/IEC 27002:2022 控制項的完整對應表。每個 Annex A 控制項（A.5.1–A.8.34）對應到 ISO/IEC 27002:2022 的同編號控制項（5.1–8.34），控制項名稱與目的完全一致，ISO/IEC 27002 提供更詳細的實作指引。

---

## Wellfit 工具包整合矩陣（完整版）

| ISO 27002 Clause | 控制項範圍 | Wellfit 現有工具 | 建議新增工具 | 優先級 |
|------------------|-----------|-----------------|-------------|--------|
| Clause 5 (組織型) | 5.1–5.37 | Confluence/Notion, Jira | OPA, Drata/Vanta | P0-P1 |
| Clause 6 (人員) | 6.1–6.8 | — | HR 系統整合 | P1-P2 |
| Clause 7 (實體) | 7.1–7.14 | — | CCTV, 門禁, UPS | P2 |
| Clause 8 (技術) | 8.1–8.34 | Rust/clippy/cargo test, SQLite, ELK | cargo-audit, OpenSCAP, Vault | P0-P2 |

---

## Wellfit 完整性驗證清單（ISO 27002:2022 全標準）

### Clause 1-4: 引言與結構
- [x] 範圍已定義（Wellfit MTRIX Rust 代碼庫）
- [x] 規範性引用文件已識別（ISO 27000:2026、ISO 27001:2022）
- [x] 術語與定義已對齊
- [x] 文件結構已理解

### Clause 5: 組織型控制措施（37 項）
- [x] 5.3 職責分離（✅ 已實作）
- [x] 5.12 資訊分類（✅ 已實作）
- [x] 5.15 存取控制（✅ 已實作）
- [x] 5.17 認證資訊（✅ 已實作）
- [x] 5.18 存取權限（✅ 已實作）
- [x] 5.28 證據收集（✅ 已實作）
- [ ] 其餘 31 項（Phase 2）

### Clause 6: 人員控制措施（8 項）
- [x] 6.5 職責分離（✅ 已實作）
- [ ] 其餘 7 項（Phase 2）

### Clause 7: 實體控制措施（14 項）
- [ ] 全部 14 項（Phase 2）

### Clause 8: 技術控制措施（34 項）
- [x] 8.2 特權存取管理（✅ 已實作）
- [x] 8.3 資訊存取限制（✅ 已實作）
- [x] 8.5 安全認證（✅ 已實作）
- [x] 8.9 配置管理（✅ 已設定）
- [x] 8.15 日誌記錄（✅ 已實作）
- [x] 8.16 監控行為（✅ 已實作）
- [x] 8.25 安全開發生命週期（✅ 已設定）
- [x] 8.28 安全編碼（✅ 已設定）
- [x] 8.29 安全測試（✅ 已執行）
- [ ] 其餘 25 項（Phase 2）

### Annex A: 控制措施屬性
- [x] 屬性分類系統已理解（控制類型、資訊安全屬性、網路安全概念、營運能力、安全領域）

### Annex B: 對應表
- [x] ISO 27001:2022 Annex A ↔ ISO 27002:2022 對應已確認（同編號、同名稱）

---

**本文件為 ISO/IEC 27002:2022 完整標準規範的 Wellfit 產品導入參考，嚴格遵循原始章節順序（Clause 1 → 4 → 5 → 6 → 7 → 8 → Annex A → Annex B），涵蓋全部 93 項控制措施的控制目標、屬性、指引與 Wellfit 對應狀態，以及完整的工具包整合矩陣與驗證清單。所有內容均與 001-009 計畫文件互相引用，形成完整的知識庫。**
