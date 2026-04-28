# MFA (TOTP)

OVLT supports TOTP-based MFA (RFC 6238), compatible with Google Authenticator, Authy, 1Password, and any standard TOTP app.

---

## User flow

### 1. Setup (authenticated)

```bash
curl -X POST http://localhost:3000/auth/mfa/setup \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-Slug: master"
```

Response:

```json
{
  "secret": "BASE32SECRET...",
  "qr_code": "data:image/png;base64,..."
}
```

Show the `qr_code` in your UI (it's an `<img src="...">` data URI) or let the user enter the `secret` manually in their authenticator app.

### 2. Confirm

After the user scans and enters the first code:

```bash
curl -X POST http://localhost:3000/auth/mfa/confirm \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-Slug: master" \
  -H "Content-Type: application/json" \
  -d '{"totp_code": "123456"}'
```

MFA is now active on the account.

### 3. Login with MFA

When MFA is enabled, a standard login returns a challenge instead of tokens:

```json
{"mfa_required": true}
```

The user must then call the MFA challenge endpoint:

```bash
curl -X POST http://localhost:3000/auth/mfa/challenge \
  -H "X-Tenant-Slug: master" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "totp_code": "123456"
  }'
```

Response is the same access/refresh token pair as a normal login.

### 4. Disable MFA (self-service)

```bash
curl -X POST http://localhost:3000/auth/mfa/disable \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-Slug: master" \
  -H "Content-Type: application/json" \
  -d '{"totp_code": "123456"}'
```

---

## Admin: disable MFA for a user

If a user loses their authenticator and can't log in:

```bash
curl -X DELETE http://localhost:3000/users/<user_id>/mfa \
  -H "X-OVLT-Admin-Key: your-admin-key" \
  -H "X-Tenant-Slug: master"
```

Via TUI: Users tab → select user → `d` on the MFA row (or use admin disable option).

---

## Notes

- TOTP codes are 6 digits, 30-second window with ±1 step tolerance
- The TOTP secret is stored encrypted at rest (AES-256-GCM, double-envelope)
- Once confirmed, the plaintext secret is no longer retrievable — the user must re-setup if they lose their device
