#!/usr/bin/env bash
# install-skills.sh -- Deploy skills to Claude, Gemini, or Codex destinations.
#
# Usage:
#   lib/install-skills.sh <skills_dir> --provider claude|gemini|codex [options]
#
# Options:
#   --dry-run
#   --clean
#   --scope user|workspace     (gemini only)
#   --dst /path/to/skills
#   --agents-dir /path/to/agents
#   --include-agent-wrappers
#   --keep-temp

set -euo pipefail

SCRIPT_DIR="$(builtin cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

yaml_top_value() {
  local file="$1"
  local key="$2"
  awk -v key="$key" '
    $0 ~ "^" key ":[[:space:]]*" {
      sub("^" key ":[[:space:]]*", "")
      gsub(/^"|"$/, "")
      gsub(/^[[:space:]]+|[[:space:]]+$/, "")
      print
      exit
    }
  ' "$file"
}

yaml_provider_value() {
  local file="$1"
  local provider="$2"
  local key="$3"
  awk -v provider="$provider" -v key="$key" '
    /^providers:[[:space:]]*$/ { in_providers=1; next }
    in_providers && /^[^[:space:]]/ { in_providers=0 }

    in_providers && $0 ~ "^  " provider ":[[:space:]]*$" {
      in_provider=1
      next
    }
    in_provider && $0 ~ "^  [^[:space:]]" {
      in_provider=0
    }
    in_provider && $0 ~ "^    " key ":[[:space:]]*" {
      sub("^    " key ":[[:space:]]*", "")
      gsub(/^"|"$/, "")
      gsub(/^[[:space:]]+|[[:space:]]+$/, "")
      print
      exit
    }
  ' "$file"
}

skill_enabled_for_provider() {
  local skill_yaml="$1"
  local provider="$2"

  if [ ! -f "$skill_yaml" ]; then
    return 1
  fi

  local value
  value="$(yaml_provider_value "$skill_yaml" "$provider" "enabled")"
  case "$value" in
    true|yes|1) return 0 ;;
    *) return 1 ;;
  esac
}

validate_skill_metadata() {
  local skill_dir="$1"
  local skill_yaml="$skill_dir/SKILL.yaml"

  if [ ! -f "$skill_yaml" ]; then
    echo "Error: Missing SKILL.yaml in $skill_dir"
    return 1
  fi

  local key
  for key in name description argument-hint; do
    if [ -z "$(yaml_top_value "$skill_yaml" "$key")" ]; then
      echo "Error: $skill_yaml is missing required key: $key"
      return 1
    fi
  done

  return 0
}

copy_skill_to_destination() {
  local skill_dir="$1"
  local dst_dir="$2"
  local dry_run="$3"

  local skill_yaml="$skill_dir/SKILL.yaml"
  local skill_name
  skill_name="$(yaml_top_value "$skill_yaml" "name")"

  if [ -z "$skill_name" ]; then
    skill_name="$(basename "$skill_dir")"
  fi

  if [ "$dry_run" = "--dry-run" ]; then
    echo "[dry-run] Would install skill: $skill_name to $dst_dir"
    return 0
  fi

  mkdir -p "$dst_dir"
  rm -rf "${dst_dir:?}/$skill_name"
  cp -R "$skill_dir" "$dst_dir/$skill_name"
  echo "Installed skill: $skill_name -> $dst_dir"
}

install_skill() {
  local skill_dir="$1"
  local provider="$2"
  local dst_dir="$3"
  local dry_run="$4"
  local scope="$5"

  local skill_yaml="$skill_dir/SKILL.yaml"

  validate_skill_metadata "$skill_dir"

  if ! skill_enabled_for_provider "$skill_yaml" "$provider"; then
    return 0
  fi

  if [ "$provider" = "gemini" ]; then
    if ! command -v gemini >/dev/null 2>&1; then
      echo "Skipping Gemini skill installation (gemini CLI not found)"
      return 0
    fi

    local skill_name
    skill_name="$(yaml_top_value "$skill_yaml" "name")"
    
    # Fail early if user created config.yml instead of config.yaml
    if [ -f "config.yml" ]; then
      echo "Error: config.yml is not supported. Rename to config.yaml." >&2
      return 1
    fi

    # Sidecar override for scope
    local sidecar_file="defaults.yaml"
    if [ -f "config.yaml" ]; then
      sidecar_file="config.yaml"
    fi

    if [ -f "$sidecar_file" ]; then
      local sidecar_scope
      sidecar_scope=$(awk -v skill="$skill_name" '
        $0 ~ "^" skill ":" { in_target=1; next }
        in_target && /^[^ ]/ { in_target=0 }
        in_target && $0 ~ "^  scope:[ ]*" {
          sub("^[ ]*scope:[ ]*", "", $0)
          print
          exit
        }
      ' "$sidecar_file")
      if [ -n "$sidecar_scope" ]; then
        scope="$sidecar_scope"
      fi
    fi

    local configured_scope
    configured_scope="$(yaml_provider_value "$skill_yaml" "$provider" "scope")"
    if [ -n "$configured_scope" ]; then
      # If not overridden by sidecar, use configured scope
      if [ -z "${sidecar_scope:-}" ]; then
        scope="$configured_scope"
      fi
    fi

    if [ "$dry_run" = "--dry-run" ]; then
      echo "[dry-run] Would install Gemini skill: $skill_name (scope: $scope)"
    else
      echo "Installing Gemini skill: $skill_name (scope: $scope)..."
      gemini skills install "$skill_dir" --scope "$scope" --consent
    fi
  else
    copy_skill_to_destination "$skill_dir" "$dst_dir" "$dry_run"
  fi

  return 0
}

install_skills_from_root() {
  local root_dir="$1"
  local provider="$2"
  local dst_dir="$3"
  local dry_run="$4"
  local scope="$5"

  [ -d "$root_dir" ] || return 0

  local skill_dir
  for skill_dir in "$root_dir"/*; do
    if [ -d "$skill_dir" ] && [ -f "$skill_dir/SKILL.md" ]; then
      install_skill "$skill_dir" "$provider" "$dst_dir" "$dry_run" "$scope"
    fi
  done
}

main() {
  local skills_dir=""
  local provider="gemini"
  local dry_run=""
  local clean=false
  local scope="workspace"
  local dst_dir=""
  local agents_dir="agents"
  local include_agent_wrappers=false
  local keep_temp=false

  local key=""
  for arg in "$@"; do
    # Skip empty arguments
    [[ -z "$arg" ]] && continue

    if [[ -n "$key" ]]; then
      case "$key" in
        --provider)   provider="$arg" ;;
        --scope)      scope="$arg" ;;
        --dst)        dst_dir="$arg" ;;
        --agents-dir) agents_dir="$arg" ;;
      esac
      key=""
      continue
    fi

    case "$arg" in
      --provider|--scope|--dst|--agents-dir)
        key="$arg"
        ;;
      --dry-run) dry_run="--dry-run" ;;
      --clean)   clean=true ;;
      --include-agent-wrappers) include_agent_wrappers=true ;;
      --keep-temp) keep_temp=true ;;
      -h|--help)
        echo "Usage: install-skills.sh <skills_dir> --provider claude|gemini|codex [--dry-run] [--clean] [--scope user|workspace] [--dst path] [--agents-dir path] [--include-agent-wrappers] [--keep-temp]"
        exit 0
        ;;
      -*)
        echo "Error: Unexpected option: $arg"
        exit 1
        ;;
      *)
        if [[ -z "$skills_dir" ]]; then
          if [[ -d "$arg" ]]; then
            skills_dir="$arg"
          else
            echo "Error: Invalid skills directory: $arg"
            exit 1
          fi
        else
          echo "Error: Unexpected argument: $arg"
          exit 1
        fi
        ;;
    esac
  done

  if [[ -n "$key" ]]; then
    echo "Error: Option $key requires an argument"
    exit 1
  fi

  if [ -z "$skills_dir" ]; then
    echo "Error: skills_dir is required"
    exit 1
  fi

  case "$provider" in
    claude|gemini|codex) ;;
    *)
      echo "Error: Invalid provider '$provider' (expected claude, gemini, or codex)"
      exit 1
      ;;
  esac

  if [ -z "$dst_dir" ]; then
    case "$provider" in
      claude) dst_dir="${HOME}/.claude/skills" ;;
      codex) dst_dir="${HOME}/.codex/skills" ;;
      gemini) dst_dir="${HOME}/.gemini/skills" ;;
    esac
  fi

  local tmp_dir
  tmp_dir="$(mktemp -d)"
  local keep_temp_value="$keep_temp"
  cleanup_temp() {
    if [ "${keep_temp_value:-false}" = true ]; then
      echo "Keeping temp directory: ${tmp_dir:-}"
      return 0
    fi
    rm -rf "${tmp_dir:-}"
  }
  trap cleanup_temp EXIT

  local stage_manual="$tmp_dir/manual"
  local stage_generated="$tmp_dir/generated"
  mkdir -p "$stage_manual"
  cp -R "$skills_dir"/. "$stage_manual"/

  if [ "$include_agent_wrappers" = true ]; then
    if [ ! -d "$agents_dir" ]; then
      echo "Error: agents directory not found: $agents_dir"
      exit 1
    fi

    if [ -n "$dry_run" ]; then
      bash "$SCRIPT_DIR/generate-agent-skills.sh" "$agents_dir" "$stage_generated" "$dry_run"
    else
      bash "$SCRIPT_DIR/generate-agent-skills.sh" "$agents_dir" "$stage_generated"
    fi
  fi

  if [ "$clean" = true ] && [ "$provider" != "gemini" ] && [ -d "$dst_dir" ]; then
    local skill_dir
    for skill_dir in "$stage_manual"/* "$stage_generated"/*; do
      [ -d "$skill_dir" ] || continue
      [ -f "$skill_dir/SKILL.yaml" ] || continue
      if skill_enabled_for_provider "$skill_dir/SKILL.yaml" "$provider"; then
        local skill_name
        skill_name="$(yaml_top_value "$skill_dir/SKILL.yaml" "name")"
        if [ -n "$skill_name" ] && [ -d "$dst_dir/$skill_name" ]; then
          if [ "$dry_run" = "--dry-run" ]; then
            echo "[dry-run] Would remove skill: $dst_dir/$skill_name"
          else
            rm -rf "${dst_dir:?}/$skill_name"
            echo "Removed skill: $dst_dir/$skill_name"
          fi
        fi
      fi
    done
  fi

  install_skills_from_root "$stage_manual" "$provider" "$dst_dir" "$dry_run" "$scope"
  install_skills_from_root "$stage_generated" "$provider" "$dst_dir" "$dry_run" "$scope"
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi
