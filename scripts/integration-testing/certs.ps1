# Generate test certificates using PowerShell PKI module (non-interactive)
$certDir = "C:\certs"
New-Item -ItemType Directory -Path $certDir -Force | Out-Null

# Generate CA certificate
$caCert = New-SelfSignedCertificate -Subject "CN=Wellfit Test CA" -KeyAlgorithm RSA -KeyLength 2048 -CertStoreLocation "Cert:\CurrentUser\My" -KeyExportPolicy Exportable -NotAfter (Get-Date).AddYears(1) -KeyUsage CertSign, CRLSign -Type Custom
$caPassword = ConvertTo-SecureString -String "test123" -Force -AsPlainText
Export-PfxCertificate -Cert $caCert -FilePath "$certDir\ca.pfx" -Password $caPassword -Force | Out-Null
Export-Certificate -Cert $caCert -FilePath "$certDir\ca.crt" -Type CERT | Out-Null
Write-Host "Generated CA certificate"

# Generate client certificate signed by CA
$clientCert = New-SelfSignedCertificate -Subject "CN=wellfit-client" -KeyAlgorithm RSA -KeyLength 2048 -CertStoreLocation "Cert:\CurrentUser\My" -KeyExportPolicy Exportable -NotAfter (Get-Date).AddYears(1) -Signer $caCert
Export-PfxCertificate -Cert $clientCert -FilePath "$certDir\client.pfx" -Password $caPassword -Force | Out-Null
Export-Certificate -Cert $clientCert -FilePath "$certDir\client.crt" -Type CERT | Out-Null

# Use .NET to extract private key from PFX
Add-Type -AssemblyName System.Security

# Client key
$pfxBytes = [System.IO.File]::ReadAllBytes("$certDir\client.pfx")
$pfx = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($pfxBytes, "test123", [System.Security.Cryptography.X509Certificates.X509KeyStorageFlags]::Exportable)
$privateKey = $pfx.PrivateKey
$rsaParams = $privateKey.ExportParameters($true)
$rsa = [System.Security.Cryptography.RSA]::Create()
$rsa.ImportParameters($rsaParams)
$pemPrivateKey = "-----BEGIN RSA PRIVATE KEY-----`n" + [Convert]::ToBase64String($rsa.ExportRSAPrivateKey(), [Base64FormattingOptions]::InsertLineBreaks) + "`n-----END RSA PRIVATE KEY-----"
$pemPrivateKey | Out-File -FilePath "$certDir\client.key" -Encoding ASCII

# CA key
$caPfxBytes = [System.IO.File]::ReadAllBytes("$certDir\ca.pfx")
$caPfxObj = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($caPfxBytes, "test123", [System.Security.Cryptography.X509Certificates.X509KeyStorageFlags]::Exportable)
$caPrivateKey = $caPfxObj.PrivateKey
$caRsaParams = $caPrivateKey.ExportParameters($true)
$caRsa = [System.Security.Cryptography.RSA]::Create()
$caRsa.ImportParameters($caRsaParams)
$pemCaPrivateKey = "-----BEGIN RSA PRIVATE KEY-----`n" + [Convert]::ToBase64String($caRsa.ExportRSAPrivateKey(), [Base64FormattingOptions]::InsertLineBreaks) + "`n-----END RSA PRIVATE KEY-----"
$pemCaPrivateKey | Out-File -FilePath "$certDir\ca.key" -Encoding ASCII

Write-Host "Generated all certificates and keys" -ForegroundColor Green
Write-Host "CA cert: $certDir\ca.crt"
Write-Host "Client cert: $certDir\client.crt"
Write-Host "Client key: $certDir\client.key"
