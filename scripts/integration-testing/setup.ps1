# E.Sun Bank Integration Testing Environment Setup Script
# Run with: powershell -ExecutionPolicy Bypass -File integration-testing\setup.ps1

param(
    [switch]$SkipCerts,
    [switch]$SkipWDAC,
    [switch]$SkipMockData
)

$ErrorActionPreference = "Stop"

Write-Host "=== E.Sun Bank Integration Testing Setup ===" -ForegroundColor Cyan

# Create required directories
$dirs = @("C:\certs", "C:\mocks", "C:\test_data", "C:\logs", "C:\WDAC")
foreach ($dir in $dirs) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Host "Created directory: $dir" -ForegroundColor Green
    }
}

# Generate test certificates
if (-not $SkipCerts) {
    Write-Host "`nGenerating test certificates..." -ForegroundColor Yellow
    
    if (-not (Get-Command openssl -ErrorAction SilentlyContinue)) {
        Write-Host "WARNING: openssl not found. Skipping certificate generation." -ForegroundColor Red
        Write-Host "Please install OpenSSL or generate certificates manually." -ForegroundColor Red
    } else {
        # Generate CA certificate
        openssl req -x509 -newkey rsa:2048 -keyout "C:\certs\ca.key" -out "C:\certs\ca.crt" -days 365 -nodes -subj "/CN=Wellfit Test CA" 2>$null
        Write-Host "Generated CA certificate" -ForegroundColor Green
        
        # Generate client certificate
        openssl req -newkey rsa:2048 -keyout "C:\certs\client.key" -out "C:\certs\client.csr" -nodes -subj "/CN=wellfit-client" 2>$null
        openssl x509 -req -in "C:\certs\client.csr" -CA "C:\certs\ca.crt" -CAkey "C:\certs\ca.key" -CAcreateserial -out "C:\certs\client.crt" -days 365 2>$null
        Write-Host "Generated client certificate" -ForegroundColor Green
        
        # Verify certificates
        $verifyResult = openssl verify -CAfile "C:\certs\ca.crt" "C:\certs\client.crt" 2>&1
        if ($verifyResult -match "OK") {
            Write-Host "Certificate verification: OK" -ForegroundColor Green
        } else {
            Write-Host "Certificate verification failed: $verifyResult" -ForegroundColor Red
        }
    }
}

# Create mock data
if (-not $SkipMockData) {
    Write-Host "`nCreating mock data..." -ForegroundColor Yellow
    
    # Mock payment response
    $mockPayment = @"
{
  "rsHeader": {
    "msgId": "MOCK12345678202610280000001",
    "logId": "MOCK-LOG-001",
    "responseTime": "2026/10/28 10:00:00"
  },
  "rsStatus": {
    "code": "910000",
    "message": "SUCCESS",
    "moreInfo": ""
  },
  "rsData": {
    "dataCount": "1",
    "txRecordList": [
      {
        "txSeq": "MOCK-TX-001",
        "autoDelay": "0",
        "payerAccountNo": "1234567890123",
        "payerId": "1234567890",
        "payeeBankId": "013",
        "payeeBranchId": "0001",
        "payeeAccountNo": "9876543210",
        "payeeName": "Test Payee",
        "payeeId": "",
        "txDate": "20261028",
        "txAmt": "100000",
        "chargeFeeType": "0",
        "memo": "Test payment",
        "notifyMethod": "1",
        "notifyEmail": "",
        "notifyFax": "",
        "writeOffNo": "MOCK-TX-001",
        "remark": "",
        "detailList": []
      }
    ]
  }
}
"@
    $mockPayment | Out-File -FilePath "C:\mocks\payment_response.json" -Encoding UTF8
    Write-Host "Created mock payment response" -ForegroundColor Green
    
    # Mock status response
    $mockStatus = @"
{
  "rsHeader": {
    "msgId": "MOCK12345678202610280000002",
    "logId": "MOCK-LOG-002",
    "responseTime": "2026/10/28 10:00:00"
  },
  "rsStatus": {
    "code": "910000",
    "message": "SUCCESS",
    "moreInfo": ""
  },
  "rsData": {
    "txStatus": "0000",
    "bankRef": "BANK-REF-001",
    "txAmt": "100000",
    "txDate": "20261028",
    "errorMessage": ""
  }
}
"@
    $mockStatus | Out-File -FilePath "C:\mocks\status_response.json" -Encoding UTF8
    Write-Host "Created mock status response" -ForegroundColor Green
    
    # Test data files
    $voucherData = "VOUCHER_NO,VENDOR_CODE,AMOUNT,BANK_ACCOUNT,DUE_DATE,REFERENCE`nAP-001,V001,1000.0000,ACCT-1,2026-11-03,INV-100`nAP-002,V002,2500.5000,ACCT-2,2026-11-04,INV-101`nAP-003,V003,5000.0000,ACCT-3,2026-11-05,INV-102"
    $voucherData | Out-File -FilePath "C:\test_data\voucher_sample.csv" -Encoding UTF8
    Write-Host "Created test voucher data" -ForegroundColor Green
    
    $resultData = "BANK_REF,VOUCHER_NO,STATUS,AMOUNT,PROCESSED_AT`nEREF-001,AP-001,paid,1000.0000,2026-11-03T09:00:00Z`nEREF-002,AP-002,paid,2500.5000,2026-11-04T09:00:00Z`nEREF-003,AP-003,failed,5000.0000,2026-11-05T09:00:00Z"
    $resultData | Out-File -FilePath "C:\test_data\result_sample.csv" -Encoding UTF8
    Write-Host "Created test result data" -ForegroundColor Green
}

# Create .env file
Write-Host "`nCreating .env file..." -ForegroundColor Yellow
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
Set-Content -Path "C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit\.env" -Value $envContent -Encoding UTF8
Write-Host "Created .env file" -ForegroundColor Green

# Copy WDAC policy to config
Copy-Item -Path "config\wdac-policy.xml" -Destination "C:\WDAC\WellfitPolicy.xml" -Force
Write-Host "Copied WDAC policy" -ForegroundColor Green

Write-Host "`n=== Setup Complete ===" -ForegroundColor Cyan
Write-Host "Next steps:"
Write-Host "1. Review .env file and adjust settings"
Write-Host "2. Build: powershell -ExecutionPolicy Bypass -File ..\..\Knowledge\tools\dev-env\build.ps1 -Command 'build -p wellfit_bankpay'"
Write-Host "3. Test: powershell -ExecutionPolicy Bypass -File ..\..\Knowledge\tools\dev-env\build.ps1 -Command 'test -p wellfit_bankpay'"

