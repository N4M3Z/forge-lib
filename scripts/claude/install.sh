#!/usr/bin/env bash
# Fallback installer for Claude Code skills, agents, and rules.
# Replaces the Rust install-skills/install-agents binaries with pure shell
# file copies. Use when the Rust toolchain is unavailable.
#
# Usage: install.sh [--scope workspace|user|project] [--modules-dir <path>] [--vault-skills <path>]
# Default: --scope workspace --modules-dir Modules

set -euo pipefail

VERSION="0.1.0"

SCOPE="workspace"
MODULES_DIR="Modules"
VAULT_SKILLS=""

print_usage() {
    echo "Usage: install.sh [--scope workspace|user|project] [--modules-dir <path>] [--vault-skills <path>]"
    echo ""
    echo "Fallback installer — copies skills, agents, and rules without the Rust toolchain."
    echo ""
    echo "Options:"
    echo "  --scope <scope>         Destination scope: workspace, user, or project (default: workspace)"
    echo "  --modules-dir <path>    Path to modules directory (default: Modules)"
    echo "  --vault-skills <path>   Extra skills source directory (e.g. Obsidian vault skills)"
    echo "  --version               Print version and exit"
    echo "  -h, --help              Print this help and exit"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --version)
            echo "install.sh ${VERSION}"
            exit 0
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        --scope)
            if [ $# -lt 2 ]; then
                echo "Error: --scope requires a value" >&2
                exit 1
            fi
            SCOPE="$2"
            shift 2
            ;;
        --modules-dir)
            if [ $# -lt 2 ]; then
                echo "Error: --modules-dir requires a value" >&2
                exit 1
            fi
            MODULES_DIR="$2"
            shift 2
            ;;
        --vault-skills)
            if [ $# -lt 2 ]; then
                echo "Error: --vault-skills requires a value" >&2
                exit 1
            fi
            VAULT_SKILLS="$2"
            shift 2
            ;;
        -*)
            echo "Error: unknown flag $1" >&2
            exit 1
            ;;
        *)
            echo "Error: unexpected argument $1" >&2
            exit 1
            ;;
    esac
done

if [ ! -d "$MODULES_DIR" ]; then
    echo "Error: modules directory not found: ${MODULES_DIR}" >&2
    exit 1
fi

# --- Scope resolution ---

resolve_project_key() {
    pwd | tr '/' '-'
}

resolve_destinations() {
    local scope="$1"
    local home="${HOME}"

    case "$scope" in
        workspace)
            SKILLS_DST=".claude/skills"
            AGENTS_DST=".claude/agents"
            RULES_DST=".claude/rules"
            ;;
        user)
            SKILLS_DST="${home}/.claude/skills"
            AGENTS_DST="${home}/.claude/agents"
            RULES_DST="${home}/.claude/rules"
            ;;
        project)
            local key
            key="$(resolve_project_key)"
            SKILLS_DST="${home}/.claude/projects/${key}/skills"
            AGENTS_DST="${home}/.claude/projects/${key}/agents"
            RULES_DST="${home}/.claude/projects/${key}/rules"
            ;;
        *)
            echo "Error: invalid scope '${scope}' (use workspace, user, or project)" >&2
            exit 1
            ;;
    esac
}

resolve_destinations "$SCOPE"

# --- Recursive copy (matches Rust copy_dir_recursive, skips SKILL.yaml) ---

copy_skill_dir() {
    local src="$1"
    local dst="$2"

    mkdir -p "$dst"

    for entry in "$src"/*; do
        [ -e "$entry" ] || continue

        local name
        name="$(basename "$entry")"

        if [ "$name" = "SKILL.yaml" ]; then
            continue
        fi

        local dst_path="${dst}/${name}"
        if [ -d "$entry" ]; then
            copy_skill_dir "$entry" "$dst_path"
        else
            command cp "$entry" "$dst_path"
        fi
    done
}

# --- Skills ---

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

install_skills() {
    mkdir -p "$SKILLS_DST"

    local total=0

    for module_dir in "$MODULES_DIR"/*/; do
        [ -d "$module_dir" ] || continue
        local skills_dir="${module_dir}skills"
        if [ -d "$skills_dir" ]; then
            local installed
            installed="$(install_skills_from_dir "$skills_dir")"
            total=$((total + installed))
        fi
    done

    if [ -n "$VAULT_SKILLS" ] && [ -d "$VAULT_SKILLS" ]; then
        local vault_installed
        vault_installed="$(install_skills_from_dir "$VAULT_SKILLS")"
        total=$((total + vault_installed))
    fi

    echo "  ${total} skills installed to ${SKILLS_DST}"
}

# --- Agents ---

install_agents() {
    mkdir -p "$AGENTS_DST"

    local total=0

    for module_dir in "$MODULES_DIR"/*/; do
        [ -d "$module_dir" ] || continue
        local agents_dir="${module_dir}agents"
        if [ -d "$agents_dir" ]; then
            for agent_file in "$agents_dir"/*.md; do
                [ -f "$agent_file" ] || continue
                command cp "$agent_file" "$AGENTS_DST/"
                total=$((total + 1))
            done
        fi
    done

    echo "  ${total} agents installed to ${AGENTS_DST}"
}

# --- Rules ---

install_rules() {
    command rm -rf "$RULES_DST"
    mkdir -p "$RULES_DST"

    local total=0

    for module_dir in "$MODULES_DIR"/*/; do
        [ -d "$module_dir" ] || continue
        local rules_dir="${module_dir}rules"
        if [ -d "$rules_dir" ]; then
            for rule_file in "$rules_dir"/*.md; do
                [ -f "$rule_file" ] || continue
                command cp "$rule_file" "$RULES_DST/"
                total=$((total + 1))
            done

            for sub_dir in "$rules_dir"/*/; do
                [ -d "$sub_dir" ] || continue
                local sub_name
                sub_name="$(basename "$sub_dir")"
                local dst_sub="${RULES_DST}/${sub_name}"
                mkdir -p "$dst_sub"
                command cp -r "$sub_dir"/* "$dst_sub/" 2>/dev/null || true
                while IFS= read -r _; do
                    total=$((total + 1))
                done < <(find "$dst_sub" -name '*.md' -type f 2>/dev/null)
            done
        fi
    done

    echo "  ${total} rules installed to ${RULES_DST}"
}

# --- Main ---

echo "=== Forge Fallback Installer ==="
echo "  scope: ${SCOPE}"
echo "  modules: ${MODULES_DIR}"
if [ -n "$VAULT_SKILLS" ]; then
    echo "  vault-skills: ${VAULT_SKILLS}"
fi

install_skills
install_agents
install_rules

echo "Installation complete."
