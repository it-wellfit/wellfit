# PAM 評估報告 — HashiCorp Vault vs CyberArk

> 文件編號：KNB-PAM-EVAL｜狀態：Draft｜版本：v1.0｜日期：2026-10-25
> 適用範圍：Wellfit MTRIX Rust 代碼庫

---

## 1. 比較矩陣

| 項目 | HashiCorp Vault | CyberArk |
|------|----------------|----------|
| 部署方式 | 自我託管/雲端 | 自我託管/雲端 |
| 成本 | 開源/商業 | 商業 |
| 密鑰管理 | 動態憑證、加密即服務 | 密碼庫、特權會議監控 |
| 臨時憑證 | 原生支援（TTL-based） | 需額外配置 |
| 會議錄製 | 需整合 | 原生支援 |
| 密碼輪替 | 自動輪替 | 自動輪替 |
| Rust 整合 | Vault API crate | REST API |
| 學習曲線 | 中等 | 較高 |

## 2. 建議

- **Phase 1（Wellfit 代碼庫）**：HashiCorp Vault（開源、輕量、與 Rust 相容）
- **Phase 2（企業級）**：CyberArk（特權會議監控、密碼庫、合規報告）

## 3. 整合步驟（Phase 1 - Vault）

1. 部署 Vault 伺服器（Docker 或原生）
2. 初始化 Vault（unseal keys、root token）
3. 配置認證方法（AppRole、OIDC）
4. 建立密鑰引擎（KV v2）
5. 配置動態憑證（資料庫、AWS 等）
6. Rust 整合（vault crate）
7. 實作 JIT 權限與 Vault 臨時憑證整合
8. 測試憑證輪替與過期
9. 部署至生產環境

## 4. 驗收標準

- [ ] Vault 伺服器正常運作
- [ ] 認證方法已配置
- [ ] 密鑰引擎已建立
- [ ] Rust 整合測試通過
- [ ] 動態憑證 TTL 設定正確
- [ ] 憑證輪替測試通過
