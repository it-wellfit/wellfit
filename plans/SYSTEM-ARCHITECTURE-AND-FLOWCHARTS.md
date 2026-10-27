# Wellfit Enterprise Platform - Complete System Flowcharts & Architecture

> Document ID: KNB-VISUAL-001 | Status: Active | Version: v1.0 | Date: 2026-10-26
> Format: Markdown with Mermaid diagrams (compatible with GitHub, GitLab, VS Code, Obsidian)

---

## 1. System Architecture Overview

```mermaid
graph TB
    subgraph "Wellfit Platform Architecture"
        subgraph "CLI Layer"
            CLI[Wellfit CLI]
            UI[UI Utilities]
        end
        subgraph "Processing Layer"
            MTRIX[MTRIX Engine]
            REPORTS[Report Automator]
            SHOPEE[Shopee Engine]
        end
        subgraph "Core Layer"
            CORE[Wellfit Core]
            MDM[MDM Catalog]
            LEDGER[Idempotency Ledger]
            AC[Access Control]
            JIT[JIT Permissions]
            TEXT[Text Processing]
            CONTRACT[Contract Validation]
        end
        subgraph "Storage"
            SQLITE[(SQLite Database)]
            FILES[File System]
        end
    end

    CLI --> MTRIX
    CLI --> REPORTS
    CLI --> SHOPEE
    CLI --> AC
    CLI --> JIT

    MTRIX --> CORE
    REPORTS --> CORE
    SHOPEE --> CORE

    CORE --> MDM
    CORE --> LEDGER
    CORE --> AC
    CORE --> JIT
    CORE --> TEXT
    CORE --> CONTRACT

    LEDGER --> SQLITE
    MDM --> SQLITE
    AC --> SQLITE
    JIT --> SQLITE

    MTRIX --> FILES
    REPORTS --> FILES
    SHOPEE --> FILES

    style CLI fill:#4a90d9,color:#fff
    style MTRIX fill:#50c878,color:#fff
    style CORE fill:#ff8c00,color:#fff
    style SQLITE fill:#9370db,color:#fff
```

---

## 2. MTRIX Processing Pipeline

```mermaid
flowchart TD
    A[Drop Folder] --> B{Discover Files}
    B --> C[VR-01: File Name Check]
    C -->|Pass| D[VR-02: Header Check]
    C -->|Fail| E[File Rejected]
    D -->|Pass| F[Parse & Repair Rows]
    D -->|Fail| E
    F --> G[VR-03: Date Validation]
    F --> H[VR-04: Document Type]
    F --> I[VR-05: Duplicate Aggregation]
    F --> J[VR-06: Negative Quantity]
    F --> K[VR-07: Package Type]
    G --> L{Any Violations?}
    H --> L
    I --> L
    J --> L
    K --> L
    L -->|Yes| M[Isolate Row + Record Finding]
    L -->|No| N[Accept Row]
    M --> O[Cross-File Validation VR-08 to VR-11]
    N --> O
    O --> P[Generate Report]
    P --> Q[Write to Ledger]
    Q --> R[Write Report File]
    E --> R

    style A fill:#e3f2fd
    style E fill:#ffcdd2
    style M fill:#fff9c4
    style N fill:#c8e6c9
    style Q fill:#d1c4e9
    style R fill:#b2dfdb
```

---

## 3. Access Control Flow (RBAC/ABAC)

```mermaid
flowchart TD
    A[Access Request] --> B{User Exists?}
    B -->|No| C[Denied: User not found]
    B -->|Yes| D{User Enabled?}
    D -->|No| E[Denied: User disabled]
    D -->|Yes| F[Collect User Roles]
    F --> G{SoD Violation?}
    G -->|Yes| H[Denied: SoD violation]
    G -->|No| I[Check Role Permissions]
    I --> J{Permission Granted?}
    J -->|No| K[Denied: No permission]
    J -->|Yes| L{ABAC: MFA Required?}
    L -->|Yes, No MFA| M[Denied: MFA required]
    L -->|No or MFA used| N{ABAC: Privileged?}
    N -->|Yes, Not Privileged| O[Denied: Privileged required]
    N -->|No or Privileged| P[Allowed]
    C --> Q[Record Access Audit]
    E --> Q
    H --> Q
    K --> Q
    M --> Q
    O --> Q
    P --> Q
    Q --> R[Persist to SQLite]

    style A fill:#e3f2fd
    style P fill:#c8e6c9
    style C fill:#ffcdd2
    style E fill:#ffcdd2
    style H fill:#ffcdd2
    style K fill:#ffcdd2
    style M fill:#ffcdd2
    style O fill:#ffcdd2
    style Q fill:#d1c4e9
    style R fill:#b2dfdb
```

---

## 4. JIT Permission Workflow

```mermaid
stateDiagram-v2
    [*] --> Pending: User requests JIT access
    Pending --> Approved: Approver approves
    Pending --> Rejected: Approver rejects
    Pending --> Expired: TTL expires
    Pending --> Cancelled: User cancels
    Approved --> Active: Grant created
    Active --> Consumed: Permission used
    Active --> Revoked: Manual revoke
    Active --> Expired: TTL expires
    Rejected --> [*]
    Expired --> [*]
    Cancelled --> [*]
    Consumed --> [*]
    Revoked --> [*]
```

---

## 5. Ledger Idempotency Flow

```mermaid
flowchart TD
    A[Process Drop] --> B[Compute Fingerprint]
    B --> C{Register Batch}
    C -->|New| D[Proceed with Processing]
    C -->|Duplicate| E{Same Fingerprint?}
    E -->|Yes| F[Reject: True Duplicate]
    E -->|No| G[Release Old Claim]
    G --> H[Re-register Batch]
    H --> D
    D --> I[Process Files]
    I --> J[Record Findings]
    J --> K[Update Ledger]
    K --> L[Generate Report]

    style A fill:#e3f2fd
    style F fill:#ffcdd2
    style D fill:#c8e6c9
    style K fill:#d1c4e9
    style L fill:#b2dfdb
```

---

## 6. Release Gate State Machine

```mermaid
stateDiagram-v2
    [*] --> DraftGenerated: Initialize
    DraftGenerated --> VerifiedPass: Payload valid
    DraftGenerated --> Quarantined: Payload invalid
    VerifiedPass --> AuthorizedDispatch: Dispatch initiated
    Quarantined --> [*]: Rejected
    AuthorizedDispatch --> Completed: Transport success
    AuthorizedDispatch --> Failed: Transport failure
    AuthorizedDispatch --> DispatchUncertain: Timeout
    DispatchUncertain --> Completed: Status check confirms delivery
    DispatchUncertain --> VerifiedPass: Status check confirms not delivered
    Completed --> [*]
    Failed --> [*]
```

---

## 7. ISO 27001:2022 Implementation Roadmap

```mermaid
gantt
    title Wellfit ISO 27001:2022 Implementation Roadmap
    dateFormat  YYYY-MM-DD
    section Phase 1: Technical Controls
    Access Control Policy    :a1, 2026-10-26, 14d
    SSO Deployment           :a2, 2026-11-09, 14d
    MFA Implementation       :a3, 2026-11-23, 14d
    PAM Deployment           :a4, 2026-12-07, 14d
    RBAC/ABAC + JIT          :a5, 2026-12-21, 14d
    Audit Log Integration    :a6, 2027-01-04, 14d
    section Phase 2: Enterprise ISMS
    SoD Conflict Detection   :b1, 2027-01-18, 14d
    Access Rights Review     :b2, 2027-02-01, 14d
    Penetration Testing      :b3, 2027-02-15, 14d
    Third-Party Review       :b4, 2027-03-01, 14d
    Zero Trust Architecture  :b5, 2027-03-15, 14d
    Compliance Automation    :b6, 2027-03-29, 14d
```

---

## 8. Fault Injection Test Flow

```mermaid
flowchart TD
    A[Start Test] --> B[Create Ledger]
    B --> C[Inject SQLite Trigger]
    C --> D[Run process_drop]
    D --> E{Ledger Write Fails?}
    E -->|Yes| F[Verify process_drop Returns Error]
    E -->|No| G[Verify Findings Persisted]
    F --> H[Verify No Partial Report]
    G --> I[Verify No Duplicate Findings]
    H --> J[Test Passed]
    I --> J
    J --> K[Clean Up]

    style A fill:#e3f2fd
    style F fill:#ffcdd2
    style H fill:#ffcdd2
    style G fill:#c8e6c9
    style I fill:#c8e6c9
    style J fill:#b2dfdb
```

---

## 9. Supply Chain Security Pipeline

```mermaid
flowchart LR
    A[Source Code] --> B[cargo build]
    B --> C[cargo clippy]
    C --> D[cargo test]
    D --> E[cargo audit]
    E --> F[cargo deny check]
    F --> G[Generate SBOM]
    G --> H[CI/CD Pipeline]
    H --> I[Deploy]

    style A fill:#e3f2fd
    style E fill:#fff9c4
    style F fill:#fff9c4
    style G fill:#d1c4e9
    style I fill:#c8e6c9
```

---

## 10. Multi-Standard Alignment

```mermaid
graph LR
    subgraph "ISO 27001:2022"
        A1[A.5.15 Access Control]
        A2[A.5.17 Authentication]
        A3[A.5.18 Access Rights]
        A4[A.6.5 SoD]
        A5[A.8.2 Privileged Access]
        A6[A.8.16 Monitoring]
    end
    subgraph "ISO 42001 (AI)"
        B1[6.2.3 AI Access Control]
        B2[6.2.4 AI Authentication]
        B3[6.2.5 AI Permission Mgmt]
        B4[6.3 SoD]
        B5[6.4 AI Privileged Access]
        B6[7.2 AI Monitoring]
    end
    subgraph "ISO 27701:2025 (PIMS)"
        C1[6.2.3 Privacy Access Control]
        C2[6.2.4 Privacy Authentication]
        C3[6.2.5 Privacy Permission Mgmt]
        C4[6.3 SoD]
        C5[6.4 Privacy Privileged Access]
        C6[7.2 Privacy Monitoring]
    end
    subgraph "SOX 404"
        D1[ITGC Access Control]
        D2[ITGC Authentication]
        D3[ITGC Permission Mgmt]
        D4[ITGC SoD]
        D5[ITGC Privileged Access]
        D6[ITGC Monitoring]
    end

    A1 --> B1
    A1 --> C1
    A1 --> D1
    A2 --> B2
    A2 --> C2
    A2 --> D2
    A3 --> B3
    A3 --> C3
    A3 --> D3
    A4 --> B4
    A4 --> C4
    A4 --> D4
    A5 --> B5
    A5 --> C5
    A5 --> D5
    A6 --> B6
    A6 --> C6
    A6 --> D6
```

---

## 11. Wellfit Data Flow

```mermaid
flowchart LR
    subgraph "Input"
        A[MTRIX Files]
        B[Shopee Orders]
        C[Fuyi Reports]
    end
    subgraph "Processing"
        D[Text Parser]
        E[Validation Engine]
        F[Repair Engine]
        G[Cross-File Validation]
    end
    subgraph "Storage"
        H[(SQLite Ledger)]
        I[Report Files]
        J[Repaired Files]
    end
    subgraph "Output"
        K[Validation Report]
        L[Knowledge Export]
        M[Audit Log]
    end

    A --> D
    B --> D
    C --> D
    D --> E
    E --> F
    F --> G
    G --> H
    G --> I
    G --> J
    H --> K
    H --> L
    H --> M

    style A fill:#e3f2fd
    style D fill:#fff9c4
    style E fill:#fff9c4
    style F fill:#fff9c4
    style G fill:#fff9c4
    style H fill:#d1c4e9
    style K fill:#c8e6c9
    style L fill:#c8e6c9
    style M fill:#c8e6c9
```

---

## 12. Quality Gates (G0-G7)

```mermaid
flowchart TD
    G0[G0: 工作區安全] -->|Pass| G1[G1: 需求完整性]
    G0 -->|Fail| STOP1[停止：覆寫操作]
    G1 -->|Pass| G2[G2: 建置品質]
    G1 -->|Fail| STOP2[補齊需求與測試設計]
    G2 -->|Pass| G3[G3: 測試品質]
    G2 -->|Fail| STOP3[阻擋合併或發布]
    G3 -->|Pass| G4[G4: 安全品質]
    G3 -->|Fail| STOP4[重現、除錯及修復]
    G4 -->|Pass| G5[G5: 資料完整性]
    G4 -->|Fail| STOP5[阻擋發布或正式風險核准]
    G5 -->|Pass| G6[G6: 發布安全]
    G5 -->|Fail| STOP6[禁止執行正式資料變更]
    G6 -->|Pass| G7[G7: 證據完整性]
    G6 -->|Fail| STOP7[延後發布]
    G7 -->|Pass| DONE[✅ 宣告完成]
    G7 -->|Fail| STOP8[不得宣告完全完成]

    style G0 fill:#e3f2fd
    style G1 fill:#e3f2fd
    style G2 fill:#e3f2fd
    style G3 fill:#e3f2fd
    style G4 fill:#e3f2fd
    style G5 fill:#e3f2fd
    style G6 fill:#e3f2fd
    style G7 fill:#e3f2fd
    style DONE fill:#c8e6c9
    style STOP1 fill:#ffcdd2
    style STOP2 fill:#ffcdd2
    style STOP3 fill:#ffcdd2
    style STOP4 fill:#ffcdd2
    style STOP5 fill:#ffcdd2
    style STOP6 fill:#ffcdd2
    style STOP7 fill:#ffcdd2
    style STOP8 fill:#ffcdd2
```

---

## 13. PDCA Continuous Improvement Cycle

```mermaid
flowchart LR
    P[Plan\n季度權限審核\n年度ISMS稽核\n年度風險重新評估] --> D[Do\n執行權限重新認證\n執行ISO 27001稽核\nAI異常行為偵測]
    D --> C[Check\n月度權限報告\n季度權限最小化評估\n年度控制項有效性評估]
    C --> A[Act\n撤銷多餘權限\n更新政策與SOP\n調整監控規則]
    A --> P

    style P fill:#e3f2fd
    style D fill:#fff9c4
    style C fill:#d1c4e9
    style A fill:#c8e6c9
```

---

## 14. Risk Management Process

```mermaid
flowchart TD
    A[Asset Identification] --> B[Threat Identification]
    B --> C[Vulnerability Identification]
    C --> D[Risk Analysis]
    D --> E[Risk Evaluation]
    E --> F{Risk Level?}
    F -->|High| G[Risk Treatment: Avoid/Reduce]
    F -->|Medium| H[Risk Treatment: Reduce/Transfer]
    F -->|Low| I[Risk Acceptance]
    G --> J[Implement Controls]
    H --> J
    I --> K[Document Acceptance]
    J --> L[Verify Effectiveness]
    K --> L
    L --> M[Update Risk Register]

    style A fill:#e3f2fd
    style F fill:#fff9c4
    style G fill:#ffcdd2
    style H fill:#fff9c4
    style I fill:#c8e6c9
    style M fill:#b2dfdb
```

---

## 15. Wellfit Deployment Pipeline

```mermaid
flowchart LR
    A[Source Code] --> B[cargo build]
    B --> C[cargo clippy]
    C --> D[cargo test]
    D --> E[cargo audit]
    E --> F[cargo deny check]
    F --> G[Generate SBOM]
    G --> H[CI/CD Pipeline]
    H --> I[Deploy]

    style A fill:#e3f2fd
    style E fill:#fff9c4
    style F fill:#fff9c4
    style G fill:#d1c4e9
    style I fill:#c8e6c9
```

---

## Document Index

| Document | ID | Location |
|----------|-----|----------|
| KNB-MASTER.md | KNB-MASTER | `~/.local/share/kilo/plans/` |
| KNB-ENG-SYSTEM.md | KNB-ENG-SYSTEM | `~/.local/share/kilo/plans/` |
| KNB-PHASE2.md | KNB-PHASE2 | `~/.local/share/kilo/plans/` |
| KNB-INDUSTRY-MASTER.md | KNB-INDUSTRY-MASTER | `~/.local/share/kilo/plans/` |
| ADR-TEMPLATE.md | ADR-TEMPLATE | `Wellfit/plans/adr/` |
| ADR-0001 | ADR-0001 | `Wellfit/plans/adr/` |
| ADR-0002 | ADR-0002 | `Wellfit/plans/adr/` |
| ADR-0003 | ADR-0003 | `Wellfit/plans/adr/` |
| requirements-traceability.md | RTM-001 | `Wellfit/plans/` |
| iam-evaluation.md | IAM-EVAL | `Wellfit/plans/` |
| pam-evaluation.md | PAM-EVAL | `Wellfit/plans/` |
| multi-standard-matrix.md | MULTI-STD | `Wellfit/plans/` |
| industry-standards-implementation.md | INDUSTRY-IMPL | `Wellfit/plans/` |
| phase2-deployment-runbook.md | PHASE2-DEPLOY | `Wellfit/plans/` |
| keycloak-realm-export.json | KEYCLOAK | `Wellfit/plans/` |
| vault-policy.hcl | VAULT | `Wellfit/plans/` |
| wazuh-agent.conf | WAZUH-AGENT | `Wellfit/plans/` |
| wazuh-rules.xml | WAZUH-RULES | `Wellfit/plans/` |
| scripts/daily-backup.ps1 | BACKUP | `Wellfit/scripts/` |
| scripts/restore.ps1 | RESTORE | `Wellfit/scripts/` |
| deny.toml | DENY | `Wellfit/` |
| .github/workflows/security.yml | SECURITY-CI | `Wellfit/.github/workflows/` |

---

**本文件為 Wellfit 系統完整架構與流程圖參考，所有流程圖使用 Mermaid 語法，可在 GitHub、GitLab、VS Code、Obsidian 等支援 Mermaid 的工具中渲染。**
