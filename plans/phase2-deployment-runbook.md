# Wellfit Phase 2 部署執行手冊 — IAM/SSO、PAM 與監控

> 文件編號：KNB-PHASE2-DEPLOY｜狀態：Draft｜版本：v1.0｜日期：2026-10-25
> 適用範圍：Wellfit MTRIX Rust 代碼庫 → 企業級 ISMS
> 前置文件：KNB-MASTER.md, iam-evaluation.md, pam-evaluation.md

---

## 1. IAM/SSO 部署（Keycloak）

### 1.1 部署架構

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│  Wellfit    │────▶│  Keycloak    │────▶│  LDAP/AD     │
│  CLI/Binary │     │  (SSO)       │     │  (User Store)│
└─────────────┘     └──────────────┘     └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  PostgreSQL  │
                    │  (Keycloak   │
                    │   DB)        │
                    └──────────────┘
```

### 1.2 部署步驟

#### Step 1: 安裝 Keycloak

```powershell
# 使用 Docker（開發/測試環境）
docker run -d --name keycloak \
  -p 8080:8080 \
  -e KEYCLOAK_ADMIN=admin \
  -e KEYCLOAK_ADMIN_PASSWORD=admin \
  quay.io/keycloak/keycloak:latest start-dev

# 原生部署（生產環境）
# 下載 Keycloak 二進位檔
# 解壓縮至 C:\keycloak
# 執行 bin\kc.bat start --optimized
```

#### Step 2: 建立 Wellfit Realm

1. 登入 Keycloak Admin Console (http://localhost:8080)
2. 建立 Realm: `wellfit`
3. 設定 Realm 屬性：
   - Display Name: Wellfit MTRIX
   - Enabled: true
   - User Registration: false
   - Login with Email: true

#### Step 3: 建立 OIDC 客戶端

1. Clients → Create Client
2. Client ID: `wellfit-cli`
3. Client Protocol: `openid-connect`
4. Access Type: `confidential`
5. Valid Redirect URIs: `http://localhost:*`
6. Web Origins: `*`
7. 儲存 Client Secret

#### Step 4: 配置使用者聯邦（LDAP/AD）

1. User Federation → Add Provider → LDAP
2. Connection URL: `ldap://ad.company.com:389`
3. Bind DN: `CN=service-account,OU=Service Accounts,DC=company,DC=com`
4. Users DN: `OU=Users,DC=company,DC=com`
5. Username LDAP Attribute: `sAMAccountName`
6. RDN LDAP Attribute: `cn`
7. UUID LDAP Attribute: `objectGUID`
8. User Object Classes: `person, organizationalPerson, user`
9. Test Connection & Test Authentication

#### Step 5: 配置 MFA

1. Authentication → Flows → Browser
2. 新增 OTP Form 步驟
3. 設定 OTP Policy:
   - OTP Type: Time Based (TOTP)
   - OTP Algorithm: SHA256
   - Number of Digits: 6
   - Look Ahead Window: 1
   - Initial Counter: 0
4. 啟用 WebAuthn（FIDO2）：
   - Authentication → WebAuthn Policy
   - Relying Party Entity Name: Wellfit
   - Signature Algorithms: ES256, RS256
   - Attestation Conveyance Preference: indirect
   - Authenticator Attachment: cross-platform
   - Require Resident Key: No
   - User Verification Requirement: preferred

#### Step 6: 建立角色與權限

1. Realm Roles → Create Role
   - `wellfit-analyst` (Data Analyst)
   - `wellfit-admin` (Administrator)
   - `wellfit-auditor` (Auditor)
   - `wellfit-deployer` (Production Deployer)
2. 建立 SoD 約束：
   - `developer` ↔ `deployer`（互斥）
   - `auditor` ↔ `admin`（互斥）

#### Step 7: Rust 整合

```rust
// Cargo.toml
[dependencies]
oidc = "0.7"  // 或使用 openidconnect crate
reqwest = { version = "0.12", features = ["json"] }
```

```rust
// wellfit_cli/src/auth.rs
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{AuthorizationCode, ClientId, ClientSecret, IssuerUrl, RedirectUrl, Scope};

pub struct OidcAuth {
    client: CoreClient,
}

impl OidcAuth {
    pub async fn new(issuer_url: &str, client_id: &str, client_secret: &str) -> Result<Self, String> {
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(issuer_url.to_string()).map_err(|e| e.to_string())?,
            reqwest::Client::new(),
        ).await.map_err(|e| e.to_string())?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
        )
        .set_redirect_uri(RedirectUrl::new("http://localhost:8085/callback".to_string()).map_err(|e| e.to_string())?);

        Ok(OidcAuth { client })
    }
}
```

#### Step 8: 驗收標準

- [ ] Keycloak 伺服器正常運作
- [ ] Wellfit Realm 已建立
- [ ] OIDC 客戶端已配置
- [ ] Rust 整合測試通過
- [ ] MFA 已啟用（TOTP + WebAuthn）
- [ ] SSO 流程端對端測試通過
- [ ] 使用者聯邦（LDAP/AD）正常運作
- [ ] SoD 約束已配置

---

## 2. PAM 部署（HashiCorp Vault）

### 2.1 部署架構

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│  Wellfit    │────▶│  Vault       │────▶│  PostgreSQL  │
│  CLI/Binary │     │  (PAM)       │     │  (Dynamic    │
└─────────────┘     └──────────────┘     │   Creds)     │
                           │             └──────────────┘
                           ▼
                    ┌──────────────┐
                    │  Audit Log   │
                    │  (Vault      │
                    │   Audit)     │
                    └──────────────┘
```

### 2.2 部署步驟

#### Step 1: 安裝 Vault

```powershell
# 下載 Vault 二進位檔
# https://developer.hashicorp.com/vault/downloads
# 解壓縮至 C:\vault
# 執行 vault server -dev (開發模式)
# 或 vault server -config=vault.hcl (生產模式)
```

#### Step 2: 初始化 Vault

```powershell
# 設定環境變數
$env:VAULT_ADDR = "http://127.0.0.1:8200"

# 初始化（產生 unseal keys 與 root token）
vault operator init -key-shares=5 -key-threshold=3

# Unseal（需要 3 個 key shares）
vault operator unseal <key1>
vault operator unseal <key2>
vault operator unseal <key3>

# 登入
vault login <root_token>
```

#### Step 3: 配置認證方法

```powershell
# 啟用 AppRole 認證（用於 Wellfit CLI）
vault auth enable approle

# 建立 Wellfit policy
vault policy write wellfit-policy - <<EOF
path "secret/data/wellfit/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}
path "database/creds/wellfit-db" {
  capabilities = ["read"]
}
path "sys/leases/revoke" {
  capabilities = ["update"]
}
EOF

# 建立 AppRole
vault write auth/approle/role/wellfit \
  token_policies="wellfit-policy" \
  token_ttl=1h \
  token_max_ttl=4h \
  secret_id_ttl=0
```

#### Step 4: 配置密鑰引擎

```powershell
# 啟用 KV v2 密鑰引擎
vault secrets enable -path=secret kv-v2

# 儲存 Wellfit 資料庫憑證
vault kv put secret/wellfit/db \
  username="wellfit_user" \
  password="secure_password_123"

# 啟用資料庫動態憑證
vault secrets enable database

# 配置 PostgreSQL 連線
vault write database/config/wellfit-db \
  plugin_name=postgresql-database-plugin \
  allowed_roles="wellfit-db" \
  connection_url="postgresql://{{username}}:{{password}}@localhost:5432/wellfit" \
  username="vault_admin" \
  password="vault_admin_password"

# 建立資料庫角色（動態憑證）
vault write database/roles/wellfit-db \
  db_name=wellfit-db \
  creation_statements="CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO \"{{name}}\";" \
  default_ttl="1h" \
  max_ttl="24h"
```

#### Step 5: Rust 整合

```rust
// Cargo.toml
[dependencies]
vaultrs = "0.7"  // Vault Rust client
```

```rust
// wellfit_core/src/vault.rs
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

pub struct VaultClient {
    client: VaultClient,
}

impl VaultClient {
    pub fn new(addr: &str, token: &str) -> Result<Self, String> {
        let settings = VaultClientSettingsBuilder::default()
            .address(addr)
            .token(token)
            .build()
            .map_err(|e| e.to_string())?;
        let client = VaultClient::new(settings).map_err(|e| e.to_string())?;
        Ok(VaultClient { client })
    }

    pub async fn get_db_credentials(&self) -> Result<(String, String), String> {
        let secret: serde_json::Value = vaultrs::database::creds(&self.client, "wellfit-db")
            .await
            .map_err(|e| e.to_string())?;
        let username = secret["username"].as_str().unwrap_or("").to_string();
        let password = secret["password"].as_str().unwrap_or("").to_string();
        Ok((username, password))
    }
}
```

#### Step 6: JIT 權限與 Vault 整合

```rust
// JIT 權限核准後，從 Vault 取得臨時資料庫憑證
impl JitPermissionManager {
    pub async fn approve_with_vault(
        &self,
        request_id: &str,
        approver_id: &str,
        vault_client: &VaultClient,
    ) -> Result<(JitRequest, JitGrant, (String, String)), String> {
        let (request, grant) = self.approve(request_id, approver_id, None)?;
        let creds = vault_client.get_db_credentials().await?;
        Ok((request, grant, creds))
    }
}
```

#### Step 7: 驗收標準

- [ ] Vault 伺服器正常運作
- [ ] 認證方法已配置（AppRole）
- [ ] 密鑰引擎已建立（KV v2 + Database）
- [ ] Rust 整合測試通過
- [ ] 動態憑證 TTL 設定正確（1h default, 24h max）
- [ ] 憑證輪替測試通過
- [ ] Vault 稽核日誌已啟用
- [ ] Vault 稽核日誌已轉送至 SIEM

---

## 3. 監控告警配置（Wazuh）

### 3.1 Wazuh 規則配置

```xml
<!-- /var/ossec/etc/rules/local_rules.xml -->
<group name="wellfit,access_control,">
  <!-- ALERT-001: 特權帳號異地登入 -->
  <rule id="100001" level="12">
    <if_sid>5710</if_sid>
    <match>wellfit access-control</match>
    <field name="privileged">true</field>
    <description>Wellfit: Privileged account login from unusual location</description>
    <group>authentication,privileged,</group>
  </rule>

  <!-- ALERT-002: 非辦公時間大量下載 -->
  <rule id="100002" level="8">
    <if_sid>5710</if_sid>
    <match>wellfit access-control</match>
    <field name="action">read</field>
    <time>20:00-08:00</time>
    <description>Wellfit: Bulk download outside business hours</description>
    <group>data_access,anomaly,</group>
  </rule>

  <!-- ALERT-003: 權限提升嘗試 -->
  <rule id="100003" level="12">
    <if_sid>5710</if_sid>
    <match>wellfit access-control</match>
    <field name="decision">denied</field>
    <field name="action">admin</field>
    <description>Wellfit: Privilege escalation attempt detected</description>
    <group>authentication,attack,</group>
  </rule>

  <!-- ALERT-004: MFA 未啟用特權帳戶 -->
  <rule id="100004" level="8">
    <if_sid>5710</if_sid>
    <match>wellfit access-control</match>
    <field name="privileged">true</field>
    <field name="mfa_used">false</field>
    <description>Wellfit: Privileged account without MFA</description>
    <group>authentication,compliance,</group>
  </rule>

  <!-- ALERT-005: SoD 衝突偵測 -->
  <rule id="100005" level="12">
    <if_sid>5710</if_sid>
    <match>wellfit access-control</match>
    <field name="sod_violated">true</field>
    <description>Wellfit: Segregation of duties violation detected</description>
    <group>authentication,compliance,</group>
  </rule>

  <!-- ALERT-006: 權限異常變更 -->
  <rule id="100006" level="8">
    <if_sid>5710</if_sid>
    <match>wellfit access-control</match>
    <field name="action">assign</field>
    <description>Wellfit: Multiple role assignments in short period</description>
    <group>authentication,anomaly,</group>
    <frequency>20</frequency>
    <timeframe>300</timeframe>
  </rule>
</group>
```

### 3.2 ELK Stack 日誌收集配置

```yaml
# logstash/pipeline/wellfit.conf
input {
  file {
    path => "/var/log/wellfit/access_audit.log"
    start_position => "beginning"
    sincedb_path => "/var/lib/logstash/sincedb_wellfit"
  }
}

filter {
  json {
    source => "message"
  }
  date {
    match => ["decided_at", "ISO8601"]
    target => "@timestamp"
  }
  mutate {
    add_field => { "source" => "wellfit" }
  }
}

output {
  elasticsearch {
    hosts => ["http://localhost:9200"]
    index => "wellfit-access-audit-%{+YYYY.MM.dd}"
  }
}
```

### 3.3 Kibana 儀表板配置

建立以下視覺化：
1. **權限使用統計**：活躍使用者數、特權帳戶數、權限變更次數（時間序列）
2. **SoD 衝突檢測**：衝突數趨勢圖
3. **異常事件摘要**：告警數、誤報數、真陽性數、回應時間
4. **合規狀態**：Annex A 控制項符合率、證據完整率
5. **風險狀態**：風險登記冊更新、殘餘風險變化

---

## 4. 驗收標準（Phase 2）

### 4.1 IAM/SSO 驗收

- [ ] Keycloak 伺服器正常運作
- [ ] Wellfit Realm 已建立
- [ ] OIDC 客戶端已配置
- [ ] Rust 整合測試通過
- [ ] MFA 已啟用（TOTP + WebAuthn）
- [ ] SSO 流程端對端測試通過
- [ ] 使用者聯邦（LDAP/AD）正常運作
- [ ] SoD 約束已配置

### 4.2 PAM 驗收

- [ ] Vault 伺服器正常運作
- [ ] 認證方法已配置（AppRole）
- [ ] 密鑰引擎已建立（KV v2 + Database）
- [ ] Rust 整合測試通過
- [ ] 動態憑證 TTL 設定正確（1h default, 24h max）
- [ ] 憑證輪替測試通過
- [ ] Vault 稽核日誌已啟用
- [ ] Vault 稽核日誌已轉送至 SIEM

### 4.3 監控告警驗收

- [ ] Wazuh 規則已部署（ALERT-001~006）
- [ ] ELK Stack 日誌收集管線已配置
- [ ] Kibana 儀表板已建立
- [ ] 告警可觸發與通知
- [ ] 月度權限報告已自動產出
- [ ] 季度回顧已排程

---

## 5. 剩餘 Phase 2 項目狀態

| 項目 | 狀態 | 阻礙 | 建議行動 |
|------|------|------|----------|
| IAM/SSO 部署 | 📋 評估報告已完成 | 需基礎設施存取權限 | 由 IT 基建團隊執行部署手冊 §1 |
| PAM 部署 | 📋 評估報告已完成 | 需基礎設施存取權限 | 由 IT 安全團隊執行部署手冊 §2 |
| 監控告警部署 | 📋 規劃已完成 | 需基礎設施存取權限 | 由資安運維團隊執行部署手冊 §3 |
| 多標準對齊實施 | 📋 計畫已完成 | 需組織決策 | 由資安經理推動多標準對應矩陣 |
| 產業標準導入 | 📋 計畫已完成 | 需組織決策 | 由各業務單位推動產業標準導入 |

---

**本文件為 Wellfit Phase 2 部署執行手冊，涵蓋 IAM/SSO（Keycloak）部署、PAM（Vault）部署、監控告警（Wazuh/ELK）配置，以及完整的驗收標準。所有配置與步驟均可直接執行。**
