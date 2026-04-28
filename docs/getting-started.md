# Getting Started

## Prerequisites

- Docker (or Podman)
- PostgreSQL 14+ (or use the included Docker Compose example)

---

## 1. Start the server

```bash
docker run -d \
  --name ovlt \
  -p 3000:3000 \
  -e DATABASE_URL=postgresql://user:pass@host:5432/ovlt \
  -e OVLT_ADMIN_KEY=your-admin-key \
  -e OVLT_BOOTSTRAP_ADMIN_EMAIL=admin@example.com \
  -e OVLT_BOOTSTRAP_ADMIN_PASSWORD=Admin1234! \
  ghcr.io/shrpp/ovlt-core:latest
```

**First run:** if `JWT_SECRET`, `MASTER_ENCRYPTION_KEY`, or `TENANT_WRAP_KEY` are not set, OVLT auto-generates them and prints them to stderr:

```
  ╔══════════════════════════════════════════════════════╗
  ║           OVLT — FIRST RUN: SECRETS GENERATED       ║
  ║                                                      ║
  ║  JWT_SECRET=<base64>                                 ║
  ║  MASTER_ENCRYPTION_KEY=<base64>                      ║
  ║  TENANT_WRAP_KEY=<base64>                            ║
  ╚══════════════════════════════════════════════════════╝
```

**Save these immediately.** Losing them makes all encrypted data unrecoverable.

Re-run with the secrets pinned:

```bash
docker run -d \
  --name ovlt \
  -p 3000:3000 \
  -e DATABASE_URL=postgresql://user:pass@host:5432/ovlt \
  -e JWT_SECRET=<value-from-logs> \
  -e MASTER_ENCRYPTION_KEY=<value-from-logs> \
  -e TENANT_WRAP_KEY=<value-from-logs> \
  -e OVLT_ADMIN_KEY=your-admin-key \
  -e OVLT_BOOTSTRAP_ADMIN_EMAIL=admin@example.com \
  -e OVLT_BOOTSTRAP_ADMIN_PASSWORD=Admin1234! \
  ghcr.io/shrpp/ovlt-core:latest
```

Check it's healthy:

```bash
curl http://localhost:3000/health
# {"status":"ok","version":"x.y.z"}
```

---

## 2. Docker Compose (with Postgres)

```yaml
version: "3.9"
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_USER: ovlt
      POSTGRES_PASSWORD: ovlt
      POSTGRES_DB: ovlt
    volumes:
      - pg_data:/var/lib/postgresql/data

  ovlt:
    image: ghcr.io/shrpp/ovlt-core:latest
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://ovlt:ovlt@postgres:5432/ovlt
      OVLT_ADMIN_KEY: change-me
      OVLT_BOOTSTRAP_ADMIN_EMAIL: admin@example.com
      OVLT_BOOTSTRAP_ADMIN_PASSWORD: Admin1234!
      # Paste generated secrets here after first run
      # JWT_SECRET:
      # MASTER_ENCRYPTION_KEY:
      # TENANT_WRAP_KEY:
    depends_on:
      postgres:
        condition: service_started

volumes:
  pg_data:
```

```bash
docker compose up -d
docker compose logs ovlt   # grab generated secrets from here
```

---

## 3. Install the Admin TUI

The `ovlt` binary is a terminal UI to manage tenants, users, clients, roles, and permissions.

```bash
# macOS ARM (M1/M2/M3)
curl -Lo ovlt https://github.com/shrpp/ovlt/releases/download/latest-main/ovlt-macos-aarch64
chmod +x ovlt && sudo mv ovlt /usr/local/bin/

# macOS Intel
curl -Lo ovlt https://github.com/shrpp/ovlt/releases/download/latest-main/ovlt-macos-x86_64
chmod +x ovlt && sudo mv ovlt /usr/local/bin/

# Linux x86_64
curl -Lo ovlt https://github.com/shrpp/ovlt/releases/download/latest-main/ovlt-linux-x86_64
chmod +x ovlt && sudo mv ovlt /usr/local/bin/
```

Connect:

```bash
ovlt --url http://localhost:3000
# or set OVLT_URL=http://localhost:3000
```

---

## 4. First login

1. Open the TUI: `ovlt --url http://localhost:3000`
2. When prompted for Admin Key, enter the value you set in `OVLT_ADMIN_KEY`
3. A **master** tenant is created automatically on first startup with the bootstrap admin credentials
4. Navigate with arrow keys / Tab; press `?` for help

---

## Next steps

- [Configuration](configuration.md) — all environment variables
- [Admin TUI](admin-tui.md) — full TUI reference
- [M2M / Client Credentials](m2m.md) — machine-to-machine auth
- [API Reference](api-reference.md) — HTTP endpoints
