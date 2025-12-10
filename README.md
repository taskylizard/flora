# oakmoss (name to be decided)

## Status
**⚠️Work in progress:** This is an early, very alpha project; expect breaking changes and rough edges while the foundations solidify. Here be dragons!

## Quick Start
- Set `DISCORD_TOKEN` in a `.env` file, then run `RUST_LOG=oakmoss=debug,oakmoss::runtime=trace,serenity=info cargo run` to boot the bot with the default `scripts/bot.js`.
- Edit scripts under `scripts/` and rebuild the SDK bundle with `bun run sdk/build.ts` after SDK changes.
- Optional: run `./dev.sh` to start Postgres/Redis on ports 5433/5434 for features that need storage.
