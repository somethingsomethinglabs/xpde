#!/usr/bin/env bash
# XPDE fetch-and-build installer (similar flow to curl … | sh installers).
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/PAT0036/xpde/main/install.sh | bash
# Or after clone:
#   ./install.sh
#
# Environment (optional):
#   XPDE_REPO              Git URL (default: https://github.com/PAT0036/xpde.git)
#   XPDE_DIR               Install/clone directory (default: $HOME/xpde)
#   XPDE_BRANCH            Branch to checkout (default: main)
#   XPDE_NO_SYSTEM_DEPS=1 Skip apt installs (build-essential, etc.)

set -euo pipefail

NO_SYSTEM_DEPS="${XPDE_NO_SYSTEM_DEPS:-0}"

DEFAULT_REPO="${XPDE_REPO:-https://github.com/PAT0036/xpde.git}"
DEFAULT_DIR="${XPDE_DIR:-$HOME/xpde}"
BRANCH="${XPDE_BRANCH:-main}"
EXISTING_CHECKOUT=0

# Resolve script path only when this file exists on disk (not when streamed: curl | bash)
SCRIPT_DIR=""
if [[ -n "${BASH_SOURCE[0]:-}" && -f "${BASH_SOURCE[0]}" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
elif [[ -n "${0:-}" && -f "$0" && "$0" != bash && "$0" != /bin/bash && "$0" != /usr/bin/bash ]]; then
  SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
fi
# Use this checkout only when XPDE_DIR was not set explicitly
if [[ -z "${XPDE_DIR:-}" && -n "${SCRIPT_DIR}" && -f "${SCRIPT_DIR}/Cargo.toml" && -f "${SCRIPT_DIR}/install.sh" ]]; then
  DEFAULT_DIR="${SCRIPT_DIR}"
  EXISTING_CHECKOUT=1
fi

usage() {
  echo "Usage: install.sh [options]"
  echo "  Environment: XPDE_REPO, XPDE_DIR, XPDE_BRANCH, XPDE_NO_SYSTEM_DEPS"
  echo "  Flags: --dir PATH   Clone/update target (overrides XPDE_DIR)"
  echo "        --repo URL   Git remote (overrides XPDE_REPO)"
  echo "        --no-rust        Skip rustup / cargo build"
  echo "        --no-js          Skip bun install"
  echo "        --no-system-deps Skip apt install of build-essential (Debian/Ubuntu)"
  echo "        -h, --help       This help"
}

NO_RUST=0
NO_JS=0
DIR_EXPLICIT=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dir) DEFAULT_DIR="${2:-}"; DIR_EXPLICIT=1; EXISTING_CHECKOUT=0; shift 2 ;;
    --repo) DEFAULT_REPO="${2:-}"; shift 2 ;;
    --no-rust) NO_RUST=1; shift ;;
    --no-js) NO_JS=1; shift ;;
    --no-system-deps) NO_SYSTEM_DEPS=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "${DIR_EXPLICIT}" -eq 1 || -n "${XPDE_DIR:-}" ]]; then
  EXISTING_CHECKOUT=0
fi

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

die() {
  echo "error: $*" >&2
  exit 1
}

ensure_git_curl() {
  need_cmd git || die "install git first (e.g. sudo apt install git)"
  need_cmd curl || die "install curl first (e.g. sudo apt install curl)"
}

has_c_linker() {
  (command -v cc >/dev/null 2>&1 && cc --version >/dev/null 2>&1) ||
    (command -v gcc >/dev/null 2>&1 && gcc --version >/dev/null 2>&1)
}

is_apt_distro() {
  [[ -r /etc/os-release ]] || return 1
  # shellcheck source=/dev/null
  . /etc/os-release
  case "${ID:-}" in
    debian|ubuntu|pop|linuxmint|raspbian) return 0 ;;
  esac
  for tok in ${ID_LIKE:-}; do
    [[ "$tok" == "debian" ]] && return 0
    [[ "$tok" == "ubuntu" ]] && return 0
  done
  return 1
}

# Rust (and some native deps) need a working C toolchain (linker 'cc').
ensure_c_build_tools() {
  [[ "${NO_SYSTEM_DEPS}" -eq 0 ]] || return 0
  [[ "${NO_RUST}" -eq 0 ]] || return 0
  has_c_linker && return 0

  if is_apt_distro; then
    echo "Installing C build tools for Rust (build-essential, pkg-config, libssl-dev)…"
    if [[ "${EUID:-1}" -eq 0 ]]; then
      apt-get update -qq
      DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        build-essential pkg-config libssl-dev
    elif need_cmd sudo; then
      sudo apt-get update -qq
      DEBIAN_FRONTEND=noninteractive sudo -E apt-get install -y --no-install-recommends \
        build-essential pkg-config libssl-dev
    else
      die "No 'sudo' and not root — install manually: sudo apt install build-essential pkg-config libssl-dev"
    fi
    has_c_linker || die "C linker still missing after apt install"
    return 0
  fi

  die "Rust needs a C linker ('cc'/gcc). On Debian/Ubuntu: sudo apt install build-essential pkg-config libssl-dev"
}

ensure_bun() {
  if need_cmd bun; then
    return 0
  fi
  echo "Installing Bun…"
  curl -fsSL https://bun.sh/install | bash
  export BUN_INSTALL="${BUN_INSTALL:-$HOME/.bun}"
  export PATH="$BUN_INSTALL/bin:$PATH"
}

ensure_cargo() {
  if need_cmd cargo; then
    return 0
  fi
  echo "Installing Rust (rustup)…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck source=/dev/null
  [[ -f "${HOME}/.cargo/env" ]] && source "${HOME}/.cargo/env"
  export PATH="${HOME}/.cargo/bin:${PATH}"
}

clone_or_update() {
  local dest="$1"
  local repo="$2"
  local branch="$3"

  if [[ -d "${dest}/.git" ]]; then
    echo "Updating existing clone: ${dest}"
    git -C "${dest}" fetch --depth 1 origin "${branch}" || true
    git -C "${dest}" checkout "${branch}" 2>/dev/null || git -C "${dest}" checkout -B "${branch}" "origin/${branch}" 2>/dev/null || true
    git -C "${dest}" pull --ff-only origin "${branch}" || git -C "${dest}" pull --ff-only || true
  else
    echo "Cloning ${repo} → ${dest} (branch ${branch})"
    mkdir -p "$(dirname "${dest}")"
    git clone --depth 1 --branch "${branch}" "${repo}" "${dest}" || {
      echo "Clone with branch failed; trying default branch…"
      git clone --depth 1 "${repo}" "${dest}"
      git -C "${dest}" checkout "${branch}" || true
    }
  fi
}

main() {
  ensure_git_curl
  ensure_c_build_tools

  if [[ "${NO_JS}" -eq 0 ]]; then
    ensure_bun
  fi
  if [[ "${NO_RUST}" -eq 0 ]]; then
    ensure_cargo
  fi

  if [[ "${EXISTING_CHECKOUT}" -eq 1 ]]; then
    echo "Using existing checkout: ${DEFAULT_DIR}"
    if [[ -d "${DEFAULT_DIR}/.git" ]]; then
      git -C "${DEFAULT_DIR}" pull --ff-only origin "${BRANCH}" 2>/dev/null || git -C "${DEFAULT_DIR}" pull --ff-only || true
    fi
  else
    clone_or_update "${DEFAULT_DIR}" "${DEFAULT_REPO}" "${BRANCH}"
  fi

  cd "${DEFAULT_DIR}" || die "cannot cd to ${DEFAULT_DIR}"

  if [[ "${NO_JS}" -eq 0 ]]; then
    echo "Running bun install…"
    bun install --frozen-lockfile 2>/dev/null || bun install
  fi

  if [[ "${NO_RUST}" -eq 0 ]]; then
    echo "Running cargo build --workspace…"
    cargo build --workspace
  fi

  echo ""
  echo "XPDE is ready at: ${DEFAULT_DIR}"
  echo "  cd ${DEFAULT_DIR}"
  echo "  bun run lint"
  echo "  cargo run -p xpde-shelld"
}

main "$@"
