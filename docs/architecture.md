# Architecture

---

## Overview

```
┌─────────────────────────────────────────────────────┐
│                    ovlt-core (Axum)                  │
│                                                     │
│  ┌──────────┐  ┌──────────┐  ┌────────────────┐    │
│  │ Auth API │  │ Admin API│  │ OIDC/OAuth 2.0 │    │
│  └────┬─────┘  └────┬─────┘  └───────┬────────┘    │
│       │              │                │             │
│  ┌────▼──────────────▼────────────────▼──────────┐  │
│  │              SeaORM + PostgreSQL               │  │
│  │         (Row-Level Security per tenant)        │  │
│  └────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘

┌─────────────────┐
│   ovlt (TUI)    │  connects via HTTP API
└─────────────────┘
```

---

## Multi-tenancy

Each tenant is a row in the `tenants` table. All data tables (users, clients, roles, sessions, etc.) have a `tenant_id` column.

**PostgreSQL Row-Level Security (RLS)** enforces isolation at the database level:

1. The application runs as role `ovlt_rls` (not a superuser)
2. On every request, the handler sets: `SET LOCAL app.tenant_id = '<uuid>'`
3. All tables have RLS policies: `USING (tenant_id = current_setting('app.tenant_id')::uuid)`
4. `FORCE ROW LEVEL SECURITY` prevents bypassing by table owners

This means even a SQL injection that executes in the application's DB session cannot read data from another tenant — the RLS policy blocks it.

---

## Encryption model

OVLT uses **double-envelope AES-256-GCM encryption** for sensitive fields (TOTP secrets, token seeds):

```
plaintext
   │
   ▼ encrypt with tenant_data_key (AES-256-GCM)
ciphertext_level1
   │
   ▼ encrypt tenant_data_key with wrapped_tenant_key (AES-256-GCM)
ciphertext_level2  ← stored in DB
```

Key hierarchy:

```
MASTER_ENCRYPTION_KEY  (env var, never stored)
   │ derives
   ▼
wrapped_tenant_key  (stored in tenants table, encrypted with MASTER_ENCRYPTION_KEY)
   │ unwraps
   ▼
tenant_data_key  (in-memory only during request)
   │ encrypts
   ▼
per-field ciphertext  (stored in DB)

TENANT_WRAP_KEY  (env var, separate key — wraps tenant keys at a second envelope layer)
```

Losing `MASTER_ENCRYPTION_KEY` or `TENANT_WRAP_KEY` makes all encrypted fields unrecoverable.

---

## Token model

### Access tokens (JWT, RS256)

- Short-lived (default 15 min)
- Signed with RSA private key; verified via JWKS endpoint
- Claims: `sub`, `iss`, `aud`, `exp`, `iat`, `jti`, `tenant_id`, `roles` (M2M only)
- JTI tracked in DB — replayed or revoked tokens rejected at introspection

### Refresh tokens

- Long-lived (default 30 days), opaque
- Stored hashed in DB
- Rotation: each use issues a new refresh token, old one invalidated

### id_tokens (OIDC)

- Issued alongside access tokens for `authorization_code` flow
- RS256 signed, includes standard OIDC claims (`email`, `name`, etc.)

---

## Request lifecycle

```
HTTP request
   │
   ▼ security_headers_middleware  (HSTS, CSP, X-Frame-Options)
   │
   ▼ CORS layer
   │
   ▼ tenant_middleware             (reads X-Tenant-Slug, sets app.tenant_id)
   │
   ▼ rate_limit_middleware         (per-IP sliding window)
   │
   ▼ auth_middleware               (validates Bearer token, extracts claims)
   │
   ▼ handler
```

---

## Background tasks

A tokio task runs every 6 hours and cleans up:
- Expired refresh tokens
- Expired JTI replay-protection entries
- Stale login attempt records (lockout cleanup)
- Expired sessions

---

## Crates

| Crate | Purpose |
|-------|---------|
| `axum` | HTTP framework |
| `sea-orm` | ORM + migrations |
| `jsonwebtoken` | JWT encode/decode |
| `argon2` | Password hashing |
| `aes-gcm` | AES-256-GCM encryption |
| `totp-rs` | TOTP generation/verification |
| `ratatui` | TUI (admin CLI) |
| `sysinfo` | Startup memory/CPU stats |
