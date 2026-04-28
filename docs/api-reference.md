# API Reference

Base URL: `http://localhost:3000` (or your `OVLT_ISSUER`).

All admin endpoints require the header: `X-OVLT-Admin-Key: <OVLT_ADMIN_KEY>`.  
Tenant-scoped endpoints require the header: `X-Tenant-Slug: <slug>`.  
Protected user endpoints require `Authorization: Bearer <access_token>`.

---

## Health

```
GET /health
```

Response:
```json
{"status": "ok", "version": "0.1.0"}
```

---

## OIDC Discovery

```
GET /.well-known/openid-configuration
GET /.well-known/jwks.json
```

Standard OIDC discovery and JWK Set endpoints.

---

## Auth (tenant-scoped)

All routes below require `X-Tenant-Slug` header.

### Register

```
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "Secret1234!"
}
```

### Login

```
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "Secret1234!"
}
```

Response:
```json
{
  "access_token": "...",
  "refresh_token": "...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

### Refresh token

```
POST /auth/refresh
Content-Type: application/json

{"refresh_token": "..."}
```

### Logout

```
POST /auth/logout
Authorization: Bearer <token>
```

### Revoke token

```
POST /auth/revoke
Authorization: Bearer <token>
Content-Type: application/json

{"token": "<refresh_token>"}
```

### Forgot password

```
POST /auth/forgot-password
Content-Type: application/json

{"email": "user@example.com"}
```

### Reset password

```
POST /auth/reset-password
Content-Type: application/json

{
  "token": "<reset_token>",
  "password": "NewSecret1234!"
}
```

### Verify email OTP

```
POST /auth/verify-otp
Content-Type: application/json

{"code": "123456"}
```

### MFA challenge (login step 2)

```
POST /auth/mfa/challenge
Content-Type: application/json

{
  "email": "user@example.com",
  "totp_code": "123456"
}
```

### MFA setup (authenticated)

```
POST /auth/mfa/setup
Authorization: Bearer <token>
```

Response includes TOTP `secret` and `qr_code` (data URI).

### MFA confirm

```
POST /auth/mfa/confirm
Authorization: Bearer <token>
Content-Type: application/json

{"totp_code": "123456"}
```

### MFA disable

```
POST /auth/mfa/disable
Authorization: Bearer <token>
Content-Type: application/json

{"totp_code": "123456"}
```

### Social login

```
GET /auth/google
GET /auth/github
```

Redirects to provider. Callback handled at `/auth/:provider/callback`.

---

## OIDC / OAuth 2.0 Authorization Server

### Authorization endpoint

```
GET /oauth/authorize
  ?response_type=code
  &client_id=<client_id>
  &redirect_uri=<uri>
  &scope=openid email profile
  &state=<random>
  &code_challenge=<S256>
  &code_challenge_method=S256
```

### Token endpoint

```
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

# Authorization Code + PKCE
grant_type=authorization_code
&code=<code>
&redirect_uri=<uri>
&client_id=<id>
&code_verifier=<verifier>

# Client Credentials (M2M)
grant_type=client_credentials
&client_id=<id>
&client_secret=<secret>
&scope=<optional>

# Refresh Token
grant_type=refresh_token
&refresh_token=<token>
&client_id=<id>
```

### Introspect

```
POST /oauth/introspect
Content-Type: application/x-www-form-urlencoded

token=<access_token>
```

### Revoke (OAuth)

```
POST /oauth/revoke
Content-Type: application/x-www-form-urlencoded

token=<refresh_token>
```

---

## Current user

```
GET /users/me
Authorization: Bearer <token>
```

---

## Admin — Tenants

All require `X-OVLT-Admin-Key`.

```
POST   /tenants           — create tenant
GET    /tenants           — list tenants
GET    /tenants/slugs     — list tenant slugs
```

---

## Admin — Clients

All require `X-OVLT-Admin-Key` and `X-Tenant-Slug`.

```
POST   /clients           — create client
GET    /clients           — list clients
PUT    /clients/:id       — update client
DELETE /clients/:id       — deactivate client
```

---

## Admin — Users

All require `X-OVLT-Admin-Key` and `X-Tenant-Slug`.

```
GET    /users                           — list users
POST   /users                           — create user
GET    /users/:id                       — get user
PUT    /users/:id                       — update user
DELETE /users/:id                       — delete user
GET    /users/:id/verification-code     — get email verification code
GET    /users/:id/password-reset-token  — get password reset token
DELETE /users/:id/mfa                   — admin disable MFA
```

---

## Admin — Roles

All require `X-OVLT-Admin-Key` and `X-Tenant-Slug`.

```
GET    /roles                              — list roles
POST   /roles                             — create role
PUT    /roles/:id                         — update role
DELETE /roles/:id                         — delete role
GET    /users/:id/roles                   — list user roles
POST   /users/:id/roles                   — assign role to user
DELETE /users/:user_id/roles/:role_id     — revoke user role
GET    /clients/:id/roles                 — list client roles (M2M)
POST   /clients/:id/roles                 — assign role to client
DELETE /clients/:client_id/roles/:role_id — revoke client role
```

---

## Admin — Permissions

All require `X-OVLT-Admin-Key` and `X-Tenant-Slug`.

```
GET    /permissions                           — list permissions
POST   /permissions                           — create permission
PUT    /permissions/:id                       — update permission
DELETE /permissions/:id                       — delete permission
GET    /roles/:id/permissions                 — list role permissions
POST   /roles/:id/permissions                 — assign permission to role
DELETE /roles/:role_id/permissions/:perm_id   — revoke permission from role
```

---

## Admin — Sessions

All require `X-OVLT-Admin-Key` and `X-Tenant-Slug`.

```
GET    /sessions         — list sessions
DELETE /sessions/:id     — revoke session
```

---

## Admin — Identity Providers

All require `X-OVLT-Admin-Key` and `X-Tenant-Slug`.

```
GET    /identity-providers       — list providers
POST   /identity-providers       — create provider
PUT    /identity-providers/:id   — update provider
DELETE /identity-providers/:id   — delete provider
```

---

## Admin — Audit Log

Requires `X-OVLT-Admin-Key` and `X-Tenant-Slug`.

```
GET /audit-log?page=1&per_page=50
```

---

## Admin — Settings

Requires `Authorization: Bearer` (admin user) and `X-Tenant-Slug`.

```
GET  /settings
PUT  /settings
```
