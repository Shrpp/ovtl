# Configuration

All configuration is via environment variables.

---

## Required

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string. In production must include `sslmode=require`. |

---

## Auto-generated (recommended to save)

These are generated on first run if not provided. OVLT prints them to stderr. **Save them — losing them makes encrypted data unrecoverable.**

| Variable | Description |
|----------|-------------|
| `JWT_SECRET` | HS256 signing key for refresh tokens. Min 32 chars. |
| `MASTER_ENCRYPTION_KEY` | AES-256-GCM master key for double-envelope encryption. Min 32 chars. |
| `TENANT_WRAP_KEY` | Wraps per-tenant keys. Must differ from `MASTER_ENCRYPTION_KEY`. Min 32 chars. |

---

## Bootstrap (first-run)

| Variable | Default | Description |
|----------|---------|-------------|
| `OVLT_ADMIN_KEY` | — | Static key required in `X-OVLT-Admin-Key` header on all admin endpoints. If unset, admin endpoints return 404. |
| `OVLT_BOOTSTRAP_ADMIN_EMAIL` | — | Email for the first admin user in the master tenant. |
| `OVLT_BOOTSTRAP_ADMIN_PASSWORD` | — | Password for the bootstrap admin. Required if email is set. |
| `OVLT_BOOTSTRAP_TENANT_SLUG` | `master` | Slug for the first tenant created on startup. |

---

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `0.0.0.0` | Bind address. |
| `SERVER_PORT` | `3000` | Port. |
| `ENVIRONMENT` | `development` | Set to `production` to enable: JSON logs, strict CORS, `sslmode` required. |
| `OVLT_ISSUER` | `http://localhost:3000` | Issuer URL in OIDC discovery and `iss` claim of id_tokens. Set to your public URL. |

---

## Tokens

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_EXPIRATION_MINUTES` | `15` | Access token lifetime (minutes). |
| `REFRESH_TOKEN_EXPIRATION_DAYS` | `30` | Refresh token lifetime (days). |

---

## CORS

| Variable | Default | Description |
|----------|---------|-------------|
| `CORS_ALLOWED_ORIGINS` | `*` | Comma-separated list of allowed origins. Wildcards forbidden in production — set explicitly or startup fails. |

Example: `CORS_ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com`

---

## RSA key (id_tokens)

| Variable | Description |
|----------|-------------|
| `RSA_PRIVATE_KEY` | Base64-encoded PKCS8 PEM for RS256 id_token signing. If unset, an ephemeral key is generated (lost on restart — JWKs endpoint will serve a new key after restart). Set in production. |

Generate a persistent key:

```bash
openssl genrsa 2048 | openssl pkcs8 -topk8 -nocrypt -out key.pem
base64 -i key.pem | tr -d '\n'
# paste output as RSA_PRIVATE_KEY
```

---

## Social login

| Variable | Description |
|----------|-------------|
| `GOOGLE_CLIENT_ID` | Google OAuth2 client ID. |
| `GOOGLE_CLIENT_SECRET` | Google OAuth2 client secret. |
| `GOOGLE_REDIRECT_URL` | Must match registered redirect URI (e.g. `https://your-domain/auth/google/callback`). |
| `GITHUB_CLIENT_ID` | GitHub OAuth2 client ID. |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth2 client secret. |
| `GITHUB_REDIRECT_URL` | Must match registered redirect URI. |

All three vars must be set for a provider to be enabled.

---

## Production checklist

```
DATABASE_URL          includes sslmode=require
JWT_SECRET            saved and pinned
MASTER_ENCRYPTION_KEY saved and pinned
TENANT_WRAP_KEY       saved and pinned
OVLT_ISSUER           set to public HTTPS URL
ENVIRONMENT           production
CORS_ALLOWED_ORIGINS  explicit list (no wildcard)
RSA_PRIVATE_KEY       set (or accept key rotation on restart)
OVLT_ADMIN_KEY        set to a strong random value
```
