# M2M / Client Credentials

Machine-to-machine (M2M) auth uses the OAuth 2.0 `client_credentials` grant. No user is involved — a service authenticates directly with its client ID and secret, and receives an access token with embedded roles.

---

## 1. Create an M2M client

Via TUI: Clients tab → `n` → set **Grant Types** to `client_credentials` (do not include `authorization_code`).

Via API:

```bash
curl -X POST http://localhost:3000/clients \
  -H "X-OVLT-Admin-Key: your-admin-key" \
  -H "X-Tenant-Slug: master" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-service",
    "grant_types": ["client_credentials"]
  }'
```

Response includes `client_id` and `client_secret`. **Save the secret — it is shown once.**

---

## 2. Assign roles (optional)

Roles included in the token let downstream services perform RBAC checks.

Via TUI: select the client → press `r` → assign roles.

Via API:

```bash
# List available roles first
curl http://localhost:3000/roles \
  -H "X-OVLT-Admin-Key: your-admin-key" \
  -H "X-Tenant-Slug: master"

# Assign role to client
curl -X POST http://localhost:3000/clients/<client_id>/roles \
  -H "X-OVLT-Admin-Key: your-admin-key" \
  -H "X-Tenant-Slug: master" \
  -H "Content-Type: application/json" \
  -d '{"role_id": "<role_uuid>"}'
```

---

## 3. Get a token

```bash
curl -X POST http://localhost:3000/oauth/token \
  -H "X-Tenant-Slug: master" \
  -d "grant_type=client_credentials" \
  -d "client_id=<client_id>" \
  -d "client_secret=<client_secret>"
```

Response:

```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

---

## 4. Token contents

The access token is a signed JWT. Decode it to inspect:

```bash
# Decode payload (no verification — for debugging only)
echo "eyJ..." | cut -d. -f2 | base64 -d 2>/dev/null | jq
```

Example payload:

```json
{
  "sub": "<client_id>",
  "iss": "http://localhost:3000",
  "aud": "ovlt",
  "exp": 1714300000,
  "iat": 1714299100,
  "jti": "<uuid>",
  "client_id": "<client_id>",
  "tenant_id": "<uuid>",
  "roles": ["admin", "data-reader"]
}
```

`roles` is omitted from the token if no roles are assigned.

---

## 5. Verify the token in your service

Use the JWKS endpoint to verify the RS256 signature:

```
GET http://localhost:3000/.well-known/jwks.json
```

Most JWT libraries support JWKS auto-discovery via:

```
GET http://localhost:3000/.well-known/openid-configuration
```

Example (Node.js / `jose`):

```js
import { createRemoteJWKSet, jwtVerify } from 'jose';

const JWKS = createRemoteJWKSet(
  new URL('http://localhost:3000/.well-known/jwks.json')
);

const { payload } = await jwtVerify(token, JWKS, {
  issuer: 'http://localhost:3000',
  audience: 'ovlt',
});

console.log(payload.roles); // ["admin", "data-reader"]
```

---

## Flow summary

```
Service                     OVLT
  │                           │
  │ POST /oauth/token          │
  │ client_credentials grant  │
  │ ─────────────────────────>│
  │                           │ load client + roles from DB
  │      access_token (JWT)   │
  │ <─────────────────────────│
  │                           │
  │ call downstream API       │
  │ Authorization: Bearer ... │
  │ ─────────────────────────>│ (verify signature via JWKS)
```
