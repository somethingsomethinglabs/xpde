# XPDE (Debian XP Workbench)

XPDE is a modern reimagining of the Windows 98/XP shell on Debian Linux.
It uses `labwc` for Wayland compositing, `SvelteKit + Tauri v2` for shell UI,
and Rust services for Linux integration and web-object orchestration.

## Workspaces

- JavaScript/TypeScript packages are managed with `pnpm` workspaces.
- Rust crates are managed with a Cargo workspace.

## Quick start

1. Install Rust stable and Node.js 20+.
2. Run `pnpm install` (Corepack: `corepack enable`). Workspace packages use `workspace:*`; **pnpm is required** for full installs. For ESLint-only at the root without workspaces, `npm install` in the repo root still installs shared dev tools listed in the root `package.json`.
3. Run `cargo build --workspace`.
4. Run one app in dev mode, for example `pnpm --filter @xpde/control dev`.

## Linting

- **JavaScript / TypeScript / Svelte**: [ESLint](https://eslint.org/) 9 flat config with [`typescript-eslint`](https://typescript-eslint.io/) and [`eslint-plugin-svelte`](https://sveltejs.github.io/eslint-plugin-svelte/) — the usual, best-supported stack for SvelteKit-style projects (Biome’s Svelte support is still catching up).
- **Rust**: `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`.

From the repo root:

```bash
pnpm lint
# or
npm run lint
```
