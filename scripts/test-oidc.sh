#!/usr/bin/env bash
# Full OIDC Authorization Code + PKCE flow.
# Requires: curl, jq, openssl
# Usage: bash scripts/test-oidc.sh
#        OVLT_URL=http://... OVLT_ADMIN_KEY=... bash scripts/test-oidc.sh

set -euo pipefail

BASE="${OVLT_URL:-http://localhost:3000}"
ADMIN_KEY="${OVLT_ADMIN_KEY:-dev-admin-key}"
EMAIL="${OVLT_ADMIN_EMAIL:-admin@example.com}"
PASSWORD="${OVLT_ADMIN_PASSWORD:-Admin1234!}"
REDIRECT_URI="http://localhost:8080/callback"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

step() { echo -e "\n${BOLD}${CYAN}▶ $*${NC}"; }
ok()   { echo -e "${GREEN}✓ $*${NC}"; }
fail() { echo -e "${RED}✗ $*${NC}"; exit 1; }

require() {
  command -v "$1" >/dev/null 2>&1 || fail "'$1' not found — install it first"
}
require curl; require jq; require openssl

# Helper: curl with separate body + status code (macOS-compatible)
curl_with_status() {
  local _tmp; _tmp=$(mktemp)
  local _code
  _code=$(curl -s -o "$_tmp" -w '%{http_code}' "$@")
  local _body; _body=$(cat "$_tmp"); rm -f "$_tmp"
  echo "$_body"$'\x1f'"$_code"
}
body()   { echo "${1%%$'\x1f'*}"; }
status() { echo "${1##*$'\x1f'}"; }

# ── 0. Health check ───────────────────────────────────────────────────────────
step "Health check"
STATUS=$(curl -sf "$BASE/health" | jq -r '.status' 2>/dev/null || echo "down")
[[ "$STATUS" == "ok" ]] || fail "Server not reachable at $BASE — is it running? (make dev)"
ok "Server is up"

# ── 1. Get tenant ID ──────────────────────────────────────────────────────────
step "Fetching tenant list"
TENANTS=$(curl -sf "$BASE/tenants" -H "x-ovlt-admin-key: $ADMIN_KEY")
TENANT_ID=$(echo "$TENANTS" | jq -r '.[0].id')
TENANT_SLUG=$(echo "$TENANTS" | jq -r '.[0].slug')
[[ "$TENANT_ID" != "null" && -n "$TENANT_ID" ]] || fail "No tenants found. Did bootstrap run?"
ok "Tenant: $TENANT_SLUG ($TENANT_ID)"

# ── 2. Create OAuth client ────────────────────────────────────────────────────
step "Creating OAuth client"
R=$(curl_with_status -X POST "$BASE/clients" \
  -H "x-ovlt-admin-key: $ADMIN_KEY" \
  -H "x-ovlt-tenant-id: $TENANT_ID" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"test-oidc-$(date +%s)\",
    \"redirect_uris\": [\"$REDIRECT_URI\"],
    \"is_confidential\": true
  }")
[[ "$(status "$R")" == "201" ]] || fail "Client creation failed (HTTP $(status "$R")): $(body "$R")"
CLIENT_ID=$(body "$R" | jq -r '.client_id')
CLIENT_SECRET=$(body "$R" | jq -r '.client_secret')
ok "client_id: $CLIENT_ID"
echo "   client_secret: $CLIENT_SECRET"

# ── 3. Login ──────────────────────────────────────────────────────────────────
step "Logging in as $EMAIL (tenant: $TENANT_SLUG)"
R=$(curl_with_status -X POST "$BASE/auth/login" \
  -H "x-ovlt-tenant-slug: $TENANT_SLUG" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")
[[ "$(status "$R")" == "200" ]] || fail "Login failed (HTTP $(status "$R")): $(body "$R")
Hint: if tenant is not 'master', bootstrap user may not exist.
Run: docker compose down -v && make dev"
ACCESS_TOKEN=$(body "$R" | jq -r '.access_token')
ok "Got access token (${#ACCESS_TOKEN} chars)"

# ── 4. PKCE ───────────────────────────────────────────────────────────────────
step "Generating PKCE code_verifier + code_challenge (S256)"
CODE_VERIFIER=$(openssl rand -base64 48 | tr -d '=+/ \n' | head -c 64)
CODE_CHALLENGE=$(printf '%s' "$CODE_VERIFIER" \
  | openssl dgst -sha256 -binary \
  | openssl base64 \
  | tr '+/' '-_' \
  | tr -d '=\n')
ok "code_verifier generated (${#CODE_VERIFIER} chars)"

# ── 5. Authorize ──────────────────────────────────────────────────────────────
step "Calling /oauth/authorize"
CLIENT_ID_ENC=$(python3 -c "import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1]))" "$CLIENT_ID")
REDIRECT_ENC=$(python3 -c "import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1]))" "$REDIRECT_URI")
AUTHORIZE_URL="$BASE/oauth/authorize?client_id=$CLIENT_ID_ENC&redirect_uri=$REDIRECT_ENC&response_type=code&scope=openid%20email%20profile&code_challenge=$CODE_CHALLENGE&code_challenge_method=S256"

LOCATION=$(curl -s -H "Authorization: Bearer $ACCESS_TOKEN" -D - -o /dev/null "$AUTHORIZE_URL" \
  | grep -i '^location:' | tr -d '\r' | awk '{print $2}')

[[ -n "$LOCATION" ]] || fail "No redirect from /oauth/authorize — check server logs"
CODE=$(echo "$LOCATION" | grep -oE '[?&]code=[^&]+' | cut -d= -f2)
[[ -n "$CODE" ]] || fail "No code in redirect: $LOCATION"
ok "Got authorization code: ${CODE:0:16}..."

# ── 6. Token exchange ─────────────────────────────────────────────────────────
step "Exchanging code for tokens (POST /oauth/token)"
R=$(curl_with_status -X POST "$BASE/oauth/token" \
  --data-urlencode "grant_type=authorization_code" \
  --data-urlencode "code=$CODE" \
  --data-urlencode "redirect_uri=$REDIRECT_URI" \
  --data-urlencode "client_id=$CLIENT_ID" \
  --data-urlencode "client_secret=$CLIENT_SECRET" \
  --data-urlencode "code_verifier=$CODE_VERIFIER")
[[ "$(status "$R")" == "200" ]] || fail "Token exchange failed (HTTP $(status "$R")): $(body "$R")"
ACCESS_TOKEN_OIDC=$(body "$R" | jq -r '.access_token')
ID_TOKEN=$(body "$R" | jq -r '.id_token')
SCOPE=$(body "$R" | jq -r '.scope')
ok "access_token: ${ACCESS_TOKEN_OIDC:0:20}..."
ok "id_token:     ${ID_TOKEN:0:20}..."
ok "scope:        $SCOPE"

# ── 7. Introspect ─────────────────────────────────────────────────────────────
step "Introspecting access token"
R=$(curl_with_status -X POST "$BASE/oauth/introspect" \
  -H "x-ovlt-admin-key: $ADMIN_KEY" \
  --data-urlencode "token=$ACCESS_TOKEN_OIDC")
ACTIVE=$(body "$R" | jq -r '.active')
[[ "$ACTIVE" == "true" ]] || fail "Token not active: $(body "$R")"
ok "Token active — sub: $(body "$R" | jq -r '.sub')"

# ── 8. JWKS ───────────────────────────────────────────────────────────────────
step "Fetching JWKS"
JWKS=$(curl -sf "$BASE/.well-known/jwks.json")
[[ "$(echo "$JWKS" | jq -r '.keys[0].kty')" == "RSA" ]] || fail "Unexpected JWKS: $JWKS"
ok "JWKS OK — alg: $(echo "$JWKS" | jq -r '.keys[0].alg'), kid: $(echo "$JWKS" | jq -r '.keys[0].kid')"

# ── 9. Discovery ──────────────────────────────────────────────────────────────
step "OIDC discovery document"
DISC=$(curl -sf "$BASE/.well-known/openid-configuration")
ok "issuer: $(echo "$DISC" | jq -r '.issuer')"

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}${GREEN}All steps passed. OIDC flow is working.${NC}"
echo ""
echo "  id_token payload:"
echo "$ID_TOKEN" | cut -d. -f2 | python3 -c "
import sys, base64, json
data = sys.stdin.read().strip()
pad = 4 - len(data) % 4
print(json.dumps(json.loads(base64.urlsafe_b64decode(data + '='*pad)), indent=2))
"
echo ""
