# Vault Policy for Wellfit MTRIX System
# File: wellfit-policy.hcl
# Applies to: HashiCorp Vault

# Wellfit database credentials
path "secret/data/wellfit/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# Dynamic database credentials
path "database/creds/wellfit-db" {
  capabilities = ["read"]
}

# Lease revocation (for JIT permission expiry)
path "sys/leases/revoke" {
  capabilities = ["update"]
}

# Transit encryption (for data at rest)
path "transit/encrypt/wellfit" {
  capabilities = ["update"]
}

path "transit/decrypt/wellfit" {
  capabilities = ["update"]
}

# Token management (for JIT temporary access)
path "auth/token/create" {
  capabilities = ["update"]
}

path "auth/token/revoke" {
  capabilities = ["update"]
}

# Audit log access (read-only for auditors)
path "sys/audit" {
  capabilities = ["read"]
}

# Health check
path "sys/health" {
  capabilities = ["read"]
}
