# XPDE (Debian XP Workbench)

XPDE is a modern reimagining of the Windows 98/XP shell on Debian Linux.
It uses `labwc` for Wayland compositing, `SvelteKit + Tauri v2` for shell UI,
and Rust services for Linux integration and web-object orchestration.

## Workspaces

- JavaScript/TypeScript packages use **Bun** workspaces (`apps/*`, `packages/*`).
- Rust crates use a Cargo workspace.

## One-line install (clone + toolchain + build)

Same idea as `curl … | sh` installers: clones the repo under `$HOME/xpde` (or `$XPDE_DIR`), installs [Bun](https://bun.sh/) and Rust if missing, then runs `bun install` and `cargo build --workspace`.

```bash
curl -fsSL https://raw.githubusercontent.com/PAT0036/xpde/main/install.sh | bash
```

Or clone first, then run the script from the repo (skips a second clone, only pulls):

```bash
git clone https://github.com/PAT0036/xpde.git
cd xpde
chmod +x install.sh
./install.sh
```

Override destination, branch, or use a fork (variables are read by the downloaded script):

```bash
export XPDE_DIR="$HOME/src/xpde"
export XPDE_REPO="https://github.com/yourfork/xpde.git"
export XPDE_BRANCH="main"
curl -fsSL https://raw.githubusercontent.com/PAT0036/xpde/main/install.sh | bash
```

Flags: `--dir PATH`, `--repo URL`, `--no-rust`, `--no-js`, `--help`.

**Platforms:** Linux and macOS (Bash). On Windows, use WSL or Git Bash.

## Quick start (manual)

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
