# IAM 評估報告 — Keycloak vs Microsoft Entra ID

> 文件編號：KNB-IAM-EVAL｜狀態：Draft｜版本：v1.0｜日期：2026-10-25
> 適用範圍：Wellfit MTRIX Rust 代碼庫

---

## 1. 比較矩陣

| 項目 | Keycloak | Microsoft Entra ID |
|------|----------|-------------------|
| 部署方式 | 自我託管（Docker/原生） | 雲端 SaaS |
| 成本 | 免費（開源） | 訂閱制 |
| SSO 協議 | OAuth2/OIDC/SAML | OAuth2/OIDC/SAML |
| MFA | 內建 TOTP/WebAuthn | 內建 MFA + 條件式存取 |
| 使用者聯邦 | LDAP/AD/Kerberos | Azure AD Connect |
| 客製化 | 高（Java SPI） | 低（雲端限制） |
| 運維負擔 | 高（需自行維護） | 低（Microsoft 維護） |
| 資料主權 | 完全控制 | Microsoft 控制 |
| Rust 整合 | OIDC 客戶端 crate | OIDC 客戶端 crate |

## 2. 建議

- **Phase 1（Wellfit 代碼庫）**：Keycloak（開源、可自我託管、與 Rust 整合較易）
- **Phase 2（企業級）**：Microsoft Entra ID（與 Azure/O365 生態整合、條件式存取成熟）

## 3. 整合步驟（Phase 1 - Keycloak）

1. 部署 Keycloak 伺服器（Docker 或原生）
2. 建立 Wellfit Realm
3. 配置 OIDC 客戶端（wellfit-cli）
4. 整合 Rust OIDC 客戶端 crate
5. 實作 MFA（TOTP + WebAuthn）
6. 配置使用者聯邦（LDAP/AD）
7. 測試 SSO 流程
8. 部署至生產環境

## 4. 驗收標準

- [ ] Keycloak 伺服器正常運作
- [ ] Wellfit Realm 已建立
- [ ] OIDC 客戶端已配置
- [ ] Rust 整合測試通過
- [ ] MFA 已啟用
- [ ] SSO 流程端對端測試通過
