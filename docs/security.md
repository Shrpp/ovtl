# Security

---

## Passwords

- Hashed with **Argon2id** (memory-hard, resistant to GPU cracking)
- Parameters: 19 MB memory, 2 iterations, 1 thread (OWASP recommended minimum)
- Plaintext is never stored or logged

---

## Account lockout

- After 5 consecutive failed login attempts, the account is locked for 15 minutes
- Lockout is per-user, per-tenant
- Stale attempt records are purged by the background cleanup task every 6 hours

---

## Transport security

In production (`ENVIRONMENT=production`):
- `DATABASE_URL` must include `sslmode=require` — startup fails otherwise
- HTTPS is expected at the reverse proxy layer (OVLT itself terminates HTTP)

HTTP response headers set on every response:
- `Strict-Transport-Security: max-age=31536000; includeSubDomains` (HSTS)
- `Content-Security-Policy: default-src 'self'`
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Referrer-Policy: strict-origin-when-cross-origin`

---

## Rate limiting

- Per-IP sliding-window rate limiting on all auth endpoints (`/auth/*`, `/oauth/*`)
- Limits apply before tenant resolution — blocks probing across tenants

---

## Admin API protection

- All admin endpoints require `X-OVLT-Admin-Key` header
- If `OVLT_ADMIN_KEY` is not configured, admin endpoints return `404` (not `401`) to prevent enumeration
- The admin key is never included in JWT claims or API responses

---

## Token security

- Access tokens: RS256 JWTs, short-lived (15 min default)
- JTI (JWT ID) tracked — replayed tokens rejected
- Refresh tokens: opaque, stored as Argon2id hash in DB, rotated on use
- Token revocation propagates immediately (JTI blocklist)

---

## Encryption at rest

AES-256-GCM double-envelope for all sensitive fields (TOTP secrets, token seeds):
- Each tenant has a unique data key
- Tenant data keys are stored encrypted (wrapped with `MASTER_ENCRYPTION_KEY`)
- See [Architecture](architecture.md) for the full key hierarchy

---

## Multi-tenant isolation

PostgreSQL Row-Level Security enforces tenant boundaries at the DB layer — not just at the application layer. A query running in the wrong tenant context returns zero rows, not a 403. See [Architecture](architecture.md#multi-tenancy).

---

## CORS

- Wildcard `*` is allowed only in development
- `ENVIRONMENT=production` with `CORS_ALLOWED_ORIGINS=*` causes startup failure
- Set `CORS_ALLOWED_ORIGINS` to an explicit comma-separated list in production

---

## Supply chain

- SBOM generated on every main push (Syft, SPDX format) and attached to GitHub Releases
- Container image scanned for CVEs (Grype) on every main push — critical CVEs fail the build
- SARIF results uploaded to GitHub Security tab

---

## Threat model

| Threat | Mitigation |
|--------|-----------|
| Brute-force passwords | Argon2id + account lockout |
| Token replay | JTI blocklist, short expiry |
| Stolen refresh token | Rotation on use, revocation endpoint |
| Cross-tenant data access | PostgreSQL RLS at DB layer |
| Plaintext secrets at rest | AES-256-GCM double-envelope |
| Lost encryption keys | Auto-generated + printed on first run; must be saved |
| Admin API exposure | Key-gated; 404 if unconfigured |
| Clickjacking | X-Frame-Options: DENY |
| Supply chain | SBOM + Grype scan on every release |

---

## Hardening checklist

```
[ ] OVLT_ADMIN_KEY set to strong random value
[ ] JWT_SECRET, MASTER_ENCRYPTION_KEY, TENANT_WRAP_KEY saved and pinned
[ ] RSA_PRIVATE_KEY set (avoid ephemeral key rotation on restart)
[ ] ENVIRONMENT=production
[ ] DATABASE_URL includes sslmode=require
[ ] CORS_ALLOWED_ORIGINS set explicitly (no wildcard)
[ ] OVLT_ISSUER set to HTTPS public URL
[ ] TLS termination at reverse proxy (nginx, Caddy, etc.)
[ ] Container runs as non-root (Dockerfile uses USER 65534)
[ ] Postgres access restricted to ovlt_rls role only
```
