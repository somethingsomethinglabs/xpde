# XPDE Developer Notes

## Build

- `bun install`
- `cargo build --workspace`
- `bun run build` (workspace packages under `packages/`; apps use `bun run build:apps` once Vite entry exists)

## Lint

- `bun run lint` — ESLint (JS/TS/Svelte) + `cargo fmt` / `cargo clippy`
- `bun run lint:fix` — auto-fix ESLint + format Rust

## Web objects / D-Bus

Run the daemon: `cargo run -p xpde-shelld`

See [web-objects.md](./web-objects.md) for `org.xpde.Web1` methods and `busctl` examples.

## Run daemon

- `cargo run -p xpde-shelld`

## Session notes

`session/xpde-session` is the entrypoint used by display managers.
