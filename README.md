# oakmoss (name to be decided)

> **⚠️Work in progress:** This is an early, very alpha project; expect breaking changes and rough edges while the foundations solidify. Here be dragons!

A single-runtime Discord bot engine that empowers server administrators to run TypeScript on a fast, isolated runtime.

## Prerequisites

- Rust toolchain (edition 2024)
- Bun (for SDK bundling)
- Optional: Postgres + Redis (dev script uses ports 5433/5434)

## Run the bot (local)

1. Create `.env` with at least `DISCORD_TOKEN=<your token>`. Optional overrides:
   - `DATABASE_URL` (default: `postgres://user:pass@localhost:5433/oakmoss`)
   - `VALKEY_URL` (default: `redis://127.0.0.1:5434/0`)
   - `API_ADDR` (default: `0.0.0.0:3000`)
   - `DISCORD_CLIENT_ID` / `DISCORD_CLIENT_SECRET` (required for OAuth login)
   - `DISCORD_REDIRECT_URI` (default: `http://localhost:3000/auth/callback`)
   - `SESSION_SECRET` (32+ chars for signing session cookies)
   - `SESSION_TTL_SECS` (optional, default 30 days)
   - `COOKIE_SECURE` (`true`/`false`, default based on redirect uri scheme)
2. Start supporting services (optional): `./dev.sh` (Postgres/Redis).
3. Run: `cargo run`\
   Logging defaults are already set in `.envrc`: `RUST_LOG=oakmoss=debug,oakmoss::runtime=trace,serenity=info`.
   If you are not using direnv, export that before running.

## Bot scripts

- The bundled SDK is loaded from `dist/sdk-bundle.js` at startup.
- For local development, if `scripts/bot.ts` exists it will also be loaded automatically.
- After editing the TypeScript SDK under `sdk/`, rebuild the bundle: `bun run sdk/build.ts`.

## API authentication

- Login flow: `GET /auth/login` redirects to Discord OAuth (scopes: `identify guilds guilds.members.read`). The callback at `/auth/callback` sets an HTTP-only session cookie. Use `/auth/me` to verify the session.
- Discoverable guilds: `GET /guilds` lists servers where the current user is admin/manage-guild and the bot is present.
- CLI tokens: `POST /tokens` to mint a user token (returns plaintext once), `GET /tokens` to list, `DELETE /tokens/{id}` to revoke. Tokens can be used as `Authorization: Bearer <token>` in API calls.

## Developing

- Format/lint Rust: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`.
- Tests: `cargo test`.
- Rebuild SDK bundle: `bun run sdk/build.ts` (run from repo root; dependencies via `bun install`).

## Deployment cache / migrations

- Migrations run automatically on startup via `DeploymentService::migrate`.
- Cached guild deployments are fetched and loaded on boot.
