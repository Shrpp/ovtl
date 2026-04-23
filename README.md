# OVTL

Self-hosted authentication platform. Drop-in Keycloak alternative — OIDC Authorization Server, multi-tenant, zero-knowledge encrypted, ~20 MB RAM.

## Features

- **OIDC Authorization Server** — Authorization Code + PKCE, RS256 id_tokens, JWKS endpoint
- **Multi-tenant** — PostgreSQL Row Level Security, isolated per-tenant encryption keys
- **Zero-knowledge encryption** — AES-256-GCM double-envelope via [hefesto](https://crates.io/crates/hefesto)
- **JWT access tokens** (HS256) + rotating refresh tokens
- **Social login** — Google, GitHub OAuth2
- **Account lockout** after failed attempts
- **Audit log** for all auth events
- **Security headers** — CSP, HSTS, X-Frame-Options, Referrer-Policy
- **Per-IP rate limiting** on auth endpoints
- **Bootstrap admin** — first-run creates master tenant + admin user, no manual DB setup

## Quickstart (Docker)

```bash
git clone <repo-url>
cd ovtl
cp ovtl-core/.env.example ovtl-core/.env  # fill in secrets

docker compose up
```

`docker compose up` runs migrations automatically (`--migrate` flag) then starts the server on port 3000. On first boot, a master tenant and admin user are created from `BOOTSTRAP_ADMIN_EMAIL` / `BOOTSTRAP_ADMIN_PASSWORD` if those vars are set.

## Quickstart (local)

```bash
git clone <repo-url>
cd ovtl
cp ovtl-core/.env.example ovtl-core/.env  # fill in secrets

docker compose up -d postgres

cd ovtl-core
cargo run -- --migrate  # runs migrations then starts server
```

## Environment Variables

### Required

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | HS256 signing key (min 32 chars). `openssl rand -base64 64` |
| `MASTER_ENCRYPTION_KEY` | Outer AES-GCM envelope key (min 32 chars) |
| `TENANT_WRAP_KEY` | Inner key for wrapping per-tenant keys — must differ from master |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `OVTL_ISSUER` | `http://localhost:3000` | Issuer URL used in OIDC discovery and id_token |
| `OVTL_ADMIN_KEY` | — | Secret for admin endpoints (`X-OVTL-Admin-Key` header). Required to use `/tenants` and `/oauth/introspect` |
| `RSA_PRIVATE_KEY` | — | Base64-encoded PKCS8 PEM for RS256. Generated ephemerally if not set (keys lost on restart) |
| `BOOTSTRAP_ADMIN_EMAIL` | — | Auto-create master tenant admin on first boot |
| `BOOTSTRAP_ADMIN_PASSWORD` | — | Password for bootstrap admin |
| `BOOTSTRAP_TENANT_SLUG` | `master` | Slug for the bootstrap tenant |
| `JWT_EXPIRATION_MINUTES` | `15` | Access token lifetime |
| `REFRESH_TOKEN_EXPIRATION_DAYS` | `30` | Refresh token lifetime |
| `SERVER_HOST` | `0.0.0.0` | Bind address |
| `SERVER_PORT` | `3000` | Bind port |
| `ENVIRONMENT` | `development` | Set `production` for HSTS, JSON logging, strict CORS |
| `CORS_ALLOWED_ORIGINS` | `*` | Comma-separated origins. Wildcard forbidden in production |
| `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` / `GOOGLE_REDIRECT_URL` | — | Google social login |
| `GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET` / `GITHUB_REDIRECT_URL` | — | GitHub social login |

> Production checklist: explicit `CORS_ALLOWED_ORIGINS`, `sslmode` in `DATABASE_URL`, `RSA_PRIVATE_KEY` set (stable across restarts), all secret keys ≥ 32 chars.

## API Endpoints

### OIDC / OAuth2 Authorization Server

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/.well-known/openid-configuration` | OIDC discovery document |
| `GET` | `/.well-known/jwks.json` | RS256 public key set |
| `GET` | `/oauth/authorize` | Authorization Code + PKCE flow |
| `POST` | `/oauth/token` | Exchange code for tokens |
| `POST` | `/oauth/introspect` | Token introspection (requires `X-OVTL-Admin-Key`) |

### Auth (requires `x-ovtl-tenant-id` header)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/auth/register` | Tenant | Create user |
| `POST` | `/auth/login` | Tenant | Login, returns access + refresh tokens |
| `POST` | `/auth/refresh` | Tenant | Rotate refresh token |
| `POST` | `/auth/logout` | JWT | Revoke refresh token |
| `POST` | `/auth/revoke` | JWT | Revoke all tokens |
| `GET` | `/auth/:provider` | Tenant | Start social login (google / github) |
| `GET` | `/auth/:provider/callback` | — | Social login callback |

### Users

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/users/me` | JWT | Authenticated user profile |

### Admin (requires `X-OVTL-Admin-Key` header)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/tenants` | Create tenant |
| `GET` | `/tenants` | List tenants |
| `POST` | `/clients` | Register OAuth2 client |
| `GET` | `/clients` | List OAuth2 clients |
| `DELETE` | `/clients/:id` | Deactivate OAuth2 client |

### System

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |

## Usage Examples

### Register a user

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

### OIDC Authorization Code flow

```bash
# 1. Discover endpoints
curl http://localhost:3000/.well-known/openid-configuration

# 2. Register an OAuth2 client
curl -X POST http://localhost:3000/clients \
  -H "X-OVTL-Admin-Key: <admin-key>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My App",
    "redirect_uris": ["https://myapp.com/callback"],
    "scopes": ["openid", "email", "profile"],
    "grant_types": ["authorization_code"]
  }'

# 3. Redirect user to /oauth/authorize with PKCE
# 4. Exchange code at /oauth/token
```

### Create a tenant

```bash
curl -X POST http://localhost:3000/tenants \
  -H "X-OVTL-Admin-Key: <admin-key>" \
  -H "Content-Type: application/json" \
  -d '{"name": "Acme Corp", "slug": "acme"}'
```

## Architecture

```
ovtl/
├── ovtl-core/              # Main server (Axum + SeaORM)
│   ├── src/
│   │   ├── handlers/       # HTTP request handlers
│   │   ├── middleware/     # Auth, tenant, security, rate limiting
│   │   ├── services/       # Business logic (tokens, JWK, users, audit, lockout)
│   │   ├── entity/         # SeaORM database models
│   │   ├── routes/         # Router definitions
│   │   ├── config.rs       # Environment configuration
│   │   ├── db.rs           # Connection pool + RLS transaction helper
│   │   ├── error.rs        # Unified error types
│   │   └── state.rs        # Shared application state (includes JwkService)
│   └── migration/          # SeaORM migrations (run with --migrate flag)
├── docker-compose.yml      # Development stack
└── docker-compose.prod.yml # Production stack (read-only fs, no exposed DB port)
```

**Multi-tenancy:** Every request carries `x-ovtl-tenant-id`. The tenant middleware decrypts the per-tenant key, stored double-envelope encrypted (`TENANT_WRAP_KEY` wraps the key, `MASTER_ENCRYPTION_KEY` wraps the wrap). PostgreSQL RLS enforces row-level isolation via `SET LOCAL app.tenant_id`.

**Encryption:** User emails are AES-256-GCM encrypted at rest. A keyed SHA-256 hash (`email_lookup`) enables lookups without decrypting every row — the lookup key never leaves the server.

**OIDC:** RS256 keypair generated at startup (or loaded from `RSA_PRIVATE_KEY`). Public key served at `/.well-known/jwks.json`. Authorization codes are single-use and expire after 10 minutes. PKCE S256 is required.

## Security

- Passwords hashed with Argon2id
- Access tokens expire in 15 minutes (configurable)
- Account locked 30 minutes after 5 failed attempts per 15-minute window
- Audit log written fire-and-forget — never blocks requests
- `FORCE ROW LEVEL SECURITY` on `users`, `oauth_accounts`, `oauth_clients`, `authorization_codes` tables
- HSTS, CSP, Referrer-Policy, Permissions-Policy headers in production

## Running Tests

```bash
docker compose up -d postgres
cd ovtl-core
cargo run -- --migrate
cargo test
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[Elastic License 2.0](LICENSE) — free to self-host and contribute; cannot be resold or offered as a managed service.
