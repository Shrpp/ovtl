# OVTL

Self-hosted authentication platform. Drop-in alternative to Keycloak and Clerk — multi-tenant, zero-knowledge encrypted, ~20 MB RAM.

## Features

- Multi-tenant with PostgreSQL Row Level Security
- Zero-knowledge encryption (AES-256-GCM via [hefesto](https://crates.io/crates/hefesto))
- JWT access tokens (HS256) + rotating refresh tokens
- OAuth2 social login (Google, GitHub)
- Account lockout after failed attempts
- Audit log for all auth events
- Security headers (CSP, HSTS, X-Frame-Options, etc.)
- Per-IP rate limiting on auth endpoints

## Requirements

- Rust 1.75+
- Docker & Docker Compose (for local Postgres)
- PostgreSQL 16 (via Docker or local install)

## Quickstart

**1. Clone and configure**

```bash
git clone <repo-url>
cd ovtl
cp ovtl-core/.env.example ovtl-core/.env   # edit secrets before use
```

**2. Start Postgres**

```bash
docker compose up -d postgres
```

**3. Run migrations**

```bash
cd ovtl-core/migration
cargo run -- up
cd ../..
```

**4. Start the server**

```bash
cd ovtl-core
cargo run
```

Server starts on `http://0.0.0.0:3000` by default.

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | ✅ | — | PostgreSQL connection string |
| `JWT_SECRET` | ✅ | — | HS256 signing key (min 32 chars). Generate: `openssl rand -base64 64` |
| `MASTER_ENCRYPTION_KEY` | ✅ | — | Outer envelope key for AES-GCM (min 32 chars) |
| `TENANT_WRAP_KEY` | ✅ | — | Inner key for wrapping per-tenant keys — must differ from master (min 32 chars) |
| `JWT_EXPIRATION_MINUTES` | — | `15` | Access token lifetime in minutes |
| `REFRESH_TOKEN_EXPIRATION_DAYS` | — | `30` | Refresh token lifetime in days |
| `SERVER_HOST` | — | `0.0.0.0` | Bind address |
| `SERVER_PORT` | — | `3000` | Bind port |
| `ENVIRONMENT` | — | `development` | Set to `production` to enable HSTS, JSON logging, and strict CORS/DB checks |
| `CORS_ALLOWED_ORIGINS` | — | `*` | Comma-separated origins. Wildcard forbidden in production |
| `GOOGLE_CLIENT_ID` | — | — | Google OAuth2 client ID |
| `GOOGLE_CLIENT_SECRET` | — | — | Google OAuth2 client secret |
| `GOOGLE_REDIRECT_URL` | — | — | Google OAuth2 redirect URI |
| `GITHUB_CLIENT_ID` | — | — | GitHub OAuth2 client ID |
| `GITHUB_CLIENT_SECRET` | — | — | GitHub OAuth2 client secret |
| `GITHUB_REDIRECT_URL` | — | — | GitHub OAuth2 redirect URI |

> Production requires: explicit `CORS_ALLOWED_ORIGINS`, `sslmode` in `DATABASE_URL`, and all three secret keys ≥ 32 chars.

## API Endpoints

All auth endpoints require the `x-ovtl-tenant-id: <uuid>` header.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/health` | — | Health check |
| `POST` | `/auth/register` | Tenant | Create a new user |
| `POST` | `/auth/login` | Tenant | Login, returns access + refresh tokens |
| `POST` | `/auth/refresh` | Tenant | Rotate refresh token, returns new pair |
| `POST` | `/auth/logout` | JWT | Revoke refresh token |
| `POST` | `/auth/revoke` | JWT | Revoke all tokens for the authenticated user |
| `GET` | `/users/me` | JWT | Return authenticated user profile |
| `GET` | `/auth/:provider` | Tenant | Start OAuth2 flow (google / github) |
| `GET` | `/auth/:provider/callback` | — | OAuth2 callback |

### Register

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -H "x-ovtl-tenant-id: <tenant-uuid>" \
  -d '{"email": "user@example.com", "password": "SecurePass1"}'
```

### Login

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -H "x-ovtl-tenant-id: <tenant-uuid>" \
  -d '{"email": "user@example.com", "password": "SecurePass1"}'
```

```json
{
  "access_token": "eyJ...",
  "refresh_token": "550e8400-...",
  "expires_in": 900
}
```

### Authenticated request

```bash
curl http://localhost:3000/users/me \
  -H "Authorization: Bearer <access_token>" \
  -H "x-ovtl-tenant-id: <tenant-uuid>"
```

## Architecture

```
ovtl/
├── ovtl-core/              # Main server (Axum + SeaORM)
│   ├── src/
│   │   ├── handlers/       # HTTP request handlers
│   │   ├── middleware/     # Auth, tenant, security, rate limiting
│   │   ├── services/       # Business logic (tokens, users, audit, lockout)
│   │   ├── entity/         # SeaORM database models
│   │   ├── routes/         # Router definitions
│   │   ├── config.rs       # Environment configuration
│   │   ├── db.rs           # Connection pool + RLS transaction helper
│   │   ├── error.rs        # Unified error types
│   │   └── state.rs        # Shared application state
│   ├── migration/          # SeaORM migrations
│   └── tests/              # Integration tests
└── docker-compose.yml      # Local development stack
```

**Multi-tenancy:** Every request carries `x-ovtl-tenant-id`. The tenant middleware decrypts the per-tenant key, which is stored double-envelope encrypted (`TENANT_WRAP_KEY` inner, `MASTER_ENCRYPTION_KEY` outer). PostgreSQL RLS policies enforce row-level isolation using `SET LOCAL app.tenant_id`.

**Encryption:** User emails are AES-256-GCM encrypted at rest. A deterministic `email_lookup` hash (SHA-256 keyed with the tenant key) enables lookups without decrypting all rows.

## Security

- Passwords hashed with Argon2id
- Access tokens expire in 15 minutes (configurable)
- Account locked for 30 minutes after 5 failed login attempts per 15-minute window
- All auth events written to `audit_log` table (fire-and-forget, never blocks requests)
- `FORCE ROW LEVEL SECURITY` on `users` and `oauth_accounts` tables
- HSTS, CSP, Referrer-Policy, and Permissions-Policy headers in production

## Running Tests

Requires a running Postgres instance (use `docker compose up -d postgres` then `cargo run -- up` for migrations).

```bash
cargo test
```
