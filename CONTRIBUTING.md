# Contributing to OVLT

Thanks for your interest. OVLT is licensed under the [Elastic License 2.0](LICENSE) — you can use, modify, and contribute freely; you cannot resell it or offer it as a managed service.

## How to contribute

- **Bug reports** — open an issue with steps to reproduce, expected vs actual behavior, and your environment (OS, Rust version, PostgreSQL version)
- **Feature requests** — open an issue first to discuss before building, especially for anything that touches the auth flow, encryption, or multi-tenancy logic
- **Pull requests** — small, focused changes are preferred; one concern per PR

## Development setup

```bash
git clone <repo-url>
cd ovlt
cp ovlt-core/.env.example ovlt-core/.env

docker compose up -d postgres

cd ovlt-core
cargo run -- --migrate
```

Run tests:

```bash
cargo test
```

Clippy (CI will enforce this):

```bash
cargo clippy -- -D warnings
```

## Code style

- `cargo fmt` before every commit
- No `unwrap()` in non-test code — use `?` or explicit error handling
- No dead code — remove unused functions/imports rather than commenting them out
- Secrets never in logs — check before adding any `tracing::*` calls that reference user data
- All new endpoints need an entry in the API table in `README.md`

## Database migrations

Migrations live in `ovlt-core/migration/src/`. Every migration must implement both `up` and `down`. Test both directions before opening a PR.

## Security issues

Do **not** open a public issue for security vulnerabilities. Email me@shrpp.dev with details.

## Commit messages

Use the imperative present tense: `add X`, `fix Y`, `remove Z`. One subject line (≤72 chars), optional body for context.
