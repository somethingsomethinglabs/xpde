# XPDE (Debian XP Workbench)

XPDE is a modern reimagining of the Windows 98/XP shell on Debian Linux.
It uses `labwc` for Wayland compositing, `SvelteKit + Tauri v2` for shell UI,
and Rust services for Linux integration and web-object orchestration.

## Workspaces

- JavaScript/TypeScript packages use **Bun** workspaces (`apps/*`, `packages/*`).
- Rust crates use a Cargo workspace.

## Quick start

1. Install [Bun](https://bun.sh/) and Rust stable.
2. Run `bun install` at the repo root (links `workspace:*` packages).
3. Run `cargo build --workspace`.
4. Run `bun run build` (packages only until each app has a full Vite/SvelteKit scaffold).
5. Run an app in dev mode, for example `bun run dev:control`.

When app stubs gain `index.html` / SvelteKit config, use `bun run build:apps` for all `apps/*` production builds.

## Linting

- **JavaScript / TypeScript / Svelte**: ESLint 9 + typescript-eslint + eslint-plugin-svelte.
- **Rust**: `cargo fmt --check` and `cargo clippy`.

From the repo root:

```bash
bun run lint
```
