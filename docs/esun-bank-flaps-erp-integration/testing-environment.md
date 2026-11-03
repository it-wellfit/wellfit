# E.Sun Bank Integration Testing Environment Setup

## Overview

This document provides step-by-step instructions for setting up the integration testing environment for the E.Sun Bank API V2.8 integration.

**Last Updated**: 2026-10-28
**Target Platform**: Windows 10/11
**Prerequisites**: Rust toolchain, PowerShell 5.1+

> **Safety/status note (2026-11-02):** This is an unverified local/mock test setup, not an E.Sun sandbox procedure or production certificate guide. Never use generated self-signed keys for bank production; use only isolated synthetic tests until bank-provided credentials, approved endpoints, and security procedures are confirmed.

---

## 1. Prerequisites

### 1.1 Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| Rust | stable-x86_64-pc-windows-msvc | Build toolchain |
| PowerShell | 5.1+ | Script execution |
| Git | Latest | Version control |
| Visual Studio Build Tools | 2022 | MSVC linker |

### 1.2 Required Directories

```
C:\certs\          # Test certificates
C:\mocks\          # Mock API responses
C:\test_data\      # Test data files
C:\logs\           # Log files
```

---

## 2. Windows Defender Application Control (WDAC) Setup

### 2.1 Create WDAC Policy

```powershell
New-Item -ItemType Directory -Path "C:\WDAC" -Force
```

Policy XML content should be saved to `C:\WDAC\WellfitPolicy.xml`.

### 2.2 Verify WDAC Status

```powershell
Get-CIPolicy -FilePath "C:\WDAC\WellfitPolicy.bin"
& "C:\path\to\wellfit.exe" --version
```

---

## 3. Test Certificate Generation

### 3.1 Generate Self-Signed Certificates

```powershell
New-Item -ItemType Directory -Path "C:\certs" -Force
openssl req -x509 -newkey rsa:2048 -keyout "C:\certs\ca.key" -out "C:\certs\ca.crt" -days 365 -nodes -subj "/CN=Wellfit Test CA"
openssl req -newkey rsa:2048 -keyout "C:\certs\client.key" -out "C:\certs\client.csr" -nodes -subj "/CN=wellfit-client"
openssl x509 -req -in "C:\certs\client.csr" -CA "C:\certs\ca.crt" -CAkey "C:\certs\ca.key" -CAcreateserial -out "C:\certs\client.crt" -days 365
openssl verify -CAfile "C:\certs\ca.crt" "C:\certs\client.crt"
```

### 3.2 Certificate Paths

| Certificate | Path | Purpose |
|-------------|------|---------|
| CA Certificate | `C:\certs\ca.crt` | Server verification |
| Client Certificate | `C:\certs\client.crt` | 2-way SSL client auth |
| Client Key | `C:\certs\client.key` | Client private key |

---

## 4. Mock Security Setup

### 4.1 Create Mock Response Files

```powershell
New-Item -ItemType Directory -Path "C:\mocks" -Force
```

Mock payment response should be saved to `C:\mocks\payment_response.json`.
Mock status response should be saved to `C:\mocks\status_response.json`.

### 4.2 Configure Mock Mode

```powershell
$env:ESUN_MOCK_MODE = "true"
$env:ESUN_MOCK_RESPONSES_PATH = "C:\mocks"
$env:ESUN_BASE_URL = "https://localhost:8443"
```

---

## 5. Environment Configuration

### 5.1 Create Environment File

```powershell
$envContent = @"
ESUN_ENV=development
ESUN_BASE_URL=https://localhost:8443
ESUN_TIMEOUT=30
ESUN_CLIENT_CERT=C:\certs\client.crt
ESUN_CLIENT_KEY=C:\certs\client.key
ESUN_CA_CERT=C:\certs\ca.crt
ESUN_MOCK_MODE=true
ESUN_MOCK_RESPONSES_PATH=C:\mocks
ESUN_WDAC_POLICY=C:\WDAC\WellfitPolicy.bin
ESUN_WDAC_ENFORCEMENT=true
ESUN_TEST_VOUCHER_FILE=C:\test_data\voucher_sample.csv
ESUN_TEST_RESULT_FILE=C:\test_data\result_sample.csv
ESUN_LOG_LEVEL=debug
ESUN_LOG_PATH=C:\logs\esun_integration.log
"@
$envContent | Out-File -FilePath ".env" -Encoding UTF8
```

### 5.2 Create Test Data Files

```powershell
New-Item -ItemType Directory -Path "C:\test_data" -Force

$voucherData = @"
VOUCHER_NO,VENDOR_CODE,AMOUNT,BANK_ACCOUNT,DUE_DATE,REFERENCE
AP-001,V001,1000.0000,ACCT-1,2026-11-03,INV-100
AP-002,V002,2500.5000,ACCT-2,2026-11-04,INV-101
AP-003,V003,5000.0000,ACCT-3,2026-11-05,INV-102
"@
$voucherData | Out-File -FilePath "C:\test_data\voucher_sample.csv" -Encoding UTF8

$resultData = @"
BANK_REF,VOUCHER_NO,STATUS,AMOUNT,PROCESSED_AT
EREF-001,AP-001,paid,1000.0000,2026-11-03T09:00:00Z
EREF-002,AP-002,paid,2500.5000,2026-11-04T09:00:00Z
EREF-003,AP-003,failed,5000.0000,2026-11-05T09:00:00Z
"@
$resultData | Out-File -FilePath "C:\test_data\result_sample.csv" -Encoding UTF8
```

---

## 6. Validation Steps

### 6.1 Environment Validation

```powershell
rustc --version
cargo --version
$PSVersionTable.PSVersion
Test-Path "C:\certs\ca.crt"
Test-Path "C:\certs\client.crt"
Test-Path "C:\certs\client.key"
Test-Path "C:\mocks\payment_response.json"
Test-Path "C:\mocks\status_response.json"
Test-Path "C:\test_data\voucher_sample.csv"
Test-Path "C:\test_data\result_sample.csv"
```

### 6.2 Build Validation

```powershell
cd "C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit"
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "build -p wellfit_bankpay" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "test -p wellfit_bankpay" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "clippy -p wellfit_bankpay -- -D warnings" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

### 6.3 Integration Test Validation

```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "test --package wellfit_bankpay --test integration_tests" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

---

## 7. Troubleshooting

### 7.1 Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| WDAC blocks executable | Policy too restrictive | Adjust file rules in policy XML |
| Certificate validation fails | Wrong CA certificate | Regenerate certificates |
| Mock responses not loaded | Wrong path | Check ESUN_MOCK_RESPONSES_PATH |
| Build fails | Missing dependencies | Run cargo build --workspace first |

### 7.2 Debug Commands

```powershell
Get-CIPolicy -FilePath "C:\WDAC\WellfitPolicy.bin"
openssl verify -CAfile "C:\certs\ca.crt" "C:\certs\client.crt"
Get-ChildItem Env:ESUN_*
Test-NetConnection -ComputerName localhost -Port 8443
```

---

## 8. Security Considerations

### 8.1 Certificate Management
- Never commit private keys to version control
- Use separate certificates for dev/test/prod
- Rotate certificates every 90 days
- Store certificates in secure location

### 8.2 Mock Data
- Mock responses are for testing only
- Never use mock data in production
- Clear mock data after testing
- Validate all inputs even in mock mode

### 8.3 WDAC
- Enable WDAC only in production
- Test thoroughly before deployment
- Keep policy updated with new executables
- Monitor for policy violations

---

## 9. References

- **WDAC Documentation**: https://docs.microsoft.com/en-us/windows/security/threat-protection/windows-defender-application-control/
- **OpenSSL Documentation**: https://www.openssl.org/docs/
- **Rust TLS**: https://docs.rs/rustls/
- **Wellfit AGENTS.md**: `AGENTS.md`
