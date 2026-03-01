# CodexAdapter

Fallback installation for Codex CLI.

## Detection

```bash
codex --version
```

## Install

```bash
bash Modules/forge-lib/scripts/codex/install.sh
```

Scope options: `--scope workspace` (default), `--scope user`.

```bash
bash Modules/forge-lib/scripts/codex/install.sh --scope user
bash Modules/forge-lib/scripts/codex/install.sh --vault-skills Vaults/Personal/Orchestration/Skills
```

## What Gets Installed

| Content | Destination | Format |
|---------|-------------|--------|
| Skills | `.codex/skills/<Name>/` | Directory (SKILL.md + companions), SKILL.yaml excluded |

## What's NOT Installed (Requires Rust)

| Content | Why |
|---------|-----|
| Agents | Codex agents use TOML format (`.codex/agents/<Name>.toml`) with triple-quoted `developer_instructions`. The Rust `install-agents` binary renders markdown → TOML and manages `.codex/config.toml` agent blocks. |
| Rules | Codex has no instructional rules directory. `.codex/rules/` exists for **command execution permissions** (Starlark `.rules` files), not AI instructions. Project instructions go in `AGENTS.md`. |

## Provider Notes

- Skill names are PascalCase (same as Claude)
- Agent format is TOML, not markdown — requires fields: `name`, `model`, `description`, `developer_instructions`
- `AGENTS.md` in the project root is the primary instruction file (like Claude's `CLAUDE.md`)
- Override instructions via `AGENTS.override.md` or `$CODEX_HOME/AGENTS.md`
- Config in `.codex/config.toml` — agent entries are managed blocks written by `install-agents`
- Max combined `AGENTS.md` size: 32 KiB

## Manual Agent Setup

If you need agents without the Rust binary, generate the base config:

```bash
codex init
```

Then manually define agents in `.codex/config.toml`:

```toml
[agents.default]
model = "o4-mini"
developer_instructions = "Follow project conventions."
```

## Verification

```bash
ls .codex/skills/ | wc -l
codex --help 2>/dev/null || echo "codex CLI not available"
```
