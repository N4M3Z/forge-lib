#!/usr/bin/env bash
set -euo pipefail

# Fallback installer for Codex CLI when Rust toolchain is unavailable.
# Installs skills via direct copy. Agents require TOML format — prints
# instructions instead of generating TOML (fragile in shell).
#
# Usage: install.sh [--scope workspace|user] [--modules-dir <path>] [--vault-skills <path>]

VERSION="0.1.0"

SCOPE="workspace"
MODULES_DIR="Modules"
VAULT_SKILLS=""

print_usage() {
    echo "Usage: install.sh [--scope workspace|user] [--modules-dir <path>] [--vault-skills <path>]"
    echo ""
    echo "Fallback installer for Codex CLI — copies skills without the Rust toolchain."
    echo ""
    echo "Options:"
    echo "  --scope <scope>         Destination scope: workspace or user (default: workspace)"
    echo "  --modules-dir <path>    Path to modules directory (default: Modules)"
    echo "  --vault-skills <path>   Extra skills source directory"
    echo "  --version               Print version and exit"
    echo "  -h, --help              Print this help and exit"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --version)      echo "install.sh ${VERSION} (codex)"; exit 0 ;;
        -h|--help)      print_usage; exit 0 ;;
        --scope)        [ $# -lt 2 ] && { echo "Error: --scope requires a value" >&2; exit 1; }; SCOPE="$2"; shift 2 ;;
        --modules-dir)  [ $# -lt 2 ] && { echo "Error: --modules-dir requires a value" >&2; exit 1; }; MODULES_DIR="$2"; shift 2 ;;
        --vault-skills) [ $# -lt 2 ] && { echo "Error: --vault-skills requires a value" >&2; exit 1; }; VAULT_SKILLS="$2"; shift 2 ;;
        -*)             echo "Error: unknown flag $1" >&2; exit 1 ;;
        *)              echo "Error: unexpected argument $1" >&2; exit 1 ;;
    esac
done

if [ ! -d "$MODULES_DIR" ]; then
    echo "Error: modules directory not found: ${MODULES_DIR}" >&2
    exit 1
fi

# --- Scope resolution ---

case "$SCOPE" in
    workspace)  SKILLS_DST=".codex/skills" ;;
    user)       SKILLS_DST="${HOME}/.codex/skills" ;;
    *)          echo "Error: invalid scope '${SCOPE}' (use workspace or user)" >&2; exit 1 ;;
esac

# --- Skills ---
# Codex skills use the same directory structure as Claude (SKILL.md + companions).

copy_skill_dir() {
    local src="$1"
    local dst="$2"

    mkdir -p "$dst"

    for entry in "$src"/*; do
        [ -e "$entry" ] || continue
        local name
        name="$(basename "$entry")"
        [ "$name" = "SKILL.yaml" ] && continue
        local dst_path="${dst}/${name}"
        if [ -d "$entry" ]; then
            copy_skill_dir "$entry" "$dst_path"
        else
            command cp "$entry" "$dst_path"
        fi
    done
}

install_skills_from_dir() {
    local skills_root="$1"
    local count=0

    for skill_dir in "$skills_root"/*/; do
        [ -d "$skill_dir" ] || continue
        local skill_name
        skill_name="$(basename "$skill_dir")"
        local target="${SKILLS_DST}/${skill_name}"

        if [ -d "$target" ]; then
            command rm -rf "$target"
        fi

        copy_skill_dir "$skill_dir" "$target"
        count=$((count + 1))
    done

    echo "$count"
}

mkdir -p "$SKILLS_DST"
total=0

for module_dir in "$MODULES_DIR"/*/; do
    [ -d "$module_dir" ] || continue
    skills_dir="${module_dir}skills"
    if [ -d "$skills_dir" ]; then
        installed="$(install_skills_from_dir "$skills_dir")"
        total=$((total + installed))
    fi
done

if [ -n "$VAULT_SKILLS" ] && [ -d "$VAULT_SKILLS" ]; then
    vault_installed="$(install_skills_from_dir "$VAULT_SKILLS")"
    total=$((total + vault_installed))
fi

echo "  ${total} skills installed to ${SKILLS_DST}"

# --- Summary ---

echo ""
echo "Codex fallback install complete (scope: ${SCOPE})."
echo "  Skills: ${total}"
echo ""
echo "Note: Codex agents require TOML format (.codex/agents/*.toml) with"
echo "developer_instructions fields. Run 'codex init' to generate AGENTS.md."
echo "Agent TOML generation requires the Rust install-agents binary."
echo "To unlock full functionality: make deps && make install"
