# ISO/IEC 27001:2022 工具包與資源參考彙整（子計畫 003）

> 文件編號：KNB-ISO-003｜狀態：Draft｜版本：v1.0｜日期：2026-10-25
> 主計畫：001-iso27001-access-control-implementation-plan.md

---

## 1. 文件目的

彙整 ISO/IEC 27001:2022 權限管理導入所需的完整工具包，包括：

- 身分與存取管理（IAM）
- 特權存取管理（PAM）
- 認證與多因子驗證（MFA）
- 權限政策與職責分離（SoD）
- 權限審計與監控（SIEM/日誌）
- 滲透測試與漏洞掃描
- 合規與文件管理
- 參考範本與專家資源

每項工具皆標註類型（開源/商業）、主要用途、對應 Annex A 控制項與建議部署階段。

---

## 2. 身分與存取管理（IAM / IdM）

| 工具 | 類型 | 主要用途 | 對應控制項 | 建議階段 | 授權模式 |
|---|---|---|---|---|---|
| **Keycloak** | 開源 | SSO、OAuth2/OIDC、SAML、使用者聯邦、權限角色管理 | A.5.17、A.5.18、A.8.5 | Phase 1 | Apache 2.0 |
| **FreeIPA** | 開源 | Linux/Unix 整合式身分管理、Kerberos、LDAP、DNS | A.5.17、A.8.1 | Phase 1 | GPLv3 |
| **Microsoft Entra ID（Azure AD）** | 商業 | 雲端身分、條件式存取、MFA、權限管理 | A.5.17、A.5.23、A.8.5 | Phase 1 | 訂閱制 |
| **Okta** | 商業 | 雲端 SSO、生命週期管理、第三方應用整合 | A.5.18、A.5.19 | Phase 1 | 訂閱制 |
| **Auth0** | 商業 | 雲端身分平台、企業 SSO、社交登入 | A.5.17、A.8.5 | Phase 1 | 訂閱制 |

### 部署決策矩陣

| 場景 | 推薦工具 | 理由 |
|---|---|---|
| Wellfit 代碼庫內部 SSO | Keycloak | 開源、可自我託管、與 Rust 生態整合較易 |
| 企業級雲端身分 | Microsoft Entra ID | 與 Azure/O365 生態整合、條件式存取成熟 |
| 多 SaaS 應用整合 | Okta | 應用目錄廣、生命週期管理自動化 |

---

## 3. 特權存取管理（PAM）

| 工具 | 類型 | 主要用途 | 對應控制項 | 建議階段 | 授權模式 |
|---|---|---|---|---|---|
| **HashiCorp Vault** | 開源/商業 | 特權帳號輪替、臨時憑證、秘密管理 | A.8.2、A.8.9 | Phase 1 | MPL / 商業 |
| **CyberArk** | 商業 | 特權會議監控、密碼庫、特權存取控制 | A.8.2、A.8.16 | Phase 1 | 商業 |
| **BeyondTrust** | 商業 | 特權存取、會議錄製、高危操作審計 | A.8.2、A.8.16 | Phase 1 | 商業 |

### PAM 實作建議

- **特權帳號分離**：管理員帳號與日常操作帳號必須分離
- **臨時憑證**：JWT/SAML assertion 有效期 ≤ 8 小時
- **密碼輪替**：每 90 天或每次使用後自動輪替
- **會議錄製**：所有特權操作會議皆需錄製並可回溯

---

## 4. 認證與多因子驗證（MFA）

| 工具 | 類型 | 主要用途 | 對應控制項 | 建議階段 | 授權模式 |
|---|---|---|---|---|---|
| **Google Authenticator / Authy** | 免費/商業 | TOTP 一次性密碼 | A.5.17 | Phase 1 | 免費 |
| **YubiKey** | 硬體 | FIDO2/WebAuthn 硬體金鑰 | A.5.17、A.8.5 | Phase 1 | 購買 |
| **Microsoft Entra ID MFA** | 商業 | 整合式 MFA 與條件式存取 | A.5.17、A.5.23 | Phase 1 | 訂閱制 |

### MFA 部署建議

- **特權帳戶強制 MFA**：所有管理員、root、特權帳號必須啟用 MFA
- **硬體金鑰優先**：YubiKey 等 FIDO2 金鑰優於 TOTP（抗釣魚）
- **條件式存取**：根據風險等級動態要求 MFA（異地登入、非辦公時間）

---

## 5. 權限政策與職責分離（SoD）

| 工具 | 類型 | 主要用途 | 對應控制項 | 建議階段 | 授權模式 |
|---|---|---|---|---|---|
| **OPA (Open Policy Agent)** | 開源 | 權限策略定義與執行（Rego 語言） | A.5.15、A.6.5 | Phase 1 | Apache 2.0 |
| **Apache Ranger** | 開源 | 資料平台權限統一管理（Hadoop、Spark、HBase） | A.5.15 | Phase 2 | Apache 2.0 |
| **SailPoint** | 商業 | 身分生命週期、存取認證、SoD 違規檢測 | A.5.18、A.6.5 | Phase 2 | 商業 |
| **Saviynt** | 商業 | 雲端身分治理、權限風險評估 | A.5.18、A.6.5 | Phase 2 | 商業 |

### SoD 衝突檢測範例（Rego）

```rego
package sod

deny[msg] {
    user_roles[user] := roles
    roles[_] == "developer"
    roles[_] == "production_deployer"
    msg := sprintf("User %v has conflicting roles: developer and production_deployer", [user])
}
```

---

## 6. 權限審計與監控（SIEM / 日誌）

| 工具 | 類型 | 主要用途 | 對應控制項 | 建議階段 | 授權模式 |
|---|---|---|---|---|---|
| **ELK Stack** | 開源 | 權限存取日誌聚合與分析 | A.8.16 | Phase 1 | Elastic / Apache 2.0 |
| **Wazuh** | 開源 | 入侵偵測、日誌監控、合規報告 | A.8.16、A.8.12 | Phase 1 | GPLv2 |
| **Splunk** | 商業 | 安全事件與權限異常監控 | A.8.16 | Phase 2 | 商業 |
| **Microsoft Sentinel** | 商業 | 雲端 SIEM、SOAR 自動回應 | A.8.16、A.5.23 | Phase 2 | 訂閱制 |

### 權限審計日誌欄位定義（符合 A.8.16）

| 欄位 | 類型 | 說明 | 範例 |
|---|---|---|---|
| `event_time` | ISO 8601 | UTC 時間戳 | `2026-10-25T07:49:42Z` |
| `user_id` | String | 執行者身分 | `alice@company.com` |
| `action` | Enum | CREATE/READ/UPDATE/DELETE/GRANT/REVOKE | `GRANT` |
| `resource` | String | 目標資源 URI/ID | `/api/v1/customers/995C/balance` |
| `result` | Enum | ALLOW/DENY | `ALLOW` |
| `classification` | Enum | 資源分類 | `Confidential` |
| `session_id` | String | 工作階段 ID | `sess_abc123` |
| `ip_address` | String | 來源 IP | `10.0.0.5` |
| `mfa_used` | Boolean | 是否使用 MFA | `true` |
| `privileged` | Boolean | 是否為特權操作 | `true` |

---

## 7. 滲透測試與漏洞掃描

| 工具 | 類型 | 主要用途 | 對應控制項 | 建議階段 | 授權模式 |
|---|---|---|---|---|---|
| **Burp Suite** | 商業/免費 | Web 應用程式權限繞過測試 | A.8.3、A.8.12 | Phase 2 | 商業 / 社區版 |
| **OWASP ZAP** | 開源 | 自動化權限漏洞掃描 | A.8.3、A.8.12 | Phase 1 | Apache 2.0 |
| **OpenSCAP** | 開源 | 系統配置合規性掃描（CIS Benchmark） | A.8.9、A.8.12 | Phase 2 | GPLv2 |

### 滲透測試範圍（權限管理）

1. **水平權限提升**：使用者 A 嘗試存取使用者 B 的資源
2. **垂直權限提升**：一般使用者嘗試存取管理介面
3. **API 權限繞過**：直接呼叫 API 端點，無權限者應返回 403
4. **JWT 操縱**：修改 JWT 聲明提升權限等級
5. **會話固定**：會話 ID 可預測或未過期

---

## 8. 合規與文件管理

| 工具 | 類型 | 主要用途 | 對應控制項 | 建議階段 | 授權模式 |
|---|---|---|---|---|---|
| **Confluence / Notion** | 商業/免費 | ISO 27001 文件、SOP、權限政策撰寫 | A.5.1 | Phase 1 | 訂閱制 / 免費 |
| **ISO 27005 風險矩陣模板** | 免費 | 資產、威脅、脆弱性、權限風險評估 | A.5.1、A.6.1 | Phase 1 | 免費 |
| **Drata / Vanta** | 商業 | ISO 27001 證照自動化證據收集 | A.5.1、A.5.15 | Phase 2 | 商業 |
| **Jira / Azure DevOps** | 商業 | 導入計畫任務追蹤、變更控制 | A.8.9 | Phase 1 | 訂閱制 |

---

## 9. 參考範本清單（可直接複製使用）

| 範本 ID | 範本名稱 | 對應控制項 | 用途 |
|---|---|---|---|
| T-001 | 存取控制政策模板 | A.5.15 | 撰寫組織層級存取控制政策 |
| T-002 | RBAC 角色矩陣範本 | A.5.15 | 定義角色、權限、資源對應關係 |
| T-003 | 權限申請與核准工作流範本 | A.5.18 | 標準化權限申請與核准流程 |
| T-004 | 權限審計日誌欄位定義 | A.8.16 | 統一日誌格式，確保可稽核 |
| T-005 | SoD 衝突檢測規則範本 | A.6.5 | 定義互斥角色組合與檢測邏輯 |
| T-006 | 第三方供應商存取審查清單 | A.5.19 | 評估與記錄第三方存取權限 |
| T-007 | 風險登記冊模板 | A.5.1、A.6.1 | 記錄權限管理相關風險與緩減措施 |

---

## 10. 專家資源與標準取得

| 資源類型 | 名稱 | 取得方式 | 用途 |
|---|---|---|---|
| 官方標準 | ISO/IEC 27001:2022 | ISO.org 購買 | 核心標準文件 |
| 官方標準 | ISO/IEC 27000:2026（第 6 版） | ISO.org 購買 | 術語與定義基礎 |
| 官方標準 | ISO/IEC 27017:2026 | ISO.org 購買 | 雲端控制項 |
| 官方標準 | ISO/IEC 27701:2025 | ISO.org 購買 | 隱私管理（PIMS） |
| 各國標準局 | 台灣 BSMI | https://www.bsmi.gov.tw | 本地標準諮詢 |
| 認證機構 | TÜV / BSI / DNV | 各機構官網 | ISO 27001 認證審核 |
| 資安顧問 | ISO 27001 導入諮詢公司 | 市場評估 | 導入規劃與稽核輔導 |

---

## 11. 工具選型決策矩陣（Wellfit 代碼庫第 1 階段）

| 需求 | 推薦組合 | 理由 |
|---|---|---|
| SSO + 身分管理 | Keycloak | 開源、可自我託管、與 Rust 生態整合 |
| 特權管理 | HashiCorp Vault | 開源、輕量、與 Rust 相容 |
| MFA | YubiKey + TOTP | 硬體金鑰抗釣魚，TOTP 作為備援 |
| 日誌與監控 | ELK Stack | 開源、彈性、與 Rust 應用日誌整合 |
| 政策即代碼 | OPA | 輕量、Rego 語言適合定義權限策略 |
| 合規自動化 | Drata / Vanta | Phase 2 引入，降低認證準備成本 |

---

**本文件為工具包與資源參考彙整，可直接用於工具選型與採購決策。**
